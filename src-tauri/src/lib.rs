pub mod commands;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

use state::AppState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config = commands::config::get_config(app.handle()).unwrap_or_else(|e| {
                println!("Failed to load config: {}", e);
                crate::models::AppConfig::default()
            });

            let models_path = config
                .models_directory
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("E:\\models"));

            app.manage(AppState::new(models_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
