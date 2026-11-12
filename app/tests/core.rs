//! `app` crate 的核心能力集成测试。
//!
//! 测试放在 `app/tests`，避免生产模块携带测试专用的模块声明。
//! 每个子模块对应一个被测核心能力。

#[path = "core/database_bootstrap.rs"]
mod database_bootstrap;
#[path = "core/pagination.rs"]
mod pagination;
