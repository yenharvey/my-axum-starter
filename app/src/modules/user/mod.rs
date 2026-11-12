//! 用户管理模块
//!
//! 提供用户注册、登录、获取当前用户信息等功能。

pub mod dto;
mod handler;
mod service;

use std::sync::Arc;

use aide::axum::ApiRouter;
use aide::axum::routing::{get_with, post_with};

use crate::AppState;
use crate::infra::middleware::{rate_limit_layer, require_auth};

/// 注册 / 登录接口的限流（防暴力破解）：每个 IP 每秒补充的令牌数
const AUTH_RATE_LIMIT_PER_SECOND: u64 = 2;
/// 注册 / 登录接口的限流：每个 IP 允许的突发请求数
const AUTH_RATE_LIMIT_BURST: u32 = 3;

/// 构建用户模块的路由
///
/// - `GET /` - 分页获取用户列表（需要认证）
/// - `POST /register` - 用户注册（严格限流）
/// - `POST /login` - 用户登录（严格限流）
/// - `GET /me` - 获取当前用户信息（需要认证）
pub fn routes(state: Arc<AppState>) -> ApiRouter {
    // 每个接口各自一个限流器，互不占用配额
    let auth_rate_limit = || rate_limit_layer(AUTH_RATE_LIMIT_PER_SECOND, AUTH_RATE_LIMIT_BURST);

    let auth = || axum::middleware::from_fn_with_state(state.clone(), require_auth);

    ApiRouter::new()
        .api_route(
            "/",
            get_with(handler::list_users, handler::list_users_docs).layer(auth()),
        )
        .api_route(
            "/register",
            post_with(handler::register, handler::register_docs).layer(auth_rate_limit()),
        )
        .api_route(
            "/login",
            post_with(handler::login, handler::login_docs).layer(auth_rate_limit()),
        )
        .api_route("/me", get_with(handler::me, handler::me_docs).layer(auth()))
        .with_state(state)
}
