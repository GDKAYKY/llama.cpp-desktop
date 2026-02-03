use crate::commands;
use tauri::{Builder, Wry};

/// IPC (Inter-Process Communication) Configuration
///
/// This module handles the registration of Tauri commands, which serve as the
/// binding layer between the frontend (JavaScript/TypeScript) and the backend (Rust).
/// These commands allow the UI to invoke Rust functions asynchronously.

pub fn configure_ipc(builder: Builder<Wry>) -> Builder<Wry> {
    builder.invoke_handler(tauri::generate_handler![
        commands::general::greet,
        commands::chat::send_message,
        commands::chat::clear_chat,
        commands::models::parse_model_manifest,
        commands::models::scan_models_directory,
        commands::models::save_model_library,
        commands::models::load_model_library,
        commands::config::load_config,
        commands::config::save_config,
        commands::config::reset_config,
        commands::config::get_config_path_string,
        commands::llama_cpp::start_llama_server,
        commands::llama_cpp::stop_llama_server,
        commands::llama_cpp::check_server_health,
        commands::llama_cpp::is_server_running,
        commands::llama_cpp::get_llama_config,
        commands::llama_cpp::get_server_metrics,
        commands::chat::load_history_context,
    ])
}
