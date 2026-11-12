//! 应用库入口
//!
//! 顶层只有四个模块：
//! - [`infra`]：基础设施（配置、日志、中间件、应用状态、服务器启动），与业务无关
//! - [`shared`]：跨业务模块共享的类型与工具（错误、响应格式、JWT、密码哈希）
//! - [`modules`]：业务模块，每个业务域一个子模块
//! - [`routes`]：路由装配

pub mod infra;
pub mod modules;
pub mod routes;
pub mod shared;

pub use infra::config::AppConfig;
pub use infra::state::AppState;
pub use shared::error::AppError;
pub use shared::response::ApiResponse;
