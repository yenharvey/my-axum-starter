use serde::Deserialize;
use std::fmt;

/// JWT 密钥的最小长度
const MIN_JWT_SECRET_LEN: usize = 32;

/// 认证配置（`[auth]`）
#[derive(Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT 签名密钥（必需，至少 32 个字符，通常通过 `JWT_SECRET` 环境变量提供）
    #[serde(default)]
    pub jwt_secret: String,

    /// 访问令牌有效期（秒）
    pub token_ttl_secs: u64,
}

impl AuthConfig {
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.jwt_secret.is_empty() {
            return Err("jwt_secret 是必需的，请设置 JWT_SECRET 环境变量".to_string());
        }
        if self.jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(format!(
                "jwt_secret 长度必须至少 {MIN_JWT_SECRET_LEN} 个字符"
            ));
        }
        if self.token_ttl_secs == 0 || i64::try_from(self.token_ttl_secs).is_err() {
            return Err("token_ttl_secs 超出有效范围".to_string());
        }
        Ok(())
    }
}

// 密钥不能出现在日志里
impl fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthConfig")
            .field("jwt_secret", &"<redacted>")
            .field("token_ttl_secs", &self.token_ttl_secs)
            .finish()
    }
}
