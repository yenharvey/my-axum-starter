//! 启动引导：在服务器开始监听之前对数据库做的一次性准备
//!
//! - 自动建库不能放进 migration：SeaORM migration 运行前必须先连上目标数据库
//! - 种子数据不属于表结构变更，也不放进 migration

mod database;
mod seed;

#[doc(hidden)]
pub use database::database_create_error_message;
pub use database::ensure_database_exists;
pub use seed::seed_demo_data;
