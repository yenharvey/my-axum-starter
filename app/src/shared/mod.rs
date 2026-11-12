//! 跨业务模块共享的类型与工具
//!
//! - [`error`]：统一错误类型
//! - [`response`]：统一 API 响应格式
//! - [`jwt`]：JWT 令牌生成和验证
//! - [`password`]：密码哈希和验证（Argon2）
//! - [`FromState`]：从应用状态构造服务的 Trait

pub mod error;
mod from_state;
pub mod jwt;
pub mod password;
pub mod response;

pub use from_state::FromState;
