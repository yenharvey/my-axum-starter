//! 游标分页：`?limit=20&cursor=<cursor>`
//!
//! 游标是上一页最后一行的排序键（如主键 id）经 JSON + base64url 编码后的不透明字符串。
//! 查询时用 `WHERE 排序键 < 游标` 代替 `OFFSET`，因此翻到任意深度速度都一样。
//!
//! 用法（按 id 倒序）：
//!
//! ```ignore
//! let mut query = Entity::find()
//!     .order_by_desc(Column::Id)
//!     .limit(pagination.fetch_limit());
//! if let Some(after) = pagination.after::<i32>()? {
//!     query = query.filter(Column::Id.lt(after));
//! }
//! let page = pagination.into_page(query.all(db).await?, |row| row.id);
//! ```
//!
//! 排序键必须唯一且与 `ORDER BY` 一致；按非唯一列（如创建时间）排序时，
//! 用 `(created_at, id)` 这样的元组作为游标键。
//!
//! 游标没有签名：客户端可以自行构造，后果只是从列表的另一个位置开始读，
//! 因此不要把游标当作权限凭据。

use aide::OperationInput;
use aide::generate::GenContext;
use aide::openapi::Operation;
use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::shared::error::ValidationError;

/// 默认每页数量。
pub const DEFAULT_LIMIT: u64 = 20;
/// 最大每页数量，避免列表接口被无上限拉取拖垮。
pub const MAX_LIMIT: u64 = 100;

/// HTTP 查询字符串中的游标分页参数。
///
/// 示例：`?limit=20&cursor=NDI`。
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct CursorQuery {
    /// 每页数据量；未传时使用 `DEFAULT_LIMIT`。
    pub limit: Option<u64>,
    /// 上一页响应里的 `next_cursor`；未传时从第一页开始。
    pub cursor: Option<String>,
}

/// 经过校验后的游标分页参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPagination {
    /// 每页数据量。
    pub limit: u64,
    cursor: Option<String>,
}

impl CursorPagination {
    /// 从原始查询参数构造，并应用默认值和边界校验。
    pub fn from_query(query: &CursorQuery) -> Result<Self, ValidationError> {
        let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
        if limit == 0 {
            return Err(ValidationError::custom("limit 必须大于等于 1"));
        }
        if limit > MAX_LIMIT {
            return Err(ValidationError::custom(format!(
                "limit 不能超过 {MAX_LIMIT}"
            )));
        }

        Ok(Self {
            limit,
            cursor: query.cursor.clone().filter(|cursor| !cursor.is_empty()),
        })
    }

    /// 解码游标里的排序键，即“从这个键之后开始取”；没有游标（第一页）时返回 `None`。
    pub fn after<K: DeserializeOwned>(&self) -> Result<Option<K>, ValidationError> {
        self.cursor.as_deref().map(decode_cursor).transpose()
    }

    /// 查询时应使用的 `LIMIT`：比一页多取一行，用来判断是否还有下一页。
    pub fn fetch_limit(&self) -> u64 {
        self.limit + 1
    }

    /// 把按 [`fetch_limit`](Self::fetch_limit) 查出来的行裁成一页。
    ///
    /// `key` 返回一行的排序键，用于生成下一页的游标。
    pub fn into_page<T, K: Serialize>(
        &self,
        mut rows: Vec<T>,
        key: impl Fn(&T) -> K,
    ) -> CursorPage<T> {
        let limit = usize::try_from(self.limit).unwrap_or(usize::MAX);
        let has_more = rows.len() > limit;
        rows.truncate(limit);

        let next_cursor = rows
            .last()
            .filter(|_| has_more)
            .map(|row| encode_cursor(&key(row)));

        CursorPage {
            items: rows,
            next_cursor,
        }
    }
}

/// 游标分页的一页数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPage<T> {
    /// 当前页的数据项。
    pub items: Vec<T>,
    /// 下一页的游标；已经是最后一页时为 `None`。
    pub next_cursor: Option<String>,
}

impl<T> CursorPage<T> {
    /// 转换数据项的类型（如数据库模型转 DTO），游标保持不变。
    pub fn map<U>(self, f: impl FnMut(T) -> U) -> CursorPage<U> {
        CursorPage {
            items: self.items.into_iter().map(f).collect(),
            next_cursor: self.next_cursor,
        }
    }
}

/// 把排序键编码为游标。
pub fn encode_cursor<K: Serialize>(key: &K) -> String {
    // 排序键是数字、字符串、时间或它们的元组，序列化为 JSON 不会失败
    let json = serde_json::to_vec(key).unwrap_or_default();
    URL_SAFE_NO_PAD.encode(json)
}

/// 把游标解码为排序键；游标被篡改或类型不符时返回校验错误。
pub fn decode_cursor<K: DeserializeOwned>(cursor: &str) -> Result<K, ValidationError> {
    let invalid = || ValidationError::custom("cursor 不是有效的游标");

    let json = URL_SAFE_NO_PAD.decode(cursor).map_err(|_| invalid())?;
    serde_json::from_slice(&json).map_err(|_| invalid())
}

impl<S: Send + Sync> FromRequestParts<S> for CursorPagination {
    type Rejection = ValidationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(query) = Query::<CursorQuery>::from_request_parts(parts, state)
            .await
            .map_err(|rejection| ValidationError::custom(rejection.body_text()))?;

        Self::from_query(&query)
    }
}

impl OperationInput for CursorPagination {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        Query::<CursorQuery>::operation_input(ctx, operation);
    }
}
