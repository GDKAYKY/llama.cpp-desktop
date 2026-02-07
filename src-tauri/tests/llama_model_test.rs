use llama_desktop_lib::models::ModelId;

#[test]
fn model_id_display_uses_inner_string() {
    let id = ModelId("abc/def".to_string());
    assert_eq!(id.to_string(), "abc/def");
}
