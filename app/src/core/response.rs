use aide::OperationOutput;
use aide::generate::GenContext;
use aide::openapi::Operation;
use axum::Json;
use axum::response::{IntoResponse, Response};
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::Serialize;

/// 统一的API响应结构
///
/// 为所有API端点提供一致的响应格式，包含业务错误码、消息、数据和时间戳
///
/// code 字段说明：
/// - 0: 成功
/// - 10000-10099: Redis 相关错误
/// - 10100-10199: 数据库相关错误
/// - 10200-10299: 配置相关错误
/// - 10300-10399: 其他系统错误 (IO、Serialization等)
/// - 11000-11099: 验证错误 (用户数据验证失败)
/// - 11100-11199: 文件上传相关错误
/// - 11200-11299: 其他业务错误
/// - 其他: 通用错误
#[derive(Serialize, JsonSchema)]
pub struct ApiResponse<T>
where
    T: Serialize + JsonSchema,
{
    /// 业务状态码，0表示成功，其他值表示错误（非HTTP状态码）
    pub code: u32,

    /// 响应消息
    pub msg: String,

    /// 响应数据，成功时包含实际数据，错误时为空
    pub data: Option<T>,

    /// 响应时间戳，ISO 8601格式
    pub timestamp: String,
}

impl<T> ApiResponse<T>
where
    T: Serialize + JsonSchema,
{
    /// 创建新的API响应
    ///
    /// # 参数
    ///
    /// * `code` - 业务状态码
    /// * `msg` - 响应消息
    /// * `data` - 响应数据
    ///
    /// # 返回值
    ///
    /// 新的 `ApiResponse` 实例
    pub fn new(code: u32, msg: String, data: Option<T>) -> Self {
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
    /// # 参数
    ///
    /// * `code` - 业务错误码
    /// * `msg` - 错误消息
    ///
    /// # 返回值
    ///
    /// 不包含数据的错误响应
    pub fn error(code: u32, msg: String) -> ApiResponse<()> {
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
    T: Serialize + JsonSchema,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl<T: Serialize + JsonSchema> OperationOutput for ApiResponse<T> {
    type Inner = T;

    fn operation_response(
        ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Option<aide::openapi::Response> {
        let schema = ctx.schema.subschema_for::<ApiResponse<T>>();

        let schema_obj = aide::openapi::SchemaObject {
            json_schema: schema,
            external_docs: None,
            example: None,
        };

        let mut content = IndexMap::new();
        content.insert(
            "application/json".to_string(),
            aide::openapi::MediaType {
                schema: Some(schema_obj),
                ..Default::default()
            },
        );

        Some(aide::openapi::Response {
            description: "操作成功，返回相应的数据".to_string(),
            content,
            ..Default::default()
        })
    }

    fn inferred_responses(
        _ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        Vec::new()
    }
}
