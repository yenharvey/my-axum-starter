//! 用户管理模块
//!
//! 提供用户注册、登录、获取当前用户信息等功能。

use crate::AppState;
use aide::axum::ApiRouter;
use aide::axum::routing::{get_with, post_with};
use std::sync::Arc;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

pub mod dto;
mod handler;
mod service;

/// 构建用户模块的路由
///
/// 配置以下端点：
/// - POST /register - 用户注册（限速2req/s）
/// - POST /login - 用户登录（限速2req/s）
/// - GET /me - 获取当前用户信息（需要认证）
///
/// # 参数
/// * `state` - 应用状态，包含数据库和服务实例
///
/// # 返回
/// 返回配置好的路由器
pub fn routes(state: Arc<AppState>) -> ApiRouter {
    // 注册：严格限速（防暴力破解）
    // 每秒 2 个请求，初始突发 3 个
    let register_limiter = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(3)
        .use_headers()
        .finish()
        .unwrap();

    // 登录：严格限速（防暴力破解）
    let login_limiter = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(3)
        .use_headers()
        .finish()
        .unwrap();

    ApiRouter::new()
        .api_route(
            "/register",
            post_with(handler::register, handler::register_docs)
                .layer(GovernorLayer::new(register_limiter)),
        )
        .api_route(
            "/login",
            post_with(handler::login, handler::login_docs).layer(GovernorLayer::new(login_limiter)),
        )
        .api_route(
            "/me",
            get_with(handler::me, handler::me_docs).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                crate::core::middleware::auth::require_auth,
            )),
        )
        .with_state(state)
}
