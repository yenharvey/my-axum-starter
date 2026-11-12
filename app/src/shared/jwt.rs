use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: i32,

    /// 过期时间（Unix timestamp）
    pub exp: i64,

    /// 签发时间（Unix timestamp）
    pub iat: i64,
}

/// JWT 服务：签发和校验访问令牌（HS256）
#[derive(Clone, Debug)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    ttl_secs: i64,
}

impl JwtService {
    /// 创建 JWT 服务
    ///
    /// # 参数
    /// * `secret` - 签名密钥
    /// * `ttl_secs` - 令牌有效期（秒）
    pub fn new(secret: &str, ttl_secs: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            ttl_secs: i64::try_from(ttl_secs).unwrap_or(i64::MAX),
        }
    }

    /// 令牌有效期（秒）
    pub fn ttl_secs(&self) -> i64 {
        self.ttl_secs
    }

    /// 为用户签发令牌
    pub fn generate_token(&self, user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id,
            exp: now.saturating_add(self.ttl_secs),
            iat: now,
        };
        encode(&Header::default(), &claims, &self.encoding_key)
    }

    /// 校验并解析令牌（签名无效或已过期时返回错误）
    pub fn verify_token(
        &self,
        token: &str,
    ) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
    }

    /// 从令牌中提取用户 ID
    pub fn extract_user_id(&self, token: &str) -> Result<i32, jsonwebtoken::errors::Error> {
        self.verify_token(token).map(|data| data.claims.sub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-at-least-32-characters-long";

    #[test]
    fn token_round_trip() {
        let jwt = JwtService::new(SECRET, 3600);
        let token = jwt.generate_token(42).unwrap();

        assert_eq!(jwt.extract_user_id(&token).unwrap(), 42);
    }

    #[test]
    fn token_signed_with_another_secret_is_rejected() {
        let token = JwtService::new(SECRET, 3600).generate_token(42).unwrap();
        let other = JwtService::new("another-secret-at-least-32-characters", 3600);

        assert!(other.extract_user_id(&token).is_err());
    }
}
