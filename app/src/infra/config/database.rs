use serde::Deserialize;
use std::fmt;

/// 数据库配置（`[database]`）
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    /// 连接 URL（必需，通常通过 `DATABASE_URL` 环境变量提供）
    #[serde(default)]
    pub url: String,

    /// 连接池最大连接数
    pub max_connections: u32,

    /// 建立连接 / 从连接池获取连接的超时时间（秒）
    pub connect_timeout_secs: u64,

    /// 启动时目标数据库不存在则自动创建（仅 PostgreSQL）
    ///
    /// 需要当前账号能连接 `postgres` 管理库并拥有 CREATEDB 权限。
    pub auto_create: bool,

    /// 启动时创建演示用户（密码是公开的，只用于本地开发和演示）
    pub seed_demo: bool,
}

impl DatabaseConfig {
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("url 是必需的，请设置 DATABASE_URL 环境变量".to_string());
        }
        if self.max_connections == 0 {
            return Err("max_connections 必须大于 0".to_string());
        }
        if self.connect_timeout_secs == 0 {
            return Err("connect_timeout_secs 必须大于 0".to_string());
        }
        Ok(())
    }
}

// URL 中通常带有密码，不能出现在日志里
impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("url", &"<redacted>")
            .field("max_connections", &self.max_connections)
            .field("connect_timeout_secs", &self.connect_timeout_secs)
            .field("auto_create", &self.auto_create)
            .field("seed_demo", &self.seed_demo)
            .finish()
    }
}
