/// 核心功能模块（配置、日志、中间件等）
mod core;
/// 错误处理模块
mod error;
/// 业务功能模块
mod modules;
/// API路由模块
mod routes;
/// 共享工具模块（JWT、密码等）
mod shared;

pub use core::*;
pub use error::*;
pub use modules::*;
pub use routes::v1;

use aide::axum::{ApiRouter, IntoApiResponse};
use aide::openapi::{OpenApi, Tag};
use aide::transform::TransformOpenApi;
use axum::body::Body;
use axum::error_handling::HandleErrorLayer;
use axum::http::Request;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::{BoxError, Extension, routing::get};
use migration::{Migrator, MigratorTrait};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::signal;
use tower::ServiceBuilder;
use tower::buffer::BufferLayer;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{Level, error, info, instrument};

/// 健康检查端点
///
/// 返回服务器是否正常运行的状态。
#[instrument]
async fn health_check() -> ApiResponse<Value> {
    info!("健康检查请求");
    ApiResponse::success(json!({
        "status": "healthy"
    }))
}

/// Hello World 测试端点
///
/// 返回一条简单的问候消息，用于测试服务器是否正常响应。
#[instrument]
async fn hello_world() -> ApiResponse<Value> {
    info!("Hello World 请求");
    ApiResponse::success(json!({
        "message": "Hello, World!"
    }))
}

/// 应用程序主入口点
///
/// 负责以下初始化工作：
/// - 加载和验证配置
/// - 初始化日志系统
/// - 建立数据库连接并执行迁移
/// - 初始化应用状态（包括Redis连接）
/// - 启动日志清理任务
/// - 构建HTTP服务器并启动
///
/// # 返回
/// 正常退出返回 Ok(())，发生错误返回 AppError
#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 加载配置
    let config = AppConfig::load()?;
    // 初始化 tracing 日志系统
    config.init_tracing()?;

    // sea-orm 数据库连接和自动迁移
    let connection = sea_orm::Database::connect(&config.database.url).await?;
    Migrator::up(&connection, None).await?;

    // 初始化 API 文档生成
    aide::generate::on_error(|error| println!("{error}"));
    aide::generate::extract_schemas(true);

    // 输出启动信息
    info!("🚀 应用启动");
    info!("服务器地址: {}", config.server_addr());
    info!("数据库连接池: {} 个连接", config.database.max_connections);
    info!("日志级别: {}", config.logging.level);

    // 初始化应用状态（包含数据库连接、Redis 连接池等）
    let app_state = Arc::new(AppState::init(&config).await?);

    // 输出 Redis 连接状态
    if app_state.redis.is_some() {
        info!("✅ Redis 连接池已初始化");
    }

    // 初始化日志清理任务
    if config.logging.cleanup_enabled {
        if config.logging.cleanup_interval == 0 {
            // 启动时立即清理一次日志文件
            cleanup_old_logs(&config.logging)?;
        } else {
            // 启动后台清理任务（cleanup_interval 单位为小时）
            let cleanup_config = config.logging.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(
                        cleanup_config.cleanup_interval * 3600,
                    ))
                    .await;
                    if let Err(e) = cleanup_old_logs(&cleanup_config) {
                        tracing::error!("日志清理任务出错: {}", e);
                    }
                }
            });
        }
    }

    // 构建路由
    let mut api = OpenApi::default();

    // 构建基础路由
    let mut app = ApiRouter::new()
        .nest_service("/static", ServeDir::new("app/assets"))
        .route("/health", get(health_check))
        .route("/", get(hello_world))
        .route("/favicon.ico", get(favicon))
        .nest_api_service("/v1", v1::routes(app_state.clone()));

    // 只在 debug 模式下添加 API 文档路由
    if config.logging.level == "debug" {
        app = app.nest_api_service("/docs", docs_routes(&*app_state));
    }

    // 配置 CORS
    let cors_layer = build_cors_layer(&config.cors)?;
    info!(
        "🌐 CORS 配置：允许源 {:?}，允许凭证 {}",
        config.cors.allow_origins, config.cors.allow_credentials
    );

    // 配置速率限制
    // 注意：在本地开发环境中，SmartIpKeyExtractor 可能无法正确提取 IP 地址
    // 生产环境中，确保配置了正确的 ConnectInfo 中间件
    let general_limiter = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(10)
            .burst_size(20)
            .use_headers()
            .finish()
            .unwrap(),
    );
    info!("⚡ 速率限制已启用: 每秒10个请求，突发20个请求");

    // 应用所有中间件
    let app = app
        .finish_api_with(&mut api, api_docs)
        .fallback(handle_404)
        .layer(
            ServiceBuilder::new()
                // CORS 跨域配置
                .layer(cors_layer)
                // 基于 IP 的速率限制
                .layer(GovernorLayer::new(general_limiter))
                // 错误处理层（处理 GovernorLayer 和其他中间件的错误）
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                // 缓冲层
                .layer(BufferLayer::new(1024))
                // HTTP 响应压缩（gzip/deflate/brotli）
                .layer(CompressionLayer::new())
                // 请求 ID 中间件（用于追踪）
                .layer(axum::middleware::from_fn(middleware::request_id_middleware))
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
                ),
        )
        .layer(Extension(Arc::new(api)))
        .with_state(app_state);

    // 绑定监听地址
    let listener = tokio::net::TcpListener::bind(&config.server_addr()).await?;
    info!("🎯 服务器启动在: http://{}", config.server_addr());

    // 优雅关闭处理
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
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

/// 监听系统关闭信号
///
/// 等待 Ctrl+C (SIGINT) 或 SIGTERM 信号，触发时返回。
/// 支持跨平台：
/// - Unix系统：监听 SIGTERM 和 SIGINT
/// - Windows系统：仅监听 Ctrl+C (SIGINT)
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

/// 获取网站图标
///
/// 返回网站的 favicon.png 文件，用于浏览器标签页显示。
async fn favicon() -> impl IntoApiResponse {
    let favicon = include_bytes!("../assets/favicon.png");
    ([(CONTENT_TYPE, "image/x-icon")], favicon.as_ref())
}

/// robots.txt
// async fn robots_txt() -> impl IntoApiResponse {
//     let robots = include_str!("../assets/robots.txt");
//     ([(CONTENT_TYPE, "text/plain")], robots.as_bytes())
// }

/// 配置 OpenAPI 文档
///
/// # 参数
/// * `api` - OpenAPI 文档转换器
///
/// # 返回
/// 配置后的 OpenAPI 文档转换器
fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("DropBuddy API Documentation")
        .summary("API for the DropBuddy platform")
        // .description(include_str!("README.md")) 
        .tag(Tag {
            name: "❤️💕".into(),
            description: Some("Endpoints related to community features and content.".into()),
            ..Default::default()
        })
        .security_scheme(
            "ApiKey",
            aide::openapi::SecurityScheme::ApiKey {
                location: aide::openapi::ApiKeyLocation::Header,
                name: "X-Auth-Key".into(),
                description: Some("API Key for authentication (Note: This might be a placeholder and needs proper implementation description).".into()), // 更谨慎的描述
                extensions: Default::default(),
            },
        )
}
