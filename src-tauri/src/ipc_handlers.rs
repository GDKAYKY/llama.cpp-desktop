#![cfg(not(test))]

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
        commands::chat_actions::chat_action_like,
        commands::chat_actions::chat_action_dislike,
        commands::chat_actions::chat_action_copy,
        commands::chat_actions::chat_action_share,
        commands::chat_actions::chat_action_regenerate,
        commands::models::parse_model_manifest,
        commands::models::scan_models_directory,
        commands::models::save_model_library,
        commands::models::load_model_library,
        commands::models::download_model_from_registry,
        commands::config::load_config,
        commands::config::save_config,
        commands::config::reset_config,
        commands::config::get_config_path_string,
        commands::mcp_config::load_mcp_config,
        commands::mcp_config::load_default_mcp_config,
        commands::mcp_config::save_mcp_config,
        commands::mcp_config::reset_mcp_config,
        commands::mcp_config::get_mcp_config_path_string,
        commands::mcp::list_servers,
        commands::mcp::add_server,
        commands::mcp::update_server,
        commands::mcp::remove_server,
        commands::mcp::connect,
        commands::mcp::disconnect,
        commands::mcp::status,
        commands::mcp::list_tools,
        commands::mcp::call_tools,
        commands::mcp::list_resources,
        commands::mcp::read_resources,
        commands::mcp::parse_config,
        commands::mcp::import_config,
        commands::mcp::refresh_mcp_capabilities,
        commands::llama_cpp::ensure_chat_template,
        commands::llama_cpp::start_llama_server,
        commands::llama_cpp::stop_llama_server,
        commands::llama_cpp::check_server_health,
        commands::llama_cpp::is_server_running,
        commands::llama_cpp::check_server_health_detail,
        commands::llama_cpp::get_llama_config,
        commands::llama_cpp::get_server_metrics,
        commands::chat::load_history_context,
        commands::chat::generate_chat_title,
    ])
}
