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
        commands::mcp_config::load_mcp_config,
        commands::mcp_config::save_mcp_config,
        commands::mcp_config::reset_mcp_config,
        commands::mcp_config::get_mcp_config_path_string,
        commands::mcp::mcp_list_servers,
        commands::mcp::mcp_add_server,
        commands::mcp::mcp_update_server,
        commands::mcp::mcp_remove_server,
        commands::mcp::mcp_connect,
        commands::mcp::mcp_disconnect,
        commands::mcp::mcp_status,
        commands::mcp::mcp_tools_list,
        commands::mcp::mcp_tools_call,
        commands::mcp::mcp_resources_list,
        commands::mcp::mcp_resources_read,
        commands::llama_cpp::start_llama_server,
        commands::llama_cpp::stop_llama_server,
        commands::llama_cpp::check_server_health,
        commands::llama_cpp::is_server_running,
        commands::llama_cpp::get_llama_config,
        commands::llama_cpp::get_server_metrics,
        commands::chat::load_history_context,
    ])
}
