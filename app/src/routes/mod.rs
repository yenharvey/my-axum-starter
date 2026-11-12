//! 路由装配
//!
//! 将系统端点、各版本 API、文档和 404 兜底组合成最终的 [`Router`]。

mod docs;
mod fallback;
mod system;
pub mod v1;

use std::sync::Arc;

use aide::axum::ApiRouter;
use aide::openapi::OpenApi;
use axum::{Extension, Router, routing::get};
use tower_http::services::ServeDir;

use crate::AppState;

/// 构建应用的完整路由（不含全局中间件）
///
/// # 参数
/// * `state` - 应用状态
/// * `enable_docs` - 是否挂载 `/docs` API 文档路由
pub fn router(state: Arc<AppState>, enable_docs: bool) -> Router {
    aide::generate::on_error(|error| tracing::warn!("OpenAPI 生成错误: {error}"));
    aide::generate::extract_schemas(true);

    let mut api = OpenApi::default();

    let mut router = ApiRouter::new()
        .nest_service("/static", ServeDir::new("app/assets"))
        .route("/health", get(system::health_check))
        .route("/", get(system::hello_world))
        .route("/favicon.ico", get(system::favicon))
        .nest_api_service("/v1", v1::routes(state.clone()));

    if enable_docs {
        router = router.nest_api_service("/docs", docs::routes());
    }

    router
        .finish_api_with(&mut api, docs::api_docs)
        .fallback(fallback::handle_404)
        .layer(Extension(Arc::new(api)))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::jwt::JwtService;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    /// 构造不连接数据库的应用状态，仅用于验证路由装配
    fn test_state() -> Arc<AppState> {
        Arc::new(AppState {
            db: sea_orm::DatabaseConnection::default(),
            redis: None,
            jwt_service: JwtService::new("test-secret-at-least-32-characters-long", 3600),
        })
    }

    async fn status_of(router: Router, uri: &str, bearer: Option<&str>) -> StatusCode {
        let mut request = Request::builder().uri(uri);
        if let Some(token) = bearer {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        router
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn system_routes_and_fallback() {
        let router = router(test_state(), true);

        assert_eq!(
            status_of(router.clone(), "/health", None).await,
            StatusCode::OK
        );
        assert_eq!(status_of(router.clone(), "/", None).await, StatusCode::OK);
        assert_eq!(
            status_of(router.clone(), "/favicon.ico", None).await,
            StatusCode::OK
        );
        assert_eq!(
            status_of(router.clone(), "/docs/private/api.json", None).await,
            StatusCode::OK
        );
        assert_eq!(
            status_of(router, "/no-such-route", None).await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn docs_are_not_mounted_when_disabled() {
        let router = router(test_state(), false);

        assert_eq!(
            status_of(router, "/docs/private/api.json", None).await,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn login_is_rate_limited_with_json_error() {
        use axum::extract::ConnectInfo;
        use std::net::SocketAddr;

        // 限流按对端 IP 计数，测试里手动注入连接信息
        let peer = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
        let router = router(test_state(), false).layer(Extension(peer));

        let login = || {
            Request::builder()
                .method("POST")
                .uri("/v1/user/login")
                .body(Body::empty())
                .unwrap()
        };

        // 突发配额内的请求会到达处理器（空请求体被拒绝，但不是 429）
        let first = router.clone().oneshot(login()).await.unwrap();
        assert_ne!(first.status(), StatusCode::TOO_MANY_REQUESTS);

        // 连续请求耗尽配额后返回统一格式的 429
        let mut last = first;
        for _ in 0..5 {
            last = router.clone().oneshot(login()).await.unwrap();
        }
        assert_eq!(last.status(), StatusCode::TOO_MANY_REQUESTS);

        let body = axum::body::to_bytes(last.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], 429);
        assert_eq!(json["error"]["errors"][0]["reason"], "RATE_LIMIT_EXCEEDED");
    }

    #[tokio::test]
    async fn user_list_requires_auth_and_validates_pagination() {
        let state = test_state();
        let token = state.jwt_service.generate_token(1).unwrap();
        let router = router(state, false);

        assert_eq!(
            status_of(router.clone(), "/v1/user", None).await,
            StatusCode::UNAUTHORIZED
        );
        // 分页参数在查询数据库之前校验
        assert_eq!(
            status_of(router.clone(), "/v1/user?page=0", Some(&token)).await,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status_of(router.clone(), "/v1/user?page_size=101", Some(&token)).await,
            StatusCode::BAD_REQUEST
        );

        // 参数类型不对时也返回统一格式的 JSON 错误，而不是 axum 默认的纯文本
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/v1/user?page=abc")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], 400);
    }

    #[tokio::test]
    async fn cursor_user_list_requires_auth_and_validates_params() {
        let state = test_state();
        let token = state.jwt_service.generate_token(1).unwrap();
        let router = router(state, false);

        assert_eq!(
            status_of(router.clone(), "/v1/user/cursor", None).await,
            StatusCode::UNAUTHORIZED
        );
        // 参数和游标都在查询数据库之前校验
        for uri in [
            "/v1/user/cursor?limit=0",
            "/v1/user/cursor?limit=101",
            "/v1/user/cursor?cursor=not-a-cursor!",
        ] {
            assert_eq!(
                status_of(router.clone(), uri, Some(&token)).await,
                StatusCode::BAD_REQUEST,
                "{uri}"
            );
        }
    }

    /// 向注册接口发送请求体，返回状态码和响应 JSON
    async fn post_register(
        content_type: Option<&str>,
        body: &str,
    ) -> (StatusCode, serde_json::Value) {
        use axum::extract::ConnectInfo;
        use std::net::SocketAddr;

        let peer = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
        let mut request = Request::builder().method("POST").uri("/v1/user/register");
        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }

        let response = router(test_state(), false)
            .layer(Extension(peer))
            .oneshot(request.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();

        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    #[tokio::test]
    async fn malformed_request_bodies_get_json_errors() {
        // 不是合法 JSON
        let (status, json) = post_register(Some("application/json"), "{not json").await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["error"]["code"], 400);

        // 缺少字段
        let (status, json) = post_register(Some("application/json"), r#"{"username":"abc"}"#).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["error"]["code"], 422);

        // 缺少 Content-Type
        let (status, json) = post_register(None, "{}").await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(json["error"]["code"], 415);
    }

    #[tokio::test]
    async fn invalid_fields_are_reported_per_field() {
        let body = r#"{
            "username": "ab",
            "email": "not-an-email",
            "password": "short",
            "password_confirm": "different"
        }"#;
        let (status, json) = post_register(Some("application/json"), body).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let errors: Vec<(&str, &str)> = json["error"]["errors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|error| {
                (
                    error["location"].as_str().unwrap(),
                    error["reason"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            errors,
            vec![
                ("email", "INVALID_EMAIL"),
                ("password", "INVALID_LENGTH"),
                ("password_confirm", "PASSWORD_MISMATCH"),
                ("username", "INVALID_LENGTH"),
            ]
        );
    }

    #[tokio::test]
    async fn username_length_counts_characters() {
        // 三个汉字是 9 个字节，按字符数应通过长度校验；
        // 邮箱故意写错，让请求在访问数据库之前返回
        let body = r#"{
            "username": "张三丰",
            "email": "not-an-email",
            "password": "password123",
            "password_confirm": "password123"
        }"#;
        let (status, json) = post_register(Some("application/json"), body).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        let errors = json["error"]["errors"].as_array().unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["location"], "email");
    }

    #[tokio::test]
    async fn me_requires_a_valid_token() {
        let router = router(test_state(), false);

        assert_eq!(
            status_of(router.clone(), "/v1/user/me", None).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status_of(router, "/v1/user/me", Some("not-a-jwt")).await,
            StatusCode::UNAUTHORIZED
        );
    }
}
