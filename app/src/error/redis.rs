//! Redis 相关错误

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::{ApiError, ApiResponse};

#[derive(Debug, Error)]
pub enum RedisError {
    #[error("Redis 连接池创建失败: {0}")]
    Pool(#[from] deadpool_redis::CreatePoolError),

    #[error("Redis 连接错误: {0}")]
    Connection(String),

    #[error("Redis 操作错误: {0}")]
    Operation(String),
}

impl IntoResponse for RedisError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "redis error");
        ApiResponse::error(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        ))
        .into_response()
    }
}
