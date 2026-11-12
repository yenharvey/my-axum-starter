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
