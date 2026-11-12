//! API 文档（OpenAPI + Scalar）

use std::sync::Arc;

use aide::{
    axum::{
        ApiRouter, IntoApiResponse,
        routing::{get, get_with},
    },
    openapi::OpenApi,
    scalar::Scalar,
    transform::TransformOpenApi,
};
use axum::{Extension, Json, response::IntoResponse};

/// 构建 API 文档路由（挂载在 `/docs` 下）
pub fn routes() -> ApiRouter {
    // 文档页面的响应类型由 aide 推断（text/html, 200）
    aide::generate::infer_responses(true);

    let router: ApiRouter = ApiRouter::new()
        .api_route_with(
            "/",
            get_with(
                Scalar::new("/docs/private/api.json")
                    .with_title("API Docs")
                    .axum_handler(),
                |op| op.description("This documentation page."),
            ),
            |p| p.security_requirement("BearerAuth"),
        )
        .route("/private/api.json", get(serve_docs));

    // 其他路由的响应类型需显式声明，推断结果可能不准确
    aide::generate::infer_responses(false);

    router
}

/// 配置 OpenAPI 文档的全局信息
pub fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("API Documentation")
        .summary("REST API")
        .security_scheme(
            "BearerAuth",
            aide::openapi::SecurityScheme::Http {
                scheme: "bearer".into(),
                bearer_format: Some("JWT".into()),
                description: Some(
                    "登录后获取的 JWT，通过 `Authorization: Bearer <token>` 传递".into(),
                ),
                extensions: Default::default(),
            },
        )
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api).into_response()
}
