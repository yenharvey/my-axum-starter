use aide::generate::GenContext;
use aide::openapi::Operation;
use aide::OperationOutput;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// 统一的API响应结构
/// 
/// 为所有API端点提供一致的响应格式，包含状态码、消息、数据和时间戳
#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    /// 响应状态码，0表示成功，其他值表示错误
    pub code: u16,
    /// 响应消息
    pub msg: String,
    /// 响应数据，成功时包含实际数据，错误时为空
    pub data: Option<T>,
    /// 响应时间戳，ISO 8601格式
    pub timestamp: String,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    /// 创建新的API响应
    /// 
    /// # Arguments
    /// 
    /// * `code` - 状态码
    /// * `msg` - 响应消息
    /// * `data` - 响应数据
    /// 
    /// # Returns
    /// 
    /// 新的 `ApiResponse` 实例
    pub fn new(code: u16, msg: String, data: Option<T>) -> Self {
        ApiResponse {
            code,
            msg,
            data,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建成功响应
    /// 
    /// # Arguments
    /// 
    /// * `data` - 成功响应的数据
    /// 
    /// # Returns
    /// 
    /// 状态码为0的成功响应
    pub fn success(data: T) -> Self {
        Self::new(0, "Success".to_string(), Some(data))
    }

    /// 创建错误响应
    /// 
    /// # Arguments
    /// 
    /// * `code` - 错误状态码
    /// * `msg` - 错误消息
    /// 
    /// # Returns
    /// 
    /// 不包含数据的错误响应
    pub fn error(code: u16, msg: String) -> ApiResponse<()> {
        ApiResponse {
            code,
            msg,
            data: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 创建通用失败响应
    /// 
    /// # Arguments
    /// 
    /// * `msg` - 失败消息
    /// 
    /// # Returns
    /// 
    /// 状态码为1的失败响应
    pub fn fail(msg: String) -> ApiResponse<()> {
        Self::error(1, msg)
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl<T: Serialize> OperationOutput for ApiResponse<T> {
    type Inner = ();

    fn operation_response(
        _ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Option<aide::openapi::Response> {
        None
    }

    fn inferred_responses(
        _ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        Vec::new()
    }
}