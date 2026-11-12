//! 游标分页契约测试。
//!
//! 覆盖参数默认值与边界、游标编解码，以及“多取一行”判断下一页的逻辑。

use app::shared::pagination::{
    CursorPagination, CursorQuery, DEFAULT_LIMIT, MAX_LIMIT, decode_cursor, encode_cursor,
};

fn pagination(limit: Option<u64>, cursor: Option<&str>) -> CursorPagination {
    CursorPagination::from_query(&CursorQuery {
        limit,
        cursor: cursor.map(str::to_string),
    })
    .unwrap()
}

#[test]
fn defaults_to_first_page() {
    let pagination = pagination(None, None);

    assert_eq!(pagination.limit, DEFAULT_LIMIT);
    assert_eq!(pagination.fetch_limit(), DEFAULT_LIMIT + 1);
    assert_eq!(pagination.after::<i32>().unwrap(), None);
}

#[test]
fn rejects_limit_out_of_range() {
    for limit in [0, MAX_LIMIT + 1] {
        let result = CursorPagination::from_query(&CursorQuery {
            limit: Some(limit),
            cursor: None,
        });

        assert!(result.is_err(), "limit={limit} 应该被拒绝");
    }
}

#[test]
fn cursor_round_trips_keys() {
    assert_eq!(decode_cursor::<i32>(&encode_cursor(&42)).unwrap(), 42);

    let key = ("2026-01-01T00:00:00Z".to_string(), 7_i64);
    assert_eq!(
        decode_cursor::<(String, i64)>(&encode_cursor(&key)).unwrap(),
        key
    );
}

#[test]
fn malformed_cursor_is_a_validation_error() {
    // 不是 base64
    assert!(pagination(None, Some("!!!")).after::<i32>().is_err());
    // 是合法游标，但键的类型不对
    let text_cursor = encode_cursor(&"abc");
    assert!(pagination(None, Some(&text_cursor)).after::<i32>().is_err());
}

#[test]
fn empty_cursor_means_first_page() {
    assert_eq!(pagination(None, Some("")).after::<i32>().unwrap(), None);
}

#[test]
fn extra_row_becomes_next_cursor() {
    let pagination = pagination(Some(2), None);

    // 查询多取了一行：说明还有下一页，游标指向本页最后一行
    let page = pagination.into_page(vec![9, 8, 7], |id| *id);
    assert_eq!(page.items, vec![9, 8]);
    let next = page.next_cursor.expect("应该有下一页");
    assert_eq!(decode_cursor::<i32>(&next).unwrap(), 8);

    // 行数不超过一页：已经是最后一页
    let last = pagination.into_page(vec![7, 6], |id| *id);
    assert_eq!(last.items, vec![7, 6]);
    assert_eq!(last.next_cursor, None);

    let empty = pagination.into_page(Vec::<i32>::new(), |id| *id);
    assert!(empty.items.is_empty());
    assert_eq!(empty.next_cursor, None);
}

#[test]
fn page_can_be_mapped_to_another_type() {
    let page = pagination(Some(1), None)
        .into_page(vec![2, 1], |id| *id)
        .map(|id| id.to_string());

    assert_eq!(page.items, vec!["2"]);
    assert!(page.next_cursor.is_some());
}
