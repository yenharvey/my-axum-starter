//! 应用运行时状态

use std::time::Duration;

use deadpool_redis::Pool as RedisPool;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::shared::jwt::JwtService;
use crate::{AppConfig, AppError};

/// 连接空闲多久后被回收
const DB_IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// 单个连接的最长存活时间
const DB_MAX_LIFETIME: Duration = Duration::from_secs(30 * 60);
/// 连接池保持的最小连接数（不会超过 `max_connections`）
const DB_MIN_CONNECTIONS: u32 = 5;

/// 应用程序运行时状态
///
/// 包含运行时共享的资源，通过 Axum 的 `State` 提取器传递给每个处理器。
#[derive(Debug, Clone)]
pub struct AppState {
    /// 数据库连接池
    pub db: DatabaseConnection,

    /// Redis 连接池（未配置 Redis 时为 `None`）
    pub redis: Option<RedisPool>,

    /// JWT 服务
    pub jwt_service: JwtService,
}

impl AppState {
    /// 根据配置初始化应用状态
    pub async fn init(config: &AppConfig) -> Result<Self, AppError> {
        Ok(Self {
            db: Self::connect_db(config).await?,
            redis: Self::create_redis_pool(config)?,
            jwt_service: JwtService::new(&config.auth.jwt_secret, config.auth.token_ttl_secs),
        })
    }

    async fn connect_db(config: &AppConfig) -> Result<DatabaseConnection, AppError> {
        let database = &config.database;
        let timeout = Duration::from_secs(database.connect_timeout_secs);

        let mut options = ConnectOptions::new(database.url.as_str());
        options
            .max_connections(database.max_connections)
            .min_connections(database.max_connections.min(DB_MIN_CONNECTIONS))
            .connect_timeout(timeout)
            .acquire_timeout(timeout)
            .idle_timeout(DB_IDLE_TIMEOUT)
            .max_lifetime(DB_MAX_LIFETIME)
            // SQL 语句只在 debug 级别输出，避免生产环境日志被刷屏
            .sqlx_logging(true)
            .sqlx_logging_level("debug".parse().expect("debug 是合法的日志级别"));

        Ok(Database::connect(options).await?)
    }

    fn create_redis_pool(config: &AppConfig) -> Result<Option<RedisPool>, AppError> {
        let Some(url) = &config.redis.url else {
            tracing::info!("未配置 Redis，跳过 Redis 连接池初始化");
            return Ok(None);
        };

        let pool = deadpool_redis::Config::from_url(url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
        tracing::info!("✅ Redis 连接池已初始化");

        Ok(Some(pool))
    }
}
