mod core;
mod error;
mod modules;
mod routes;
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
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Request};
use axum::{http::StatusCode, routing::get, BoxError, Extension};
use migration::{Migrator, MigratorTrait};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{error, info, instrument, Level};

#[instrument]
async fn health_check() -> ApiResponse<Value> {
    info!("健康检查请求");
    ApiResponse::success(json!({
        "status": "healthy"
    }))
}

#[instrument]
async fn hello_world() -> ApiResponse<Value> {
    info!("Hello World 请求");
    ApiResponse::success(json!({
        "message": "Hello, World!"
    }))
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 加载配置
    let config = AppConfig::load()?;
    // 初始化 tracing
    config.init_tracing()?;
   
    // sea-orm 自动迁移
    // let connection = sea_orm::Database::connect(&config.database.url).await?;
    // Migrator::up(&connection, None).await?;

    aide::generate::on_error(|error| {
        println!("{error}");
    });
    aide::generate::extract_schemas(true);
    info!("🚀 应用启动");
    info!("服务器地址: {}", config.server_addr());
    info!("数据库连接池: {} 个连接", config.database.max_connections);
    info!("日志级别: {}", config.logging.level);
    
    // 初始化应用状态
    let app_state = Arc::new(AppState::init(&config).await?);
    let mut api = OpenApi::default();
    let cors_layer = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
    
    // 构建基础路由
    let mut app = ApiRouter::new()
        .nest_service("/static", ServeDir::new("app/assets"))
        .route("/health", get(health_check))
        .route("/", get(hello_world))
        .route("/favicon.ico", get(favicon))
        .nest_api_service("/v1", v1::routes(app_state.clone()));
    
    // 只在debug模式下添加文档路由
    if config.logging.level == "debug" {
        app = app.nest_api_service("/docs", docs_routes(&*app_state));
    }
    
    let app = app
        .finish_api_with(&mut api, api_docs)
        .fallback(handle_404)
        .layer(
            ServiceBuilder::new()
                .layer(cors_layer)
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(1)))
                .layer(axum::middleware::from_fn(middleware::request_id_middleware))
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
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&config.server_addr()).await?;
    info!("🎯 服务器启动在: http://{}", config.server_addr());
    
    // 优雅关闭
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            error!("服务器错误: {}", e);
            AppError::Io(e)
        })?;
    info!("🛑 服务器已优雅关闭");
    Ok(())
}

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

/// favicon
async fn favicon() -> impl IntoApiResponse {
    let favicon = include_bytes!("../assets/favicon.png");
    ([(CONTENT_TYPE, "image/x-icon")], favicon.as_ref())
}

/// robots.txt
// async fn robots_txt() -> impl IntoApiResponse {
//     let robots = include_str!("../assets/robots.txt");
//     ([(CONTENT_TYPE, "text/plain")], robots.as_bytes())
// }

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
