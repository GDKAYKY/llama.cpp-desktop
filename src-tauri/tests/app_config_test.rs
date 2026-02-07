use llama_desktop_lib::models::{AppConfig, ConfigError};

#[test]
fn default_config_values_are_stable() {
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
fn config_error_display_messages() {
    let io_err = ConfigError::Io("boom".to_string()).to_string();
    assert!(io_err.contains("IO error"));
    let not_found = ConfigError::NotFound.to_string();
    assert_eq!(not_found, "Config not found");
    let parse = ConfigError::Parse("bad".to_string()).to_string();
    assert!(parse.contains("Parse error"));
}
