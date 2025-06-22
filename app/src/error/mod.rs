mod config;
mod file_upload;

use aide::OperationOutput;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use thiserror::Error;

use crate::ApiResponse;
pub use config::*;
pub use file_upload::FileUploadError;

/// 应用程序错误枚举
/// 
/// 统一处理应用程序中可能出现的各种错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] EnvConfigError),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("HTTP request error: {status}")]
    Http { status: StatusCode },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("General error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("File handle error: {0}")]
    FileHandle(#[from] FileUploadError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error".to_string()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error".to_string()),
            AppError::Serde(_) => (StatusCode::BAD_REQUEST, "Invalid data format".to_string()),
            AppError::Anyhow(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            AppError::FileHandle(_) => (StatusCode::BAD_REQUEST, "File upload error".to_string()),
            AppError::Http { status } => (status, "Request error".to_string()),
        };

        let response = ApiResponse::<()>::error(status.as_u16(), message);
        (status, Json(response)).into_response()
    }
}

impl From<axum::extract::multipart::MultipartError> for AppError {
    fn from(err: axum::extract::multipart::MultipartError) -> Self {
        AppError::FileHandle(FileUploadError::Multipart(err))
    }
}

impl OperationOutput for AppError {
    type Inner = ();
}