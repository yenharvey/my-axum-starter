use crate::{auth::service::AuthService, shared::FromState, ApiResponse, AppError, AppState};
use aide::transform::TransformOperation;
use axum::extract::State;
use axum::Json;
use std::sync::Arc;
use tracing::{info, instrument};

#[instrument(skip(state))]
pub async fn register_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<String>,
) -> Result<ApiResponse<String>, AppError> {
    info!("用户注册请求: {}", req);

    let auth_service = AuthService::from_state(&*state);

    let user = auth_service.register_user(&req).await?;

    info!("用户创建成功!");
    Ok(ApiResponse::success(user))
}

pub fn register_user_docs(op: TransformOperation) -> TransformOperation {
    op.description("创建一个新的用户")
        .tag("认证")
        .response::<201, ApiResponse<String>>()
}
