//! 认证相关错误

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::shared::response::{ApiError, ApiResponse, Domain, ErrorDetail, Reason};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("用户名或邮箱已被注册")]
    UserAlreadyExists,

    #[error("用户不存在")]
    UserNotFound,

    /// 登录失败：不区分“用户不存在”和“密码错误”，避免泄露账号是否存在
    #[error("用户名或密码错误")]
    InvalidCredentials,

    #[error("用户已被停用")]
    UserInactive,

    #[error("无效的访问令牌")]
    InvalidToken,

    #[error("内部错误: {0}")]
    Internal(String),
}

impl AuthError {
    /// 对应的 HTTP 状态码和错误原因；内部错误没有对外的原因
    fn status_and_reason(&self) -> (StatusCode, Option<Reason>) {
        match self {
            Self::UserAlreadyExists => (StatusCode::CONFLICT, Some(Reason::AlreadyExists)),
            Self::UserNotFound => (StatusCode::NOT_FOUND, Some(Reason::UserNotFound)),
            Self::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, Some(Reason::AuthenticationFailed))
            }
            Self::UserInactive => (StatusCode::FORBIDDEN, Some(Reason::PermissionDenied)),
            Self::InvalidToken => (StatusCode::UNAUTHORIZED, Some(Reason::InvalidToken)),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, None),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let api_error = match self.status_and_reason() {
            (status, Some(reason)) => ApiError::new(status, self.to_string())
                .with_detail(ErrorDetail::new(Domain::AUTH, reason)),
            (status, None) => {
                // 细节只进日志，不返回给客户端
                tracing::error!(error = %self, "auth internal error");
                ApiError::new(status, "Internal server error")
            }
        };
        ApiResponse::error(api_error).into_response()
    }
}
