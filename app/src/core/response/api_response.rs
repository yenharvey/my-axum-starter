//! API 响应结构
//!
//! 遵循 Google JSON Style Guide 的响应格式。

use aide::OperationOutput;
use aide::generate::GenContext;
use aide::openapi::Operation;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ApiError, Domain, ErrorDetail, Reason};

/// API 版本号
pub const API_VERSION: &str = "1.0";

/// API 响应
///
/// 遵循 Google JSON Style Guide，响应要么包含 `data`，要么包含 `error`。
/// 如果同时存在，`error` 优先。
///
/// ## 字段说明
///
/// - `api_version`: API 版本号（固定为 "1.0"）
/// - `data`: 成功时的数据对象（包含资源数据和元数据）
/// - `error`: 失败时的错误对象
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiResponse<T: Serialize> {
    /// API 版本号
    pub api_version: String,

    /// 成功时的数据（包装在 DataWrapper 中）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<DataWrapper<T>>,

    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
}

/// Data 对象包装器
///
/// 支持 Google JSON Style Guide 定义的保留属性：
/// - `kind`: 资源类型标识
/// - `id`: 资源唯一标识符
/// - `etag`: 资源版本标识（用于缓存和并发控制）
/// - `lang`: 语言标识（BCP 47 格式）
/// - `updated`: 最后更新时间（RFC 3339 格式）
/// - `deleted`: 资源是否已删除
///
/// 实际数据内容通过 `content` 字段提供，支持单个资源或列表。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataWrapper<T: Serialize> {
    /// 资源类型标识（如 "User", "UserList"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    /// 资源唯一标识符
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// ETag 版本标识（用于缓存和并发控制）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,

    /// 语言标识（BCP 47 格式，如 "en", "zh-CN"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,

    /// 最后更新时间（RFC 3339 格式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,

    /// 资源是否已删除
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,

    /// 实际数据内容（单个资源或列表）
    #[serde(flatten)]
    pub content: DataContent<T>,
}

/// Data 内容（单个资源或列表）
///
/// 使用 `untagged` 枚举自动序列化为扁平结构：
/// - `Single(T)`: 单个资源，直接展开为 JSON 对象
/// - `List { items, ... }`: 列表资源，包含 items 数组和分页元数据
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DataContent<T: Serialize> {
    /// 单个资源
    Single(T),

    /// 列表资源（带分页信息）
    /// 使用 Box 减小枚举大小
    List(Box<ListData<T>>),
}

/// 列表数据及分页信息
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListData<T: Serialize> {
    /// 数据项列表
    pub items: Vec<T>,

    /// 当前返回的数据项数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_item_count: Option<i64>,

    /// 每页数据量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_per_page: Option<i64>,

    /// 起始索引（从 0 开始）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i64>,

    /// 总数据量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_items: Option<i64>,

    /// 当前页码（从 1 开始）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_index: Option<i64>,

    /// 总页数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<i64>,

    /// 分页链接模板（如 "http://api.example.com/users?page={page}"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_link_template: Option<String>,

    /// 下一页链接
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_link: Option<String>,

    /// 上一页链接
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_link: Option<String>,

    /// 当前页链接
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_link: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应（单个资源）
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use crate::response::ApiResponse;
    /// let response = ApiResponse::success(user);
    /// ```ignore
    pub fn success(data: T) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            data: Some(DataWrapper {
                kind: None,
                id: None,
                etag: None,
                lang: None,
                updated: None,
                deleted: None,
                content: DataContent::Single(data),
            }),
            error: None,
        }
    }

    /// 创建分页列表响应
    ///
    /// # Arguments
    ///
    /// * `items` - 数据项列表
    /// * `total` - 总数据量
    /// * `page` - 当前页码（从 1 开始）
    /// * `per_page` - 每页数据量
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use crate::response::ApiResponse;
    /// let response = ApiResponse::list(users, 100, 1, 10)
    ///     .with_kind("UserList")
    ///     .with_links(
    ///         Some("http://api.example.com/users?page=2".to_string()),
    ///         Some("http://api.example.com/users?page=0".to_string())
    ///     );
    /// ```ignore
    pub fn list(items: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        let current_count = items.len() as i64;
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as i64;
        let start_index = (page - 1) * per_page;

        Self {
            api_version: API_VERSION.to_string(),
            data: Some(DataWrapper {
                kind: None,
                id: None,
                etag: None,
                lang: None,
                updated: None,
                deleted: None,
                content: DataContent::List(Box::new(ListData {
                    items,
                    current_item_count: Some(current_count),
                    items_per_page: Some(per_page),
                    start_index: Some(start_index),
                    total_items: Some(total),
                    page_index: Some(page),
                    total_pages: Some(total_pages),
                    page_link_template: None,
                    next_link: None,
                    previous_link: None,
                    self_link: None,
                })),
            }),
            error: None,
        }
    }

    /// 创建简单列表响应（无分页信息）
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use crate::response::ApiResponse;
    /// let response = ApiResponse::simple_list(tags)
    ///     .with_kind("TagList");
    /// ```ignore
    pub fn simple_list(items: Vec<T>) -> Self {
        let current_count = items.len() as i64;

        Self {
            api_version: API_VERSION.to_string(),
            data: Some(DataWrapper {
                kind: None,
                id: None,
                etag: None,
                lang: None,
                updated: None,
                deleted: None,
                content: DataContent::List(Box::new(ListData {
                    items,
                    current_item_count: Some(current_count),
                    items_per_page: None,
                    start_index: None,
                    total_items: None,
                    page_index: None,
                    total_pages: None,
                    page_link_template: None,
                    next_link: None,
                    previous_link: None,
                    self_link: None,
                })),
            }),
            error: None,
        }
    }

    /// 设置资源类型
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use crate::response::ApiResponse;
    /// let response = ApiResponse::success(user).with_kind("User");
    /// ```ignore
    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        if let Some(ref mut data) = self.data {
            data.kind = Some(kind.into());
        }
        self
    }

    /// 设置资源 ID
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::success(user).with_id("user123");
    /// ```ignore
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        if let Some(ref mut data) = self.data {
            data.id = Some(id.into());
        }
        self
    }

    /// 设置 ETag
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::success(user).with_etag("\"abc123\"");
    /// ```ignore
    pub fn with_etag(mut self, etag: impl Into<String>) -> Self {
        if let Some(ref mut data) = self.data {
            data.etag = Some(etag.into());
        }
        self
    }

    /// 设置语言标识
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::success(user).with_lang("zh-CN");
    /// ```ignore
    pub fn with_lang(mut self, lang: impl Into<String>) -> Self {
        if let Some(ref mut data) = self.data {
            data.lang = Some(lang.into());
        }
        self
    }

    /// 设置更新时间
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::success(user)
    ///     .with_updated("2024-01-16T12:00:00Z");
    /// ```ignore
    pub fn with_updated(mut self, updated: impl Into<String>) -> Self {
        if let Some(ref mut data) = self.data {
            data.updated = Some(updated.into());
        }
        self
    }

    /// 设置删除标记
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::success(user).with_deleted(true);
    /// ```ignore
    pub fn with_deleted(mut self, deleted: bool) -> Self {
        if let Some(ref mut data) = self.data {
            data.deleted = Some(deleted);
        }
        self
    }

    /// 设置分页链接（仅对列表响应有效）
    ///
    /// # Arguments
    ///
    /// - `next`: 下一页链接
    /// - `previous`: 上一页链接
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::list(users, 100, 1, 10)
    ///     .with_links(
    ///         Some("http://api.example.com/users?page=2".to_string()),
    ///         Some("http://api.example.com/users?page=0".to_string())
    ///     );
    /// ```ignore
    pub fn with_links(mut self, next: Option<String>, previous: Option<String>) -> Self {
        if let Some(ref mut data) = self.data
            && let DataContent::List(ref mut list_data) = data.content
        {
            list_data.next_link = next;
            list_data.previous_link = previous;
        }
        self
    }

    /// 设置当前页链接（仅对列表响应有效）
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::list(users, 100, 1, 10)
    ///     .with_self_link("http://api.example.com/users?page=1");
    /// ```ignore
    pub fn with_self_link(mut self, link: impl Into<String>) -> Self {
        if let Some(ref mut data) = self.data
            && let DataContent::List(ref mut list_data) = data.content
        {
            list_data.self_link = Some(link.into());
        }
        self
    }

    /// 设置分页链接模板（仅对列表响应有效）
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::list(users, 100, 1, 10)
    ///     .with_page_link_template("http://api.example.com/users?page={page}");
    /// ```ignore
    pub fn with_page_link_template(mut self, template: impl Into<String>) -> Self {
        if let Some(ref mut data) = self.data
            && let DataContent::List(ref mut list_data) = data.content
        {
            list_data.page_link_template = Some(template.into());
        }
        self
    }

    /// 获取 HTTP 状态码
    fn status_code(&self) -> StatusCode {
        self.error
            .as_ref()
            .map(|e| e.status_code())
            .unwrap_or(StatusCode::OK)
    }
}

impl ApiResponse<()> {
    /// 创建错误响应
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let error = ApiError::new(StatusCode::NOT_FOUND, "用户不存在");
    /// let response = ApiResponse::error(error);
    /// ```ignore
    pub fn error(error: ApiError) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            data: None,
            error: Some(error),
        }
    }

    /// 从状态码、域、原因创建错误响应
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::fail(
    ///     StatusCode::NOT_FOUND,
    ///     Domain::Auth,
    ///     Reason::UserNotFound
    /// );
    /// ```ignore
    pub fn fail(status: StatusCode, domain: Domain, reason: Reason) -> Self {
        Self::error(ApiError::from_reason(status, domain, reason))
    }

    /// 带自定义消息的错误响应
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::fail_with_message(
    ///     StatusCode::BAD_REQUEST,
    ///     Domain::Validation,
    ///     Reason::InvalidParameter,
    ///     "用户名长度必须在 3-20 个字符之间"
    /// );
    /// ```ignore
    pub fn fail_with_message(
        status: StatusCode,
        domain: Domain,
        reason: Reason,
        message: impl Into<String>,
    ) -> Self {
        let msg = message.into();
        Self::error(
            ApiError::new(status, &msg).with_detail(ErrorDetail::with_message(domain, reason, msg)),
        )
    }

    // === 便捷方法（直接映射 HTTP 状态码）===

    /// 400 Bad Request
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::bad_request(Domain::Validation, Reason::InvalidParameter);
    /// ```ignore
    pub fn bad_request(domain: Domain, reason: Reason) -> Self {
        Self::fail(StatusCode::BAD_REQUEST, domain, reason)
    }

    /// 401 Unauthorized
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::unauthorized(Reason::InvalidToken);
    /// ```ignore
    pub fn unauthorized(reason: Reason) -> Self {
        Self::fail(StatusCode::UNAUTHORIZED, Domain::AUTH, reason)
    }

    /// 403 Forbidden
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::forbidden(Reason::InsufficientPermissions);
    /// ```ignore
    pub fn forbidden(reason: Reason) -> Self {
        Self::fail(StatusCode::FORBIDDEN, Domain::AUTH, reason)
    }

    /// 404 Not Found
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::not_found(Domain::User, Reason::UserNotFound);
    /// ```ignore
    pub fn not_found(domain: Domain, reason: Reason) -> Self {
        Self::fail(StatusCode::NOT_FOUND, domain, reason)
    }

    /// 409 Conflict
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::conflict(Domain::User, Reason::ResourceAlreadyExists);
    /// ```ignore
    pub fn conflict(domain: Domain, reason: Reason) -> Self {
        Self::fail(StatusCode::CONFLICT, domain, reason)
    }

    /// 422 Unprocessable Entity
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::unprocessable_entity(Domain::Validation, Reason::ValidationFailed);
    /// ```ignore
    pub fn unprocessable_entity(domain: Domain, reason: Reason) -> Self {
        Self::fail(StatusCode::UNPROCESSABLE_ENTITY, domain, reason)
    }

    /// 429 Too Many Requests
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::too_many_requests();
    /// ```ignore
    pub fn too_many_requests() -> Self {
        Self::fail(
            StatusCode::TOO_MANY_REQUESTS,
            Domain::RATE_LIMIT,
            Reason::RateLimitExceeded,
        )
    }

    /// 500 Internal Server Error
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::internal_error(Domain::Database);
    /// ```ignore
    pub fn internal_error(domain: Domain) -> Self {
        Self::fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            domain,
            Reason::InternalError,
        )
    }

    /// 503 Service Unavailable
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let response = ApiResponse::service_unavailable(Domain::External);
    /// ```ignore
    pub fn service_unavailable(domain: Domain) -> Self {
        Self::fail(
            StatusCode::SERVICE_UNAVAILABLE,
            domain,
            Reason::ServiceUnavailable,
        )
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = self.status_code();
        (status, Json(self)).into_response()
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
            description: "API 响应".to_string(),
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
