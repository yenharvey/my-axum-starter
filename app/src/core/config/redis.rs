use std::env;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::section::ConfigSection;

/// Redis 连接配置
///
/// 包含 Redis 服务器连接信息。Redis 是可选的，如果未配置 URL 则不会初始化 Redis。
/// 连接池参数采用 deadpool-redis 的默认配置。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RedisConfig {
    /// Redis 服务器 URL（可选，格式: redis://[:password]@host:port/db）
    pub url: Option<String>,
}

impl ConfigSection for RedisConfig {
    fn section_name(&self) -> &str {
        "redis"
    }

    fn load_from_value(&mut self, value: &Value) -> Result<(), String> {
        if let Some(obj) = value.as_object()
            && let Some(url) = obj.get("url").and_then(|v| v.as_str())
        {
            self.url = Some(url.to_string());
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        // Redis 是可选的，无需额外验证
        Ok(())
    }

    fn apply_env_overrides(&mut self) -> Result<(), String> {
        if let Ok(url) = env::var("REDIS_URL") {
            self.url = Some(url);
        }
        Ok(())
    }
}
