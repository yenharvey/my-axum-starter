//! 系统端点：健康检查、欢迎页、favicon

use aide::axum::IntoApiResponse;
use axum::http::header::CONTENT_TYPE;
use serde_json::{Value, json};
use tracing::{info, instrument};

use crate::ApiResponse;

/// 健康检查端点
///
/// 返回服务器是否正常运行的状态。
#[instrument]
pub async fn health_check() -> ApiResponse<Value> {
    info!("健康检查请求");
    ApiResponse::success(json!({
        "status": "healthy"
    }))
}

/// Hello World 测试端点
///
/// 返回一条简单的问候消息，用于测试服务器是否正常响应。
#[instrument]
pub async fn hello_world() -> ApiResponse<Value> {
    info!("Hello World 请求");
    ApiResponse::success(json!({
        "message": "Hello, World!"
    }))
}

/// 获取网站图标
pub async fn favicon() -> impl IntoApiResponse {
    let favicon = include_bytes!("../../assets/favicon.png");
    ([(CONTENT_TYPE, "image/png")], favicon.as_ref())
}
