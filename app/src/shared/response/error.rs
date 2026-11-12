//! 错误结构定义
//!
//! 遵循 Google JSON Style Guide 的 error 对象结构。

use axum::http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Domain, Reason};

/// 错误详情（errors 数组中的元素）
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorDetail {
    /// 错误来源域
    pub domain: String,

    /// 错误原因标识符
    pub reason: String,

    /// 错误消息
    pub message: String,

    /// 错误位置（如字段名）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// 位置类型（parameter, header, body）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_type: Option<String>,
}

impl ErrorDetail {
    /// 从枚举创建
    pub fn new(domain: Domain, reason: Reason) -> Self {
        Self {
            domain: domain.to_string(),
            reason: reason.to_string(),
            message: reason.to_string(),
            location: None,
            location_type: None,
        }
    }

    /// 从枚举创建，带自定义消息
    pub fn with_message(domain: Domain, reason: Reason, message: impl Into<String>) -> Self {
        Self {
            domain: domain.to_string(),
            reason: reason.to_string(),
            message: message.into(),
            location: None,
            location_type: None,
        }
    }

    /// 设置位置信息
    pub fn at(mut self, location: impl Into<String>, location_type: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self.location_type = Some(location_type.into());
        self
    }
}

/// API 错误对象
///
/// Google JSON Style Guide 定义的 error 对象结构。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiError {
    /// HTTP 状态码
    pub code: u16,

    /// 主要错误消息
    pub message: String,

    /// 错误详情列表
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<ErrorDetail>,
}

impl ApiError {
    /// 创建错误
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code: status.as_u16(),
            message: message.into(),
            errors: Vec::new(),
        }
    }

    /// 从 Domain 和 Reason 创建
    pub fn from_reason(status: StatusCode, domain: Domain, reason: Reason) -> Self {
        Self {
            code: status.as_u16(),
            message: reason.to_string(),
            errors: vec![ErrorDetail::new(domain, reason)],
        }
    }

    /// 添加错误详情
    pub fn with_detail(mut self, detail: ErrorDetail) -> Self {
        self.errors.push(detail);
        self
    }

    /// 添加多个错误详情
    pub fn with_details(mut self, details: impl IntoIterator<Item = ErrorDetail>) -> Self {
        self.errors.extend(details);
        self
    }

    /// 获取 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    // === 便捷构造方法 ===

    pub fn bad_request(domain: Domain, reason: Reason) -> Self {
        Self::from_reason(StatusCode::BAD_REQUEST, domain, reason)
    }

    pub fn unauthorized(reason: Reason) -> Self {
        Self::from_reason(StatusCode::UNAUTHORIZED, Domain::AUTH, reason)
    }

    pub fn forbidden(reason: Reason) -> Self {
        Self::from_reason(StatusCode::FORBIDDEN, Domain::AUTH, reason)
    }

    pub fn too_many_requests(reason: Reason) -> Self {
        Self::from_reason(StatusCode::TOO_MANY_REQUESTS, Domain::RATE_LIMIT, reason)
    }
}
