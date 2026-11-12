//! REST API 标准状态码
//!
//! 借鉴 Google 17 个标准错误码的语义，映射到 HTTP 状态码。

use axum::http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// REST API 标准状态码
///
/// 借鉴 Google gRPC 的 17 个标准状态码语义，用于 REST API。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    /// 成功
    #[default]
    Ok,
    /// 操作被取消
    Cancelled,
    /// 未知错误
    Unknown,
    /// 无效参数
    InvalidArgument,
    /// 请求超时
    DeadlineExceeded,
    /// 资源未找到
    NotFound,
    /// 资源已存在
    AlreadyExists,
    /// 权限不足
    PermissionDenied,
    /// 资源耗尽（限流/配额）
    ResourceExhausted,
    /// 前置条件失败
    FailedPrecondition,
    /// 操作中止（并发冲突）
    Aborted,
    /// 超出范围
    OutOfRange,
    /// 未实现
    Unimplemented,
    /// 内部错误
    Internal,
    /// 服务不可用
    Unavailable,
    /// 数据丢失
    DataLoss,
    /// 未认证
    Unauthenticated,
}

impl Status {
    /// 获取对应的 HTTP 状态码
    pub const fn http_status(self) -> StatusCode {
        match self {
            Status::Ok => StatusCode::OK,
            Status::Cancelled => StatusCode::BAD_REQUEST,
            Status::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Status::InvalidArgument => StatusCode::BAD_REQUEST,
            Status::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
            Status::NotFound => StatusCode::NOT_FOUND,
            Status::AlreadyExists => StatusCode::CONFLICT,
            Status::PermissionDenied => StatusCode::FORBIDDEN,
            Status::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            Status::FailedPrecondition => StatusCode::BAD_REQUEST,
            Status::Aborted => StatusCode::CONFLICT,
            Status::OutOfRange => StatusCode::BAD_REQUEST,
            Status::Unimplemented => StatusCode::NOT_IMPLEMENTED,
            Status::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Status::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Status::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
            Status::Unauthenticated => StatusCode::UNAUTHORIZED,
        }
    }

    /// 获取状态码名称
    pub const fn as_str(self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::Cancelled => "CANCELLED",
            Status::Unknown => "UNKNOWN",
            Status::InvalidArgument => "INVALID_ARGUMENT",
            Status::DeadlineExceeded => "DEADLINE_EXCEEDED",
            Status::NotFound => "NOT_FOUND",
            Status::AlreadyExists => "ALREADY_EXISTS",
            Status::PermissionDenied => "PERMISSION_DENIED",
            Status::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Status::FailedPrecondition => "FAILED_PRECONDITION",
            Status::Aborted => "ABORTED",
            Status::OutOfRange => "OUT_OF_RANGE",
            Status::Unimplemented => "UNIMPLEMENTED",
            Status::Internal => "INTERNAL",
            Status::Unavailable => "UNAVAILABLE",
            Status::DataLoss => "DATA_LOSS",
            Status::Unauthenticated => "UNAUTHENTICATED",
        }
    }

    /// 是否是成功状态
    pub const fn is_ok(self) -> bool {
        matches!(self, Status::Ok)
    }

    /// 是否可重试
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Status::Unavailable
                | Status::ResourceExhausted
                | Status::Aborted
                | Status::DeadlineExceeded
        )
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<Status> for StatusCode {
    fn from(status: Status) -> Self {
        status.http_status()
    }
}
