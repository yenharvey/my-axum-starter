//! 应用配置
//!
//! # 配置来源（优先级从低到高）
//!
//! 1. `config/default.toml` —— 所有配置项的默认值（编译进二进制，是默认值的唯一来源）
//! 2. `config/{APP_ENV}.toml` —— 环境专属配置（`APP_ENV` 默认为 `development`）
//! 3. `config/local.toml` —— 个人本地覆盖（不进版本库）
//! 4. 环境变量别名：`DATABASE_URL`、`REDIS_URL`、`JWT_SECRET`
//! 5. `APP_<SECTION>_<KEY>` 环境变量
//!
//! # 命名规则
//!
//! 每个配置项只有一个名字 `section.key`，对应的环境变量是 `APP_<SECTION>_<KEY>`
//! （只有第一个下划线是层级分隔符）。例如 `database.max_connections` 对应
//! `APP_DATABASE_MAX_CONNECTIONS`。列表类型用逗号分隔。

mod auth;
mod cors;
mod database;
mod logging;
mod redis;
mod server;

pub use auth::AuthConfig;
pub use cors::CorsConfig;
pub use database::DatabaseConfig;
pub use logging::{ConsoleFormat, LogLevel, LogRotation, LoggingConfig};
pub use redis::RedisConfig;
pub use server::ServerConfig;

use crate::shared::error::ConfigError;
use config::{Config, File, FileFormat};
use serde::Deserialize;

/// 默认配置（编译期嵌入，保证没有配置目录时也能启动）
const DEFAULT_CONFIG: &str = include_str!("../../../../config/default.toml");

/// 所有配置段的名字，同时也是 `APP_<SECTION>_<KEY>` 中合法的 SECTION
const SECTIONS: [&str; 6] = ["server", "database", "redis", "auth", "logging", "cors"];

/// 约定俗成的环境变量别名（部署平台通常直接注入这些名字）
const ENV_ALIASES: [(&str, &str); 3] = [
    ("DATABASE_URL", "database.url"),
    ("REDIS_URL", "redis.url"),
    ("JWT_SECRET", "auth.jwt_secret"),
];

/// 列表类型的配置项：环境变量的值按逗号拆分
const LIST_KEYS: [&str; 4] = [
    "cors.allow_origins",
    "cors.allow_methods",
    "cors.allow_headers",
    "cors.expose_headers",
];

/// 应用程序配置
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    /// 服务器配置
    pub server: ServerConfig,

    /// 数据库配置
    pub database: DatabaseConfig,

    /// Redis 配置
    pub redis: RedisConfig,

    /// 认证配置
    pub auth: AuthConfig,

    /// 日志配置
    pub logging: LoggingConfig,

    /// CORS 跨域资源共享配置
    pub cors: CorsConfig,
}

impl AppConfig {
    /// 从配置文件和环境变量加载并校验配置
    pub fn load() -> Result<Self, ConfigError> {
        // `.env` 只是把变量放进进程环境，不存在时忽略
        dotenvy::dotenv().ok();

        Self::from_env_vars(std::env::vars())
    }

    /// 用给定的环境变量集合构建配置（便于测试）
    fn from_env_vars(vars: impl Iterator<Item = (String, String)>) -> Result<Self, ConfigError> {
        let vars: Vec<(String, String)> = vars.collect();

        let env = vars
            .iter()
            .find(|(name, _)| name == "APP_ENV")
            .map(|(_, value)| value.as_str())
            .unwrap_or("development");

        let mut builder = Config::builder()
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{env}")).required(false))
            .add_source(File::with_name("config/local").required(false));

        for (key, value) in env_overrides(&vars)? {
            builder = if LIST_KEYS.contains(&key.as_str()) {
                let items: Vec<String> = value
                    .split(',')
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect();
                builder.set_override(key, items)
            } else {
                builder.set_override(key, value)
            }
            .map_err(|e| ConfigError::Invalid(format!("环境变量覆盖失败：{e}")))?;
        }

        let config: Self = builder
            .build()
            .and_then(Config::try_deserialize)
            .map_err(|e| ConfigError::Invalid(format!("配置加载失败：{e}")))?;

        config.validate()?;

        Ok(config)
    }

    /// 校验所有配置段
    fn validate(&self) -> Result<(), ConfigError> {
        let results = [
            ("server", self.server.validate()),
            ("database", self.database.validate()),
            ("redis", self.redis.validate()),
            ("auth", self.auth.validate()),
            ("logging", self.logging.validate()),
            ("cors", self.cors.validate()),
        ];

        for (section, result) in results {
            result.map_err(|e| ConfigError::Invalid(format!("[{section}] {e}")))?;
        }

        Ok(())
    }

    /// 服务器监听地址，格式为 `host:port`
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

/// 把环境变量转换为 `section.key` 覆盖项
///
/// 返回值按优先级从低到高排列：先是别名，后是 `APP_<SECTION>_<KEY>`。
///
/// `APP_` 前缀是本应用专用的：除 `APP_ENV` 外，不属于任何配置段的 `APP_*` 变量
/// 会被当作拼写错误而报错（键名是否存在由反序列化阶段检查）。值为空的变量视为未设置。
fn env_overrides(vars: &[(String, String)]) -> Result<Vec<(String, String)>, ConfigError> {
    let mut overrides = Vec::new();

    for (name, value) in vars {
        if value.is_empty() {
            continue;
        }
        if let Some((_, key)) = ENV_ALIASES.iter().find(|(alias, _)| alias == name) {
            overrides.push((key.to_string(), value.clone()));
        }
    }

    for (name, value) in vars {
        let Some(rest) = name.strip_prefix("APP_") else {
            continue;
        };
        if name == "APP_ENV" {
            continue;
        }

        let rest = rest.to_lowercase();
        let key = rest
            .split_once('_')
            .filter(|(section, key)| SECTIONS.contains(section) && !key.is_empty())
            .map(|(section, key)| format!("{section}.{key}"))
            .ok_or_else(|| {
                ConfigError::Invalid(format!(
                    "未知的环境变量 {name}：应为 APP_<SECTION>_<KEY>，SECTION 可选 {}",
                    SECTIONS.join(" / ")
                ))
            })?;

        if !value.is_empty() {
            overrides.push((key, value.clone()));
        }
    }

    Ok(overrides)
}

#[cfg(test)]
mod tests {
    use super::*;

    const JWT_SECRET: &str = "test-secret-at-least-32-characters-long";

    fn load(extra: &[(&str, &str)]) -> Result<AppConfig, ConfigError> {
        let required = [
            ("DATABASE_URL", "postgresql://localhost/test"),
            ("JWT_SECRET", JWT_SECRET),
        ];
        AppConfig::from_env_vars(
            required
                .iter()
                .chain(extra)
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    #[test]
    fn defaults_come_from_default_toml() {
        let config = load(&[]).unwrap();

        assert_eq!(config.server.port, 3000);
        assert!(!config.server.enable_docs);
        assert!(!config.database.auto_create);
        assert!(!config.database.seed_demo);
        assert_eq!(config.database.url, "postgresql://localhost/test");
        assert_eq!(config.auth.jwt_secret, JWT_SECRET);
        assert_eq!(config.auth.token_ttl_secs, 7 * 24 * 3600);
        assert_eq!(config.logging.level, LogLevel::Info);
        assert_eq!(config.redis.url, None);
    }

    #[test]
    fn required_secrets_are_enforced() {
        let missing = AppConfig::from_env_vars(std::iter::empty());
        assert!(missing.is_err());

        let short_secret = load(&[("JWT_SECRET", "too-short")]);
        assert!(short_secret.is_err());
    }

    #[test]
    fn app_prefixed_vars_override_any_key() {
        let config = load(&[
            ("APP_SERVER_PORT", "8080"),
            ("APP_SERVER_ENABLE_DOCS", "true"),
            ("APP_DATABASE_MAX_CONNECTIONS", "20"),
            ("APP_LOGGING_LEVEL", "debug"),
            ("APP_LOGGING_CONSOLE_FORMAT", "pretty"),
            ("APP_REDIS_URL", "redis://localhost:6379"),
            ("APP_ENV", "development"),
            ("APP_LOGGING_FILE_DIR", ""),
        ])
        .unwrap();

        assert_eq!(config.server.port, 8080);
        assert!(config.server.enable_docs);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.logging.level, LogLevel::Debug);
        assert_eq!(config.logging.console_format, ConsoleFormat::Pretty);
        assert_eq!(config.redis.url.as_deref(), Some("redis://localhost:6379"));
    }

    #[test]
    fn app_prefixed_vars_win_over_aliases() {
        let config = load(&[("APP_DATABASE_URL", "postgresql://localhost/override")]).unwrap();

        assert_eq!(config.database.url, "postgresql://localhost/override");
    }

    #[test]
    fn list_values_are_comma_separated() {
        let config = load(&[(
            "APP_CORS_ALLOW_ORIGINS",
            "https://a.example.com, https://b.example.com",
        )])
        .unwrap();

        assert_eq!(
            config.cors.allow_origins,
            vec!["https://a.example.com", "https://b.example.com"]
        );
    }

    #[test]
    fn invalid_values_are_rejected() {
        assert!(load(&[("APP_LOGGING_LEVEL", "verbose")]).is_err());
        assert!(load(&[("APP_SERVER_PORT", "0")]).is_err());
        assert!(
            load(&[
                ("APP_CORS_ALLOW_ORIGINS", "*"),
                ("APP_CORS_ALLOW_CREDENTIALS", "true"),
            ])
            .is_err()
        );
    }

    #[test]
    fn misspelled_env_vars_are_rejected() {
        // 键名拼错
        assert!(load(&[("APP_SERVER_PROT", "8080")]).is_err());
        // 配置段不存在
        assert!(load(&[("APP_SEVER_PORT", "8080")]).is_err());
        assert!(load(&[("APP_PORT", "8080")]).is_err());
        // 即使值为空，名字也要合法
        assert!(load(&[("APP_UNKNOWN_KEY", "")]).is_err());
    }

    #[test]
    fn unknown_keys_in_config_files_are_rejected() {
        let parse = |toml: &str| {
            Config::builder()
                .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
                .add_source(File::from_str(toml, FileFormat::Toml))
                .build()
                .and_then(Config::try_deserialize::<AppConfig>)
        };

        assert!(
            parse(
                "[server]
port = 8080"
            )
            .is_ok()
        );
        assert!(
            parse(
                "[server]
prot = 8080"
            )
            .is_err()
        );
        assert!(
            parse(
                "[sever]
port = 8080"
            )
            .is_err()
        );
    }

    /// 字符串表必须与 default.toml（即配置结构体）保持一致
    #[test]
    fn key_tables_match_default_config() {
        let defaults = Config::builder()
            .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .build()
            .unwrap();

        let mut sections: Vec<String> = defaults
            .clone()
            .try_deserialize::<std::collections::HashMap<String, config::Value>>()
            .unwrap()
            .into_keys()
            .collect();
        sections.sort();
        let mut expected: Vec<String> = SECTIONS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(sections, expected);

        for key in LIST_KEYS {
            assert!(
                defaults.get_array(key).is_ok(),
                "{key} 不是列表类型的配置项"
            );
        }
    }
}
