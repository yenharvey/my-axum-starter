//! REST API 响应模块
//!
//! 遵循 Google JSON Style Guide 的 REST API 响应格式。
//!
//! ## 响应结构
//!
//! 响应要么包含 `data`（成功），要么包含 `error`（失败），不能同时存在。
//!
//! ### 成功响应（单个资源）
//!
//! ```json
//! {
//!   "api_version": "1.0",
//!   "data": {
//!     "kind": "User",
//!     "id": "user123",
//!     "etag": "\"abc123\"",
//!     "username": "admin",
//!     "email": "admin@example.com"
//!   }
//! }
//! ```
//!
//! ### 分页响应（列表资源）
//!
//! ```json
//! {
//!   "api_version": "1.0",
//!   "data": {
//!     "kind": "UserList",
//!     "items": [
//!       { "id": 1, "username": "admin" },
//!       { "id": 2, "username": "user" }
//!     ],
//!     "total_items": 100,
//!     "items_per_page": 10,
//!     "current_item_count": 2,
//!     "start_index": 0,
//!     "page_index": 1,
//!     "total_pages": 10,
//!     "next_link": "http://api.example.com/users?page=2",
//!     "previous_link": "http://api.example.com/users?page=0"
//!   }
//! }
//! ```
//!
//! ### 错误响应
//!
//! ```json
//! {
//!   "api_version": "1.0",
//!   "error": {
//!     "code": 404,
//!     "message": "用户不存在",
//!     "errors": [{
//!       "domain": "auth",
//!       "reason": "USER_NOT_FOUND",
//!       "message": "用户不存在",
//!       "location": "userId",
//!       "locationType": "parameter"
//!     }]
//!   }
//! }
//! ```
//!
//! ## 核心类型
//!
//! - [`ApiResponse`] - 统一响应包装器
//! - [`DataWrapper`] - Data 对象包装器（支持 Google 保留属性）
//! - [`DataContent`] - 数据内容（单个资源或列表）
//! - [`ApiError`] - 错误对象
//! - [`ErrorDetail`] - 错误详情
//! - [`Domain`] - 错误域枚举
//! - [`Reason`] - 错误原因枚举
//!
//! ## 使用示例
//!
//! ```ignore
//! use crate::shared::response::{ApiResponse, Domain, Reason};
//! use axum::http::StatusCode;
//!
//! // 单个资源响应
//! let response = ApiResponse::success(user)
//!     .with_kind("User")
//!     .with_id("user123");
//!
//! // 分页列表响应
//! let response = ApiResponse::list(users, 100, 1, 10)
//!     .with_kind("UserList")
//!     .with_links(
//!         Some("http://api.example.com/users?page=2".to_string()),
//!         Some("http://api.example.com/users?page=0".to_string())
//!     );
//!
//! // 简单列表响应（无分页信息）
//! let response = ApiResponse::simple_list(tags)
//!     .with_kind("TagList");
//!
//! // 错误响应
//! let response = ApiResponse::not_found(Domain::Auth, Reason::UserNotFound);
//! let response = ApiResponse::unauthorized(Reason::InvalidToken);
//!
//! // 自定义错误消息
//! let response = ApiResponse::fail_with_message(
//!     StatusCode::BAD_REQUEST,
//!     Domain::Validation,
//!     Reason::InvalidParameter,
//!     "用户名长度必须在 3-20 个字符之间"
//! );
//! ```

mod api_response;
mod domain;
mod error;
mod reason;
mod status;

pub use api_response::{API_VERSION, ApiResponse, DataContent, DataWrapper};
pub use domain::Domain;
pub use error::{ApiError, ErrorDetail};
pub use reason::Reason;
pub use status::Status;
