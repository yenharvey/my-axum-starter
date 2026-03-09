mod cors;
mod database;
mod logging;
mod redis;
mod secrets;
mod section;
mod server;

pub use cors::CorsConfig;
pub use database::DatabaseConfig;
pub use logging::LoggingConfig;
pub use redis::RedisConfig;
pub use secrets::SecretsConfig;
pub use section::ConfigSection;
pub use server::ServerConfig;

use crate::error::ConfigError;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

/// 应用程序配置入口
///
/// 聚合所有配置段（服务器、数据库、日志、敏感信息、跨域、Redis）。
/// 通过 `load()` 方法从配置文件和环境变量加载配置，支持多层次优先级管理。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct AppConfig {
    /// 服务器配置
    pub server: ServerConfig,

    /// 数据库配置
    pub database: DatabaseConfig,

    /// 日志配置
    pub logging: LoggingConfig,

    /// 敏感信息配置
    pub secrets: SecretsConfig,

    /// CORS 跨域资源共享配置
    pub cors: CorsConfig,

    /// Redis 连接池配置
    pub redis: RedisConfig,
}

impl AppConfig {
    /// 从配置文件和环境变量加载应用配置
    ///
    /// # 优先级顺序（从低到高）
    ///
    /// 1. 代码中的默认值
    /// 2. `config/default.toml` — 所有环境共享的基准配置
    /// 3. `config/{APP_ENV}.toml` — 当前环境专属配置（默认 `development`）
    /// 4. `config/local.toml` — 个人本地覆盖（gitignored，不进版本库）
    /// 5. `APP_*` 环境变量
    /// 6. 敏感信息环境变量（`DATABASE_URL`、`JWT_SECRET` 等，最高优先级）
    ///
    /// # 环境选择
    ///
    /// 通过 `APP_ENV` 环境变量指定，默认为 `development`。
    /// 例如：`APP_ENV=production cargo run`
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        // 读取当前环境（默认 development）
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        // 构建配置源（优先级从低到高）
        let builder = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{env}")).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(
                Environment::with_prefix("APP_")
                    .try_parsing(true)
                    .separator("_"),
            );

        let config = builder
            .build()
            .map_err(|e| ConfigError::Invalid(format!("配置构建失败：{}", e)))?;

        // 加载配置
        let mut app_config = Self::default();
        app_config.load_from_config(&config)?;

        // 应用环境变量覆盖（最高优先级）
        app_config.apply_env_overrides()?;

        // 验证配置
        app_config.validate()?;

        Ok(app_config)
    }

    /// 从 config 对象加载配置到各个配置段
    ///
    /// 这是支持 Trait 扩展的核心方法，通过 ConfigSection trait
    /// 实现灵活的配置加载机制。
    fn load_from_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        let app_config: AppConfig = config
            .clone()
            .try_deserialize()
            .map_err(|e| ConfigError::Invalid(format!("配置反序列化失败：{}", e)))?;

        self.server = app_config.server;
        self.database = app_config.database;
        self.logging = app_config.logging;
        self.secrets = app_config.secrets;
        self.cors = app_config.cors;
        self.redis = app_config.redis;

        Ok(())
    }

    /// 应用环境变量覆盖
    ///
    /// 遍历所有配置段，应用来自环境变量的覆盖值（最高优先级）。
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        let sections: Vec<&mut dyn ConfigSection> = vec![
            &mut self.server,
            &mut self.database,
            &mut self.logging,
            &mut self.secrets,
            &mut self.cors,
            &mut self.redis,
        ];

        for section in sections {
            section.apply_env_overrides().map_err(|e| {
                ConfigError::Invalid(format!(
                    "为 {} 应用环境变量覆盖失败：{}",
                    section.section_name(),
                    e
                ))
            })?;
        }

        Ok(())
    }

    /// 验证所有配置段
    ///
    /// 确保所有配置值都符合规范和约束条件。
    fn validate(&self) -> Result<(), ConfigError> {
        let sections: Vec<&dyn ConfigSection> = vec![
            &self.server,
            &self.database,
            &self.logging,
            &self.secrets,
            &self.cors,
            &self.redis,
        ];

        for section in sections {
            section.validate().map_err(|e| {
                ConfigError::Invalid(format!("{} 配置验证失败：{}", section.section_name(), e))
            })?;
        }

        Ok(())
    }

    /// 获取服务器监听地址
    ///
    /// # 返回值
    ///
    /// 格式为 "host:port" 的地址字符串
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// 初始化日志跟踪系统
    ///
    /// # 返回值
    ///
    /// 成功返回 `Ok(())`，失败返回应用错误
    pub fn init_tracing(&self) -> Result<(), crate::AppError> {
        crate::core::logging::init_tracing(&self.logging)
    }
}
