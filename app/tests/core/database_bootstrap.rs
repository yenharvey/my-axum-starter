//! 数据库启动前置检查的错误提示测试。
//!
//! 真实建库路径依赖 PostgreSQL 实例和账号权限；这里固定 `CREATE DATABASE`
//! 失败时对用户展示的处理建议，避免权限问题退化成模糊的内部错误。

use app::infra::bootstrap::database_create_error_message;

#[test]
fn create_database_error_mentions_createdb_permission() {
    let message = database_create_error_message("app_db", &"permission denied to create database");

    assert!(message.contains("app_db"));
    assert!(message.contains("CREATEDB 权限"));
    assert!(message.contains("提前手动创建数据库"));
}
