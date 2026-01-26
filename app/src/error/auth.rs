use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::ApiResponse;

/// 认证相关错误
#[derive(Debug, Error)]
pub enum AuthError {
    /// 用户已存在
    #[error("用户已存在")]
    UserAlreadyExists,

    /// 用户不存在
    #[error("用户不存在")]
    UserNotFound,

    /// 密码错误
    #[error("密码错误")]
    InvalidPassword,

    /// 用户名格式无效
    #[error("用户名格式无效")]
    InvalidUsername,

    /// 密码长度至少8个字符
    #[error("密码长度至少8个字符")]
    PasswordTooShort,

    /// 两次输入的密码不一致
    #[error("两次输入的密码不一致")]
    PasswordMismatch,

    /// 用户被停用
    #[error("用户已被停用")]
    UserInactive,

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),
}

impl AuthError {
    /// 获取错误码
    #[allow(dead_code)]
    fn error_code(&self) -> u32 {
        match self {
            AuthError::UserAlreadyExists => 11201,
            AuthError::UserNotFound => 11202,
            AuthError::InvalidPassword => 11203,
            AuthError::InvalidUsername => 11204,
            AuthError::PasswordTooShort => 11206,
            AuthError::PasswordMismatch => 11207,
            AuthError::UserInactive => 11205,
            AuthError::Internal(_) => 11299,
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match self {
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, 11201, "用户已存在".to_string()),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, 11202, "用户不存在".to_string()),
            AuthError::InvalidPassword => (StatusCode::UNAUTHORIZED, 11203, "密码错误".to_string()),
            AuthError::InvalidUsername => {
                (StatusCode::BAD_REQUEST, 11204, "用户名格式无效".to_string())
            }
            AuthError::PasswordTooShort => (
                StatusCode::BAD_REQUEST,
                11206,
                "密码长度至少8个字符".to_string(),
            ),
            AuthError::PasswordMismatch => (
                StatusCode::BAD_REQUEST,
                11207,
                "两次输入的密码不一致".to_string(),
            ),
            AuthError::UserInactive => (StatusCode::FORBIDDEN, 11205, "用户已被停用".to_string()),
            AuthError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, 11299, e),
        };

        let response = ApiResponse::<()>::new(code, msg, None);
        (status, Json(response)).into_response()
    }
}
