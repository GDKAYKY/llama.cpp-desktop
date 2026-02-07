use llama_desktop_lib::services::mcp::service::{apply_allowlist_by_field, extract_list};
use serde_json::json;

#[test]
fn extract_list_handles_array_and_object() {
    let from_obj = extract_list(&json!({ "tools": [1, 2] }), "tools");
    assert_eq!(from_obj.len(), 2);
    let from_arr = extract_list(&json!([1, 2, 3]), "tools");
    assert_eq!(from_arr.len(), 3);
    let empty = extract_list(&json!({ "nope": [] }), "tools");
    assert!(empty.is_empty());
}

#[test]
fn apply_allowlist_by_field_filters_items() {
    let items = vec![json!({ "name": "alpha" }), json!({ "name": "beta" })];
    let allowed = apply_allowlist_by_field(items, &["beta".to_string()], "name");
    assert_eq!(allowed.len(), 1);
    assert_eq!(allowed[0]["name"], "beta");
}

#[test]
fn apply_allowlist_by_field_allows_all_when_empty() {
    let items = vec![json!({ "name": "alpha" })];
    let allowed = apply_allowlist_by_field(items.clone(), &[], "name");
    assert_eq!(allowed, items);
}
