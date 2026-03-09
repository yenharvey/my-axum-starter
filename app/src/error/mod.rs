//! 错误处理模块
//!
//! 所有错误类型统一转换为 Google JSON Style Guide 格式的响应。

mod auth;
mod config;
mod file_upload;
mod redis;
mod validation;

use aide::OperationOutput;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::{ApiError, ApiResponse};

pub use auth::AuthError;
pub use config::ConfigError;
pub use file_upload::FileUploadError;
pub use redis::RedisError;
pub use validation::ValidationError;

/// 应用程序错误
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    FileUpload(#[from] FileUploadError),

    #[error(transparent)]
    Redis(#[from] RedisError),

    #[error("数据库错误: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            // 委托给具体错误类型
            Self::Auth(e) => e.into_response(),
            Self::Validation(e) => e.into_response(),
            Self::Config(e) => e.into_response(),
            Self::FileUpload(e) => e.into_response(),
            Self::Redis(e) => e.into_response(),

            Self::Database(e) => {
                tracing::error!(error = %e, "database error");
                ApiResponse::error(ApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                ))
                .into_response()
            }

            Self::Io(e) => {
                tracing::error!(error = %e, "io error");
                ApiResponse::error(ApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                ))
                .into_response()
            }

            Self::Serde(e) => {
                tracing::error!(error = %e, "serialization error");
                ApiResponse::error(ApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                ))
                .into_response()
            }

            Self::Anyhow(e) => {
                tracing::error!(error = %e, "internal error");
                ApiResponse::error(ApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                ))
                .into_response()
            }
        }
    }
}

impl From<axum::extract::multipart::MultipartError> for AppError {
    fn from(e: axum::extract::multipart::MultipartError) -> Self {
        Self::FileUpload(FileUploadError::Multipart(e))
    }
}

impl From<deadpool_redis::CreatePoolError> for AppError {
    fn from(e: deadpool_redis::CreatePoolError) -> Self {
        Self::Redis(RedisError::Pool(e))
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self {
        Self::Validation(ValidationError::from_validator(e))
    }
}

impl OperationOutput for AppError {
    type Inner = ();
}
