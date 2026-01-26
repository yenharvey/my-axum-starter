use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

use super::section::ConfigSection;

/// 敏感信息配置
///
/// 包含应用的敏感信息，如密钥和令牌，应妥善保管。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SecretsConfig {
    /// JWT 签名密钥（必需，至少 32 字符）
    pub jwt_secret: String,

    /// Redis 连接 URL（可选）
    pub redis_url: Option<String>,
}

impl ConfigSection for SecretsConfig {
    fn section_name(&self) -> &str {
        "secrets"
    }

    fn load_from_value(&mut self, value: &Value) -> Result<(), String> {
        if let Some(obj) = value.as_object() {
            if let Some(secret) = obj.get("jwt_secret").and_then(|v| v.as_str()) {
                self.jwt_secret = secret.to_string();
            }
            if let Some(redis) = obj.get("redis_url").and_then(|v| v.as_str()) {
                self.redis_url = Some(redis.to_string());
            }
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.jwt_secret.is_empty() {
            return Err("JWT 密钥是必需的，但未提供".to_string());
        }
        if self.jwt_secret.len() < 32 {
            return Err("JWT 密钥长度必须至少 32 个字符".to_string());
        }
        Ok(())
    }

    fn required_env_vars(&self) -> Vec<&str> {
        vec!["JWT_SECRET"]
    }

    fn apply_env_overrides(&mut self) -> Result<(), String> {
        if let Ok(secret) = env::var("JWT_SECRET") {
            self.jwt_secret = secret;
        }
        if let Ok(redis) = env::var("REDIS_URL") {
            self.redis_url = Some(redis);
        }
        Ok(())
    }
}
