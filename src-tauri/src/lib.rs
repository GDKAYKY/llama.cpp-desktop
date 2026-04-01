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

#[cfg(not(test))]
pub mod ipc_handlers;

pub mod models {
    pub mod app_settings_model;
    pub mod chat_model;
    pub mod llama_model;
    pub mod manifest_model;
    pub mod mcp_model;

    pub use app_settings_model::*;
    pub use chat_model::*;
    pub use llama_model::*;
    pub use manifest_model::*;
    pub use mcp_model::*;
}

pub mod services {
    pub mod llama {
        pub mod actor;
        pub mod service;

        pub use actor::ActorMessage;
        pub use service::LlamaCppService;
    }
    pub mod mcp {
        pub mod service;

        pub use service::McpService;
    }
    pub mod capability_registry;
    pub mod orchestrator;
    pub mod subagent;
    pub mod templates;
    pub mod thinking_parser;
}

pub mod state;
pub mod utils;

#[cfg(not(test))]
use state::AppState;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[cfg(not(test))]
use tauri::Manager;

#[cfg(not(test))]
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
            let mcp_config = commands::mcp_config::load_mcp_config_file(app.handle())
                .unwrap_or_else(|e| {
                    println!("Failed to load MCP config: {}", e);
                    crate::models::McpConfig::default()
                });

            let models_path = config
                .models_directory
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("E:\\models"));

            let resource_dir = app.path().resource_dir().ok();
            app.manage(AppState::new(models_path, mcp_config, resource_dir));

            // Hydrate capability registry on startup
            let state = app.state::<AppState>();
            let orchestrator = state.orchestrator.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = orchestrator.refresh_capabilities().await {
                    eprintln!("[Startup] Failed to refresh capabilities: {}", e);
                }
            });

            Ok(())
        });

    ipc_handlers::configure_ipc(builder)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
