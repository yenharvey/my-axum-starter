use std::str::FromStr;
use std::time::Duration;

use axum::http::header::HeaderName;
use axum::http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

use crate::infra::config::CorsConfig;
use crate::shared::error::ConfigError;

/// 根据 CORS 配置构建 [`CorsLayer`]
///
/// 凭证与通配符的组合已在配置校验阶段（`CorsConfig::validate`）拦截。
pub fn build_cors_layer(config: &CorsConfig) -> Result<CorsLayer, ConfigError> {
    let is_wildcard = |values: &[String]| values.iter().any(|value| value == "*");

    let mut cors = CorsLayer::new()
        .allow_credentials(config.allow_credentials)
        .max_age(Duration::from_secs(config.max_age_secs));

    // 源列表为空时不设置 allow_origin，即不允许任何跨域请求
    cors = if is_wildcard(&config.allow_origins) {
        cors.allow_origin(Any)
    } else {
        cors.allow_origin(parse_all::<HeaderValue>(
            &config.allow_origins,
            "allow_origins",
        )?)
    };

    cors = if is_wildcard(&config.allow_methods) {
        cors.allow_methods(Any)
    } else {
        cors.allow_methods(parse_all::<Method>(&config.allow_methods, "allow_methods")?)
    };

    cors = if is_wildcard(&config.allow_headers) {
        cors.allow_headers(Any)
    } else {
        cors.allow_headers(parse_all::<HeaderName>(
            &config.allow_headers,
            "allow_headers",
        )?)
    };

    cors = if is_wildcard(&config.expose_headers) {
        cors.expose_headers(Any)
    } else {
        cors.expose_headers(parse_all::<HeaderName>(
            &config.expose_headers,
            "expose_headers",
        )?)
    };

    Ok(cors)
}

/// 解析配置列表，遇到无效值时报错（而不是静默丢弃）
fn parse_all<T: FromStr>(values: &[String], field: &str) -> Result<Vec<T>, ConfigError> {
    values
        .iter()
        .map(|value| {
            value
                .parse::<T>()
                .map_err(|_| ConfigError::Invalid(format!("[cors] {field} 包含无效值：{value}")))
        })
        .collect()
}
