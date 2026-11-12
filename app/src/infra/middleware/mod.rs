//! HTTP 中间件：认证、请求 ID、CORS、限流

/// JWT 认证
mod auth;
/// CORS 跨域层构建
mod cors;
/// 基于 IP 的速率限制
mod rate_limit;
/// 请求 ID 生成和追踪
mod request_id;

pub use auth::{CurrentUser, require_auth};
pub use cors::build_cors_layer;
pub use rate_limit::rate_limit_layer;
pub use request_id::request_id_middleware;
