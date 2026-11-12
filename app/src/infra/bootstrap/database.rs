use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};

use crate::AppError;
use crate::shared::error::ConfigError;

/// 确保目标 PostgreSQL 数据库存在。
///
/// 对非 PostgreSQL URL 直接跳过；PostgreSQL URL 会先连接默认 `postgres`
/// 管理库，若目标库不存在则尝试创建。
pub async fn ensure_database_exists(database_url: &str) -> Result<(), AppError> {
    let parsed = url::Url::parse(database_url)
        .map_err(|error| ConfigError::Invalid(format!("数据库 URL 解析失败: {error}")))?;

    if parsed.scheme() != "postgres" && parsed.scheme() != "postgresql" {
        return Ok(());
    }

    let db_name = parsed.path().trim_start_matches('/');
    if db_name.is_empty() {
        return Ok(());
    }

    let mut admin_url = parsed.clone();
    admin_url.set_path("/postgres");

    // 错误信息里只写主机名：完整的连接串带有密码
    let admin_conn = Database::connect(admin_url.as_str())
        .await
        .map_err(|error| {
            ConfigError::Invalid(format!(
                "无法连接到 {} 上的 postgres 管理库: {error}。\
                 如果当前账号无权访问管理库，请关闭 database.auto_create 并提前手动创建数据库。",
                parsed.host_str().unwrap_or("数据库服务器")
            ))
        })?;

    let exists = admin_conn
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT 1 FROM pg_database WHERE datname = $1",
            [db_name.to_string().into()],
        ))
        .await?
        .is_some();

    if exists {
        tracing::info!("数据库 '{}' 已存在", db_name);
    } else {
        tracing::info!("数据库 '{}' 不存在，正在自动创建", db_name);
        admin_conn
            .execute_unprepared(&format!("CREATE DATABASE {}", quote_pg_identifier(db_name)))
            .await
            .map_err(|error| {
                ConfigError::Invalid(database_create_error_message(db_name, &error))
            })?;
        tracing::info!("数据库 '{}' 创建成功", db_name);
    }

    admin_conn.close().await?;
    Ok(())
}

fn quote_pg_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

pub fn database_create_error_message(db_name: &str, error: &impl std::fmt::Display) -> String {
    format!(
        "创建数据库 '{db_name}' 失败: {error}。请确认当前 PostgreSQL 用户拥有 CREATEDB 权限，或提前手动创建数据库后再启动应用。"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_are_quoted() {
        assert_eq!(quote_pg_identifier("app_db"), "\"app_db\"");
        assert_eq!(quote_pg_identifier("a\"b"), "\"a\"\"b\"");
    }
}
