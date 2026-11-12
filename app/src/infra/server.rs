//! HTTP 服务器启动
//!
//! 负责初始化应用状态、执行数据库迁移、装配全局中间件并启动服务。

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::error_handling::HandleErrorLayer;
use axum::http::{Request, StatusCode};
use axum::{BoxError, Router};
use migration::{Migrator, MigratorTrait};
use tokio::signal;
use tower::ServiceBuilder;
use tower::buffer::BufferLayer;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{Level, error, info};

use crate::infra::bootstrap::{ensure_database_exists, seed_demo_data};
use crate::infra::middleware::{build_cors_layer, rate_limit_layer, request_id_middleware};
use crate::{AppConfig, AppError, AppState, routes};

/// 全局速率限制：每个 IP 每秒补充的令牌数
const RATE_LIMIT_PER_SECOND: u64 = 10;
/// 全局速率限制：每个 IP 允许的突发请求数
const RATE_LIMIT_BURST: u32 = 20;

/// 启动 HTTP 服务器，直到收到关闭信号
pub async fn run(config: AppConfig) -> Result<(), AppError> {
    info!("🚀 应用启动");

    if config.database.auto_create {
        ensure_database_exists(&config.database.url).await?;
    }

    // 初始化应用状态（包含数据库连接、Redis 连接池等）
    let state = Arc::new(AppState::init(&config).await?);

    // 复用应用连接池执行自动迁移（避免额外建立一个不会被释放的连接池）
    Migrator::up(&state.db, None).await?;

    if config.database.seed_demo {
        seed_demo_data(&state.db).await?;
    }

    let app = build_app(&config, state)?;

    let listener = tokio::net::TcpListener::bind(&config.server_addr()).await?;
    info!("🎯 服务器启动在: http://{}", config.server_addr());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(|e| {
        error!("服务器错误: {}", e);
        AppError::Io(e)
    })?;

    info!("🛑 服务器已优雅关闭");
    Ok(())
}

/// 构建路由并挂载全局中间件
fn build_app(config: &AppConfig, state: Arc<AppState>) -> Result<Router, AppError> {
    let router = routes::router(state, config.server.enable_docs);

    let cors_layer = build_cors_layer(&config.cors)?;
    info!(
        "🌐 CORS 配置：允许源 {:?}，允许凭证 {}",
        config.cors.allow_origins, config.cors.allow_credentials
    );

    // 中间件自上而下依次包裹：越靠上越外层
    let middleware = ServiceBuilder::new()
        // CORS 跨域配置
        .layer(cors_layer)
        // 基于 IP 的速率限制
        .layer(rate_limit_layer(RATE_LIMIT_PER_SECOND, RATE_LIMIT_BURST))
        // 将 BufferLayer 等中间件产生的错误转换为响应
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            error!(error = %err, "unhandled middleware error");
            StatusCode::INTERNAL_SERVER_ERROR
        }))
        // 缓冲层
        .layer(BufferLayer::new(1024))
        // HTTP 响应压缩（gzip/deflate/brotli）
        .layer(CompressionLayer::new())
        // 请求 ID 中间件（用于追踪）
        .layer(axum::middleware::from_fn(request_id_middleware))
        // 请求追踪和日志
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown");
                tracing::span!(
                    Level::DEBUG,
                    "request",
                    method = display(request.method()),
                    uri = display(request.uri()),
                    version = debug(request.version()),
                    request_id = request_id
                )
            }),
        )
        // 请求超时
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.server.request_timeout_secs),
        ));

    Ok(router.layer(middleware))
}

/// 监听系统关闭信号
///
/// 等待 Ctrl+C (SIGINT) 或 SIGTERM 信号，触发时返回。
/// - Unix 系统：监听 SIGTERM 和 SIGINT
/// - Windows 系统：仅监听 Ctrl+C
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("🔄 收到 Ctrl+C 信号，开始优雅关闭...");
        },
        _ = terminate => {
            info!("🔄 收到 SIGTERM 信号，开始优雅关闭...");
        },
    }
}
