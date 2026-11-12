//! 跨业务模块共享的类型与工具
//!
//! - [`error`]：统一错误类型
//! - [`response`]：统一 API 响应格式
//! - [`ApiJson`]：带校验的 JSON 请求体提取器
//! - [`jwt`]：JWT 令牌生成和验证
//! - [`pagination`]：列表接口的分页（页码分页与游标分页）
//! - [`password`]：密码哈希和验证（Argon2）
//! - [`FromState`]：从应用状态构造服务的 Trait

pub mod error;
mod from_state;
mod json;
pub mod jwt;
pub mod pagination;
pub mod password;
pub mod response;

pub use from_state::FromState;
pub use json::ApiJson;
