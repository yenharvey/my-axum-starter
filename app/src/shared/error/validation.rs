//! 验证相关错误

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::shared::response::{ApiError, ApiResponse, Domain, ErrorDetail, Reason};

/// 单个字段的校验错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// 字段名
    pub field: String,
    /// 机器可读的错误原因
    pub reason: Reason,
    /// 给用户看的错误消息
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    /// 请求体字段校验失败（来自 `validator`），每个字段一条错误详情
    #[error("{}", join_messages(.0))]
    Fields(Vec<FieldError>),

    #[error("{0}")]
    Custom(String),
}

impl ValidationError {
    /// 从 validator::ValidationErrors 创建
    pub fn from_validator(errors: validator::ValidationErrors) -> Self {
        let mut fields: Vec<FieldError> = errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| FieldError {
                    field: field.to_string(),
                    reason: reason_for(&error.code),
                    message: error
                        .message
                        .as_ref()
                        .map(|message| message.to_string())
                        .unwrap_or_else(|| "验证失败".to_string()),
                })
            })
            .collect();
        // field_errors 的顺序不固定，排序后响应才稳定
        fields.sort_by(|a, b| a.field.cmp(&b.field));

        Self::Fields(fields)
    }

    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

/// validator 的校验规则名对应的错误原因
fn reason_for(code: &str) -> Reason {
    match code {
        "required" => Reason::RequiredFieldMissing,
        "length" => Reason::InvalidLength,
        "range" => Reason::ValueOutOfRange,
        "email" => Reason::InvalidEmail,
        "must_match" => Reason::PasswordMismatch,
        _ => Reason::InvalidFormat,
    }
}

fn join_messages(fields: &[FieldError]) -> String {
    fields
        .iter()
        .map(|error| format!("{}: {}", error.field, error.message))
        .collect::<Vec<_>>()
        .join("; ")
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let api_error = ApiError::new(StatusCode::BAD_REQUEST, self.to_string());

        let api_error = match self {
            Self::Fields(fields) => api_error.with_details(fields.into_iter().map(|error| {
                ErrorDetail::with_message(Domain::VALIDATION, error.reason, error.message)
                    .at(error.field, "body")
            })),
            Self::Custom(message) => api_error.with_detail(ErrorDetail::with_message(
                Domain::VALIDATION,
                Reason::InvalidFormat,
                message,
            )),
        };

        ApiResponse::error(api_error).into_response()
    }
}
