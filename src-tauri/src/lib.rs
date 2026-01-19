pub mod commands;
pub mod services;
pub mod state;
pub mod utils;

use state::AppState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::chat::send_message,
            commands::models::parse_model_manifest,
            commands::models::scan_models_directory,
            commands::models::save_model_library,
            commands::models::load_model_library,
            commands::config::load_config,
            commands::config::save_config,
            commands::config::reset_config,
            commands::config::get_config_path_string,
            commands::llama_cpp::init_llama,
            commands::llama_cpp::shutdown_llama,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
