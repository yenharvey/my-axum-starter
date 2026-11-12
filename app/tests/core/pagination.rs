//! 分页契约测试。
//!
//! 覆盖列表接口复用的默认值和边界校验。

use app::shared::pagination::{
    DEFAULT_PAGE, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, Pagination, PaginationQuery,
};

#[test]
fn defaults_page_and_page_size() {
    let pagination = Pagination::from_query(&PaginationQuery::default()).unwrap();

    assert_eq!(pagination.page, DEFAULT_PAGE);
    assert_eq!(pagination.page_size, DEFAULT_PAGE_SIZE);
    assert_eq!(pagination.zero_based_page(), 0);
}

#[test]
fn rejects_zero_page() {
    let err = Pagination::from_query(&PaginationQuery {
        page: Some(0),
        page_size: None,
    })
    .unwrap_err();

    assert!(err.to_string().contains("page 必须大于等于 1"));
}

#[test]
fn rejects_page_size_above_maximum() {
    let err = Pagination::from_query(&PaginationQuery {
        page: None,
        page_size: Some(MAX_PAGE_SIZE + 1),
    })
    .unwrap_err();

    assert!(err.to_string().contains("page_size 不能超过 100"));
}

#[test]
fn rejects_page_size_below_minimum() {
    let err = Pagination::from_query(&PaginationQuery {
        page: None,
        page_size: Some(0),
    })
    .unwrap_err();

    assert!(err.to_string().contains("page_size 必须大于等于 1"));
}
