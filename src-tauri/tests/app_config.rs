use llama_desktop_lib::models::app_config::{AppConfig, ConfigError};
use llama_desktop_lib::utils::{read_json, save_json};
use tempfile::TempDir;

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.theme, "dark");
    assert_eq!(config.language, "en");
    assert_eq!(config.max_tokens, 2048);
    assert_eq!(config.temperature, 0.7);
    assert_eq!(config.auto_save_chat, true);
    assert_eq!(config.chat_history_limit, 50);
    assert_eq!(config.server_port, 8080);
}

#[test]
fn test_app_config_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("config.json");

    let config = AppConfig {
        models_directory: Some("/models".to_string()),
        llama_directory: Some("/llama".to_string()),
        theme: "light".to_string(),
        language: "pt".to_string(),
        max_tokens: 4096,
        temperature: 0.9,
        auto_save_chat: false,
        chat_history_limit: 100,
        server_port: 9090,
    };

    save_json(&path, &config).unwrap();
    let loaded: AppConfig = read_json(&path).unwrap();

    assert_eq!(loaded.theme, "light");
    assert_eq!(loaded.language, "pt");
    assert_eq!(loaded.max_tokens, 4096);
    assert_eq!(loaded.temperature, 0.9);
    assert_eq!(loaded.server_port, 9090);
}

#[test]
fn test_config_error_display() {
    let io_err = ConfigError::Io("test error".to_string());
    assert_eq!(format!("{}", io_err), "IO error: test error");

    let not_found = ConfigError::NotFound;
    assert_eq!(format!("{}", not_found), "Config not found");

    let parse_err = ConfigError::Parse("invalid json".to_string());
    assert_eq!(format!("{}", parse_err), "Parse error: invalid json");
}

#[test]
fn test_app_config_clone() {
    let config = AppConfig::default();
    let cloned = config.clone();
    
    assert_eq!(config.theme, cloned.theme);
    assert_eq!(config.max_tokens, cloned.max_tokens);
}
