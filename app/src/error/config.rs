//! 配置相关错误

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::{ApiError, ApiResponse};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("缺少必需的环境变量: {0}")]
    MissingVar(String),

    #[error("环境变量 {var} 的值无效: {value}")]
    InvalidValue { var: String, value: String },

    #[error("配置错误: {0}")]
    Invalid(String),

    #[error("解析错误: {0}")]
    Parse(String),
}

impl IntoResponse for ConfigError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "config error");
        ApiResponse::error(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        ))
        .into_response()
    }
}

impl From<std::env::VarError> for ConfigError {
    fn from(e: std::env::VarError) -> Self {
        Self::MissingVar(e.to_string())
    }
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::Parse(e.to_string())
    }
}
