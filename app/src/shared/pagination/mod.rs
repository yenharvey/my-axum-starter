//! 列表接口的分页
//!
//! 两种方式并存，按场景选择：
//!
//! | | 页码分页（[`Pagination`]） | 游标分页（[`CursorPagination`]） |
//! | --- | --- | --- |
//! | 请求参数 | `?page=1&page_size=20` | `?limit=20&cursor=<cursor>` |
//! | 跳到任意页、显示总数 | 支持 | 不支持 |
//! | 翻到很深的页 | 越深越慢（OFFSET） | 速度恒定 |
//! | 翻页期间有新数据 | 可能重复或遗漏 | 不受影响 |
//! | 适合 | 后台管理表格 | 信息流、无限滚动、大表 |
//!
//! 两者都是 axum 提取器：处理器参数里直接写 `pagination: Pagination`
//! 或 `pagination: CursorPagination`，参数校验失败时返回统一格式的 400。

mod cursor;
mod page;

pub use cursor::{
    CursorPage, CursorPagination, CursorQuery, DEFAULT_LIMIT, MAX_LIMIT, decode_cursor,
    encode_cursor,
};
pub use page::{
    DEFAULT_PAGE, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, MIN_PAGE_SIZE, Pagination, PaginationQuery,
};
