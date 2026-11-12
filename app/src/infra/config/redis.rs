use serde::Deserialize;
use std::fmt;

/// Redis 配置（`[redis]`）
///
/// Redis 是可选的：未配置 `url` 时不会初始化连接池。
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedisConfig {
    /// 连接 URL（可选，通常通过 `REDIS_URL` 环境变量提供）
    ///
    /// 格式：`redis://[:password@]host:port[/db]`
    #[serde(default)]
    pub url: Option<String>,
}

impl RedisConfig {
    pub(super) fn validate(&self) -> Result<(), String> {
        match self.url.as_deref() {
            Some(url) if !url.starts_with("redis://") && !url.starts_with("rediss://") => {
                Err("url 必须以 redis:// 或 rediss:// 开头".to_string())
            }
            _ => Ok(()),
        }
    }
}

// URL 中可能带有密码，不能出现在日志里
impl fmt::Debug for RedisConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisConfig")
            .field("url", &self.url.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}
