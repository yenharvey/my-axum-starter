//! 列表接口的分页查询参数
//!
//! 这不是中间件：它不拦截请求链路，而是处理器使用的 HTTP 查询参数约定。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::shared::error::ValidationError;

/// 默认页码。接口层使用 1-based 页码，和多数前端分页组件保持一致。
pub const DEFAULT_PAGE: u64 = 1;
/// 默认每页数量。
pub const DEFAULT_PAGE_SIZE: u64 = 20;
/// 最小每页数量。
pub const MIN_PAGE_SIZE: u64 = 1;
/// 最大每页数量，避免列表接口被无上限拉取拖垮。
pub const MAX_PAGE_SIZE: u64 = 100;

/// 经过校验后的分页参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    /// 当前页码，从 1 开始。
    pub page: u64,
    /// 每页数据量。
    pub page_size: u64,
}

impl Pagination {
    /// 从原始查询参数构造分页参数，并应用默认值和边界校验。
    pub fn from_query(query: &PaginationQuery) -> Result<Self, ValidationError> {
        let page = query.page.unwrap_or(DEFAULT_PAGE);
        if page == 0 {
            return Err(ValidationError::custom("page 必须大于等于 1"));
        }

        let page_size = query.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page_size < MIN_PAGE_SIZE {
            return Err(ValidationError::custom(format!(
                "page_size 必须大于等于 {MIN_PAGE_SIZE}"
            )));
        }
        if page_size > MAX_PAGE_SIZE {
            return Err(ValidationError::custom(format!(
                "page_size 不能超过 {MAX_PAGE_SIZE}"
            )));
        }

        Ok(Self { page, page_size })
    }

    /// 从 0 开始的页码（SeaORM paginator 使用）
    pub fn zero_based_page(self) -> u64 {
        self.page - 1
    }
}

/// HTTP 查询字符串中的分页参数。
///
/// 示例：`?page=1&page_size=20`。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct PaginationQuery {
    /// 当前页码，从 1 开始；未传时使用 `DEFAULT_PAGE`。
    pub page: Option<u64>,
    /// 每页数据量；未传时使用 `DEFAULT_PAGE_SIZE`。
    pub page_size: Option<u64>,
}
