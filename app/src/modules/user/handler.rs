use crate::{ApiResponse, AppError, AppState, core::middleware::CurrentUser, shared::FromState};
use aide::transform::TransformOperation;
use axum::Json;
use axum::extract::{Extension, State};
use std::sync::Arc;
use tracing::{info, instrument};

use super::dto::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use super::service::UserService;

/// 用户注册处理器
///
/// 处理用户注册请求，验证输入数据、哈希密码并创建新用户。
///
/// # 参数
/// * `state` - 应用状态（包含数据库连接）
/// * `req` - 注册请求数据（用户名、邮箱、密码）
///
/// # 返回
/// 成功返回注册用户信息（ID、用户名、邮箱），失败返回错误
#[instrument(skip(state))]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<ApiResponse<RegisterResponse>, AppError> {
    info!("处理用户注册请求: {}", req.username);

    let user_service = UserService::from_state(&state);
    let response = user_service.register(req).await?;

    info!("用户注册成功: {}", response.username);
    Ok(ApiResponse::success(response))
}

/// 用户注册 API 文档
pub fn register_docs(op: TransformOperation) -> TransformOperation {
    op.description("用户注册")
        .tag("认证")
        .response::<201, ApiResponse<RegisterResponse>>()
}

/// 用户登录处理器
///
/// 处理用户登录请求，验证用户名/邮箱和密码，生成 JWT 令牌。
///
/// # 参数
/// * `state` - 应用状态（包含数据库连接和 JWT 服务）
/// * `req` - 登录请求数据（用户名/邮箱、密码）
///
/// # 返回
/// 成功返回用户信息和 JWT 令牌（7天过期），失败返回错误
#[instrument(skip(state))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<ApiResponse<LoginResponse>, AppError> {
    info!("处理用户登录请求: {}", req.username_or_email);

    let user_service = UserService::from_state(&state);
    let response = user_service.login(req).await?;

    info!("用户登录成功: {}", response.username);
    Ok(ApiResponse::success(response))
}

/// 用户登录 API 文档
pub fn login_docs(op: TransformOperation) -> TransformOperation {
    op.description("用户登录")
        .tag("认证")
        .response::<200, ApiResponse<LoginResponse>>()
}

/// 获取当前用户处理器
///
/// 获取当前登录用户的信息。需要在 Authorization header 中提供有效的 JWT 令牌。
///
/// # 参数
/// * `state` - 应用状态（包含数据库连接）
/// * `current_user` - 当前登录用户（由认证中间件注入）
///
/// # 返回
/// 返回当前用户信息（ID、用户名、邮箱），如果用户不存在返回错误
#[instrument(skip(state, current_user))]
pub async fn me(
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<ApiResponse<RegisterResponse>, AppError> {
    info!("获取当前用户信息，用户ID: {}", current_user.user_id);

    let user_service = UserService::from_state(&state);
    let response = user_service.get_user(current_user.user_id).await?;

    Ok(ApiResponse::success(response))
}

/// 获取当前用户 API 文档
pub fn me_docs(op: TransformOperation) -> TransformOperation {
    op.description("获取当前登录用户信息")
        .tag("用户")
        .response::<200, ApiResponse<RegisterResponse>>()
}
