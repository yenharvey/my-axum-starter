use crate::{AppConfig, AppError};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppStateConfig,
}

#[derive(Debug, Clone)]
pub struct AppStateConfig {
    pub jwt_secret: String,
}

impl AppState {
    pub async fn init(app_config: &AppConfig) -> Result<Self, AppError> {
        let db = Self::create_db_connection(app_config).await?;

        Ok(AppState {
            db,
            config: AppStateConfig {
                jwt_secret: app_config.clone().secrets.jwt_secret,
            },
        })
    }

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
                AppError::Validation(format!("Invalid log level: {}", app_config.logging.level))
            })?);

        Database::connect(opt)
            .await
            .map_err(|e| AppError::Database(e))
    }
}
