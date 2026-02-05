pub mod commands {
    pub mod chat;
    pub mod chat_actions;
    pub mod config;
    pub mod general;
    pub mod llama_cpp;
    pub mod mcp;
    pub mod mcp_config;
    pub mod models;
}

pub mod infrastructure {
    pub mod llama {
        pub mod process;
        pub mod server;
    }
    pub mod metrics;
    pub mod nvidia_smi;
}

pub mod ipc_handlers;

pub mod models {
    pub mod app_config;
    pub mod chat;
    pub mod llama;
    pub mod mcp;
    pub mod manifest;

    pub use app_config::*;
    pub use chat::*;
    pub use llama::*;
    pub use mcp::*;
    pub use manifest::*;
}

pub mod services {
    pub mod llama {
        pub mod actor;
        pub mod service;
        
        pub use actor::ActorMessage;
        pub use service::LlamaCppService;
    }
    pub mod mcp {
        pub mod client;
        pub mod protocol;
        pub mod service;
        
        pub use service::McpService;
    }
    pub mod orchestrator;
}

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
            let mcp_config =
                commands::mcp_config::load_mcp_config_file(app.handle()).unwrap_or_else(|e| {
                    println!("Failed to load MCP config: {}", e);
                    crate::models::McpConfig::default()
                });

            let models_path = config
                .models_directory
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("E:\\models"));

            app.manage(AppState::new(models_path, mcp_config));
            Ok(())
        });

    ipc_handlers::configure_ipc(builder)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
