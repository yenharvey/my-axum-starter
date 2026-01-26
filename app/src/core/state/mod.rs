mod runtime;

pub use runtime::AppStateConfig;

use crate::{AppConfig, AppError, ValidationError, shared::jwt::JwtService};
use deadpool_redis::Pool as RedisPool;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

/// 应用程序运行时状态
///
/// 包含应用程序在运行时需要的所有共享资源，如数据库连接、Redis 连接池和配置。
/// 通过 Axum 的 State 提取器传递给每个处理器。
#[derive(Debug, Clone)]
pub struct AppState {
    /// 数据库连接
    pub db: DatabaseConnection,

    /// Redis 连接池（可选）
    pub redis: Option<RedisPool>,

    /// JWT 服务
    pub jwt_service: JwtService,

    /// 应用状态配置
    pub config: AppStateConfig,
}

impl AppState {
    /// 初始化应用状态
    ///
    /// 根据应用配置创建数据库连接池和运行时状态。
    ///
    /// # 参数
    ///
    /// * `app_config` - 应用配置对象
    ///
    /// # 返回值
    ///
    /// 成功返回初始化后的应用状态，失败返回应用错误
    ///
    /// # 异步
    ///
    /// 此方法是异步的，因为建立数据库连接是 I/O 操作
    pub async fn init(app_config: &AppConfig) -> Result<Self, AppError> {
        let db = Self::create_db_connection(app_config).await?;
        let redis = Self::create_redis_pool(app_config).await?;
        let jwt_service = JwtService::new(app_config.clone().secrets.jwt_secret.clone());

        Ok(AppState {
            db,
            redis,
            jwt_service,
            config: AppStateConfig {
                jwt_secret: app_config.clone().secrets.jwt_secret,
            },
        })
    }

    /// 创建数据库连接
    ///
    /// 根据应用配置创建连接池并连接到数据库。
    ///
    /// # 参数
    ///
    /// * `app_config` - 应用配置对象
    ///
    /// # 返回值
    ///
    /// 成功返回数据库连接，失败返回应用错误
    async fn create_db_connection(app_config: &AppConfig) -> Result<DatabaseConnection, AppError> {
        let mut opt = ConnectOptions::new(app_config.database.url.as_str());
        opt.max_connections(app_config.database.max_connections)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(app_config.database.pool_timeout))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(app_config.logging.level.parse().map_err(|_| {
                AppError::Validation(ValidationError::custom(format!(
                    "无效的日志级别：{}",
                    app_config.logging.level
                )))
            })?);

        Database::connect(opt).await.map_err(AppError::Database)
    }

    /// 创建 Redis 连接池
    ///
    /// 根据应用配置创建 Redis 连接池。如果未配置 Redis URL，返回 None。
    ///
    /// # 参数
    ///
    /// * `app_config` - 应用配置对象
    ///
    /// # 返回值
    ///
    /// 成功返回 Redis 连接池（如果配置了）或 None，失败返回应用错误
    async fn create_redis_pool(app_config: &AppConfig) -> Result<Option<RedisPool>, AppError> {
        match &app_config.redis.url {
            Some(redis_url) => {
                let cfg = deadpool_redis::Config::from_url(redis_url);
                let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

                tracing::info!("Redis 连接池已初始化");

                Ok(Some(pool))
            }
            None => {
                tracing::info!("未配置 Redis，将跳过 Redis 连接池初始化");
                Ok(None)
            }
        }
    }
}
