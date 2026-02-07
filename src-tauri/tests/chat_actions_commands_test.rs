use llama_desktop_lib::commands::chat_actions::build_share_file_name;

#[test]
fn build_share_file_name_includes_session_and_index() {
    let name = build_share_file_name("session", 3);
    assert!(name.starts_with("share_session_3_"));
    assert!(name.ends_with(".md"));
}
