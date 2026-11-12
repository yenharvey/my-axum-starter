use serde::Deserialize;
use std::fmt;

/// 数据库配置（`[database]`）
#[derive(Clone, Deserialize)]
pub struct DatabaseConfig {
    /// 连接 URL（必需，通常通过 `DATABASE_URL` 环境变量提供）
    #[serde(default)]
    pub url: String,

    /// 连接池最大连接数
    pub max_connections: u32,

    /// 建立连接 / 从连接池获取连接的超时时间（秒）
    pub connect_timeout_secs: u64,
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
            .finish()
    }
}
