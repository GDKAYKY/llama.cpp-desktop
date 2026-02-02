pub mod commands;
pub mod infrastructure;
pub mod ipc_handlers;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

use state::AppState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
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
        });

    ipc_handlers::configure_ipc(builder)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
