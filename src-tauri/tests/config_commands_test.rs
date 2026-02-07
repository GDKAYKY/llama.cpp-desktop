use llama_desktop_lib::commands::config::{
    build_config_file_path_from_dir, get_config_from_path, reset_config_at_path,
    save_config_to_path,
};
use llama_desktop_lib::models::AppConfig;
use tempfile::tempdir;

#[test]
fn build_config_file_path_from_dir_appends_filename() {
    let dir = tempdir().expect("tempdir");
    let path = build_config_file_path_from_dir(dir.path().to_path_buf());
    assert!(path.ends_with("config.json"));
}

#[test]
fn get_config_from_path_returns_default_when_missing() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("config.json");
    let config = get_config_from_path(&path).expect("default");
    assert_eq!(config.theme, AppConfig::default().theme);

    // Continue with remaining fields of AppConfig comparing with AppConfig::default()
}

#[test]
fn save_and_load_config_round_trip() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("config.json");
    let mut config = AppConfig::default();
    config.theme = "light".to_string();
    save_config_to_path(&path, &config).expect("save");

    let loaded = get_config_from_path(&path).expect("load");
    assert_eq!(loaded.theme, "light");
}

#[test]
fn reset_config_at_path_writes_default() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("config.json");

    let reset = reset_config_at_path(&path).expect("reset");
    assert_eq!(reset, AppConfig::default());
    let loaded = get_config_from_path(&path).expect("load");
    assert_eq!(loaded, AppConfig::default());
}
