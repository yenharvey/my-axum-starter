use axum::http::HeaderValue;
use axum::http::header::HeaderName;
use axum::http::method::Method;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

use super::config::CorsConfig;
use crate::error::ConfigError;

/// 根据 CORS 配置构建 CorsLayer
///
/// 根据配置对象动态构建跨域资源共享的中间件，包括：
/// - 允许的源（支持通配符和特定域名）
/// - 允许的请求方法
/// - 允许的请求头
/// - 暴露的响应头
/// - 凭证和缓存时间设置
///
/// # 参数
///
/// * `cors_config` - CORS 配置对象
///
/// # 返回值
///
/// 配置好的 CorsLayer 中间件
///
/// # 示例
///
/// ```ignore
/// let config = AppConfig::load()?;
/// let cors_layer = build_cors_layer(&config.cors)?;
/// ```
pub fn build_cors_layer(cors_config: &CorsConfig) -> Result<CorsLayer, ConfigError> {
    // 当允许凭证时，不能使用通配符方法
    if cors_config.allow_credentials && cors_config.allow_methods.contains(&"*".to_string()) {
        return Err(ConfigError::Invalid(
            "Cannot combine `Access-Control-Allow-Credentials: true` with `Access-Control-Allow-Methods: *`"
                .to_string(),
        ));
    }

    let mut cors = CorsLayer::new();

    // 处理允许的请求方法
    if cors_config.allow_methods.contains(&"*".to_string()) {
        cors = cors.allow_methods(Any);
    } else {
        let methods: Vec<Method> = cors_config
            .allow_methods
            .iter()
            .filter_map(|m| m.parse::<Method>().ok())
            .collect();
        if !methods.is_empty() {
            cors = cors.allow_methods(methods);
        }
    }

    // 处理允许的源
    if cors_config.allow_origins.contains(&"*".to_string()) {
        cors = cors.allow_origin(Any);
    } else {
        let origins: Vec<HeaderValue> = cors_config
            .allow_origins
            .iter()
            .filter_map(|origin| origin.parse::<HeaderValue>().ok())
            .collect();
        if !origins.is_empty() {
            cors = cors.allow_origin(origins);
        }
    }

    // 处理允许的请求头
    let allow_headers: Vec<HeaderName> = cors_config
        .allow_headers
        .iter()
        .filter_map(|header| header.parse::<HeaderName>().ok())
        .collect();
    if !allow_headers.is_empty() {
        cors = cors.allow_headers(allow_headers);
    }

    // 处理暴露的响应头
    let expose_headers: Vec<HeaderName> = cors_config
        .expose_headers
        .iter()
        .filter_map(|header| header.parse::<HeaderName>().ok())
        .collect();
    if !expose_headers.is_empty() {
        cors = cors.expose_headers(expose_headers);
    }

    // 设置凭证和缓存时间
    if cors_config.allow_credentials {
        cors = cors.allow_credentials(true);
    }

    cors = cors.max_age(Duration::from_secs(cors_config.max_age));

    Ok(cors)
}
