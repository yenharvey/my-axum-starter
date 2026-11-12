use axum::body::Body;
use axum::response::{IntoResponse, Response};
use governor::middleware::StateInformationMiddleware;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};

use crate::ApiResponse;
use crate::shared::response::{ApiError, Domain};

/// 构建基于对端 IP 的速率限制层
///
/// 每个 IP 每秒补充 `per_second` 个令牌，最多累积 `burst_size` 个。
/// 超限时返回统一格式的 429 响应，并带上 `x-ratelimit-*` 响应头。
///
/// 注意：部署在反向代理之后时所有请求的对端 IP 相同，
/// 需要改用 `SmartIpKeyExtractor` 从 `X-Forwarded-For` 等请求头提取真实 IP。
///
/// # Panics
///
/// `per_second` 或 `burst_size` 为 0 时 panic（两者都是代码里的常量）。
pub fn rate_limit_layer(
    per_second: u64,
    burst_size: u32,
) -> GovernorLayer<PeerIpKeyExtractor, StateInformationMiddleware, Body> {
    let config = GovernorConfigBuilder::default()
        .per_second(per_second)
        .burst_size(burst_size)
        .use_headers()
        .finish()
        .expect("速率限制参数必须大于 0");

    GovernorLayer::new(config).error_handler(rate_limit_error_response)
}

/// 把限流错误转换为统一格式的 API 响应
fn rate_limit_error_response(err: GovernorError) -> Response {
    let (mut response, headers) = match err {
        GovernorError::TooManyRequests { headers, .. } => {
            (ApiResponse::too_many_requests().into_response(), headers)
        }
        GovernorError::UnableToExtractKey => {
            tracing::error!("无法提取速率限制的 key（缺少 ConnectInfo？）");
            (
                ApiResponse::internal_error(Domain::RATE_LIMIT).into_response(),
                None,
            )
        }
        GovernorError::Other { code, msg, headers } => {
            tracing::error!(?msg, "速率限制错误");
            let message = msg.unwrap_or_else(|| "Rate limit error".to_string());
            (
                ApiResponse::error(ApiError::new(code, message)).into_response(),
                headers,
            )
        }
    };

    if let Some(headers) = headers {
        response.headers_mut().extend(headers);
    }

    response
}
