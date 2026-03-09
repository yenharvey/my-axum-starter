//! 文件上传相关错误

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::{ApiError, ApiResponse, Domain, ErrorDetail, Reason};

#[derive(Debug, Error)]
pub enum FileUploadError {
    #[error("文件解析错误: {0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),

    #[error("文件大小超出限制: {0} 字节")]
    TooLarge(usize),

    #[error("文件类型不允许: {0}")]
    TypeNotAllowed(String),

    #[error("上传失败: {0}")]
    Failed(String),

    #[error("缺少必需字段: {0}")]
    MissingField(String),
}

impl IntoResponse for FileUploadError {
    fn into_response(self) -> Response {
        let api_error = match self {
            Self::Multipart(_) => ApiError::new(StatusCode::BAD_REQUEST, self.to_string())
                .with_detail(ErrorDetail::new(Domain::FILE, Reason::InvalidFormat)),

            Self::TooLarge(_) => ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, self.to_string())
                .with_detail(ErrorDetail::new(Domain::FILE, Reason::FileTooLarge)),

            Self::TypeNotAllowed(_) => {
                ApiError::new(StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string())
                    .with_detail(ErrorDetail::new(Domain::FILE, Reason::FileTypeNotAllowed))
            }

            Self::Failed(ref msg) => {
                tracing::error!(error = %msg, "file upload failed");
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }

            Self::MissingField(_) => ApiError::new(StatusCode::BAD_REQUEST, self.to_string())
                .with_detail(ErrorDetail::new(Domain::FILE, Reason::RequiredFieldMissing)),
        };
        ApiResponse::error(api_error).into_response()
    }
}
