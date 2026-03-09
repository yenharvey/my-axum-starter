//! 验证相关错误

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::{ApiError, ApiResponse, Domain, ErrorDetail, Reason};

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{0}")]
    Fields(String),

    #[error("{0}")]
    Custom(String),
}

impl ValidationError {
    /// 从 validator::ValidationErrors 创建
    pub fn from_validator(errors: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .map(|(field, errs)| {
                let msgs = errs
                    .iter()
                    .filter_map(|e| e.message.as_ref())
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "{}: {}",
                    field,
                    if msgs.is_empty() {
                        "验证失败"
                    } else {
                        &msgs
                    }
                )
            })
            .collect();
        Self::Fields(messages.join("; "))
    }

    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let api_error = ApiError::new(StatusCode::BAD_REQUEST, self.to_string()).with_detail(
            ErrorDetail::with_message(Domain::VALIDATION, Reason::InvalidFormat, self.to_string()),
        );
        ApiResponse::error(api_error).into_response()
    }
}
