//! 错误原因定义
//!
//! 机器可读的错误标识符，用于 `errors[].reason` 字段。
//! 使用 UPPER_SNAKE_CASE 格式，前端以此作为 i18n key 映射用户可见文案。
//!
//! ## 防膨胀规则
//!
//! 只有「前端需要针对它做不同处理」的错误才值得单独一个 variant。
//! 内部实现错误（数据库、Redis、配置等）统一返回 `InternalError`，详情只进日志。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 错误原因
///
/// 每个 reason 在特定 domain 内唯一标识一种错误。
/// 客户端可根据 `(domain, reason)` 组合编程处理特定错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum Reason {
    // ==================== 认证 (auth) ====================
    /// 用户不存在
    UserNotFound,
    /// 密码错误
    InvalidPassword,
    /// 访问令牌无效
    InvalidToken,
    /// 访问令牌已过期
    TokenExpired,
    /// 缺少认证凭据
    MissingCredentials,
    /// 认证失败（通用）
    AuthenticationFailed,

    // ==================== 验证 (validation) ====================
    /// 格式无效
    InvalidFormat,
    /// 缺少必需字段
    RequiredFieldMissing,
    /// 值超出有效范围
    ValueOutOfRange,
    /// 长度无效
    InvalidLength,
    /// 邮箱格式无效
    InvalidEmail,
    /// 用户名格式无效
    InvalidUsername,
    /// 密码强度不足
    WeakPassword,
    /// 两次密码不匹配
    PasswordMismatch,

    // ==================== 资源通用 ====================
    /// 资源未找到
    NotFound,
    /// 资源已存在
    AlreadyExists,
    /// 资源冲突
    Conflict,
    /// 使用次数/变更次数已达上限
    UsageLimitReached,

    // ==================== 权限 ====================
    /// 权限不足
    PermissionDenied,

    // ==================== 文件 (file) ====================
    /// 文件大小超出限制
    FileTooLarge,
    /// 文件类型不允许
    FileTypeNotAllowed,
    /// 上传失败
    UploadFailed,

    // ==================== 限流 (rate_limit) ====================
    /// 请求频率超限
    RateLimitExceeded,

    // ==================== 通用 ====================
    /// 内部服务器错误（不对外暴露细节，详情只进日志）
    InternalError,
    /// 服务暂时不可用
    ServiceUnavailable,
    /// 功能未实现
    NotImplemented,
    /// 请求超时
    Timeout,
    /// 未知错误
    #[default]
    Unknown,
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::InvalidPassword => "INVALID_PASSWORD",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::MissingCredentials => "MISSING_CREDENTIALS",
            Self::AuthenticationFailed => "AUTHENTICATION_FAILED",
            Self::InvalidFormat => "INVALID_FORMAT",
            Self::RequiredFieldMissing => "REQUIRED_FIELD_MISSING",
            Self::ValueOutOfRange => "VALUE_OUT_OF_RANGE",
            Self::InvalidLength => "INVALID_LENGTH",
            Self::InvalidEmail => "INVALID_EMAIL",
            Self::InvalidUsername => "INVALID_USERNAME",
            Self::WeakPassword => "WEAK_PASSWORD",
            Self::PasswordMismatch => "PASSWORD_MISMATCH",
            Self::NotFound => "NOT_FOUND",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::Conflict => "CONFLICT",
            Self::UsageLimitReached => "USAGE_LIMIT_REACHED",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::FileTooLarge => "FILE_TOO_LARGE",
            Self::FileTypeNotAllowed => "FILE_TYPE_NOT_ALLOWED",
            Self::UploadFailed => "UPLOAD_FAILED",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::InternalError => "INTERNAL_ERROR",
            Self::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            Self::NotImplemented => "NOT_IMPLEMENTED",
            Self::Timeout => "TIMEOUT",
            Self::Unknown => "UNKNOWN",
        };
        f.write_str(s)
    }
}
