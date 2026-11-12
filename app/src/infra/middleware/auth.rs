use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

use crate::AppState;
use crate::shared::error::AuthError;

/// 当前登录用户标识（由 [`require_auth`] 注入到请求扩展中）
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: i32,
}

/// 认证中间件：校验 `Authorization: Bearer <token>` 中的 JWT
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| {
            warn!("缺少 Authorization 请求头或格式不是 Bearer");
            AuthError::InvalidToken
        })?;

    let user_id = state.jwt_service.extract_user_id(token).map_err(|e| {
        warn!(error = %e, "访问令牌无效或已过期");
        AuthError::InvalidToken
    })?;

    request.extensions_mut().insert(CurrentUser { user_id });

    Ok(next.run(request).await)
}
