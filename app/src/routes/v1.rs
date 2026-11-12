//! V1 版本 API 路由
//!
//! 包含 V1 版本所有的 API 端点。

use crate::{AppState, modules::user};
use aide::axum::ApiRouter;
use std::sync::Arc;

/// 构建 V1 版本的 API 路由
///
/// 聚合所有 V1 版本的业务模块路由。目前包括：
/// - /user - 用户管理相关的端点
///
/// # 参数
/// * `state` - 应用状态，包含数据库连接等资源
///
/// # 返回
/// 返回配置好的 V1 API 路由器
pub fn routes(state: Arc<AppState>) -> ApiRouter {
    ApiRouter::new()
        .nest_api_service("/user", user::routes(state.clone()))
        .with_state(state)
}
