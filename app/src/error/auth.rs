//! 认证相关错误

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::{ApiError, ApiResponse, Domain, ErrorDetail, Reason};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("用户已存在")]
    UserAlreadyExists,

    #[error("用户不存在")]
    UserNotFound,

    #[error("密码错误")]
    InvalidPassword,

    #[error("用户名格式无效")]
    InvalidUsername,

    #[error("密码长度至少8个字符")]
    PasswordTooShort,

    #[error("两次输入的密码不一致")]
    PasswordMismatch,

    #[error("用户已被停用")]
    UserInactive,

    #[error("无效的访问令牌")]
    InvalidToken,

    #[error("内部错误: {0}")]
    Internal(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let api_error = match self {
            Self::UserAlreadyExists => ApiError::new(StatusCode::CONFLICT, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, Reason::AlreadyExists)),

            Self::UserNotFound => ApiError::new(StatusCode::NOT_FOUND, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, Reason::UserNotFound)),

            Self::InvalidPassword => ApiError::new(StatusCode::UNAUTHORIZED, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, Reason::InvalidPassword)),

            Self::InvalidUsername => ApiError::new(StatusCode::BAD_REQUEST, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, Reason::InvalidUsername)),

            Self::PasswordTooShort => ApiError::new(StatusCode::BAD_REQUEST, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, Reason::WeakPassword)),

            Self::PasswordMismatch => ApiError::new(StatusCode::BAD_REQUEST, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, Reason::PasswordMismatch)),

            Self::UserInactive => ApiError::new(StatusCode::FORBIDDEN, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, Reason::AuthenticationFailed)),

            Self::InvalidToken => ApiError::new(StatusCode::UNAUTHORIZED, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, Reason::InvalidToken)),

            Self::Internal(ref msg) => {
                tracing::error!(error = %msg, "auth internal error");
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        ApiResponse::error(api_error).into_response()
    }
}
