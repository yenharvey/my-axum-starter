//! 密码哈希和验证（Argon2id）
//!
//! Argon2 是 CPU 密集型计算（单次几十毫秒），因此对外接口都在阻塞线程池中执行，
//! 避免卡住异步运行时。

use argon2::password_hash::{Error as HashError, phc::PasswordHash};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use thiserror::Error;

/// 密码哈希错误
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("密码哈希失败：{0}")]
    Hash(String),

    #[error("无效的哈希格式：{0}")]
    InvalidHash(String),

    #[error("密码哈希任务异常终止：{0}")]
    Task(#[from] tokio::task::JoinError),
}

/// 对密码进行哈希，返回 PHC 格式的字符串（含算法参数和随机盐）
pub async fn hash_password(password: String) -> Result<String, PasswordError> {
    tokio::task::spawn_blocking(move || hash_blocking(&password)).await?
}

/// 验证密码是否与哈希匹配
pub async fn verify_password(
    password: String,
    password_hash: String,
) -> Result<bool, PasswordError> {
    tokio::task::spawn_blocking(move || verify_blocking(&password, &password_hash)).await?
}

fn hash_blocking(password: &str) -> Result<String, PasswordError> {
    // 盐由操作系统随机数生成器自动生成
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::Hash(e.to_string()))
}

fn verify_blocking(password: &str, password_hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| PasswordError::InvalidHash(e.to_string()))?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(HashError::PasswordInvalid) => Ok(false),
        Err(e) => Err(PasswordError::Hash(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hash_and_verify() {
        let hash = hash_password("correct horse".to_string()).await.unwrap();

        assert!(
            verify_password("correct horse".to_string(), hash.clone())
                .await
                .unwrap()
        );
        assert!(
            !verify_password("wrong password".to_string(), hash)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn malformed_hash_is_an_error() {
        let result = verify_password("password".to_string(), "not-a-hash".to_string()).await;

        assert!(matches!(result, Err(PasswordError::InvalidHash(_))));
    }
}
