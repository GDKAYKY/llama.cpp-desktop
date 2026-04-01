use crate::common;

use llama_desktop_lib::services::llama::service::LlamaCppService;
use llama_desktop_lib::models::ModelLibrary;

#[tokio::test]
async fn test_service_loads_existing_registry() {
    let dir = common::temp_dir();
    let registry_path = dir.path().join("modelLibrary.json");
    
    let library = ModelLibrary {
        models: vec![common::create_test_model_info()],
    };
    llama_desktop_lib::utils::save_json(&registry_path, &library).unwrap();
    
    let service = LlamaCppService::new(dir.path().to_path_buf());
    
    // Service should load the registry on initialization
    // We can't directly verify internal state, but we can verify it doesn't panic
    assert!(service.get_config().await.is_none());
}

#[tokio::test]
async fn test_service_handles_missing_registry() {
    let dir = common::temp_dir();
    let service = LlamaCppService::new(dir.path().to_path_buf());
    
    // Should not panic with missing registry
    assert!(service.get_config().await.is_none());
}

#[tokio::test]
async fn test_service_handles_invalid_registry() {
    let dir = common::temp_dir();
    let registry_path = dir.path().join("modelLibrary.json");
    std::fs::write(&registry_path, "{invalid json}").unwrap();
    
    let service = LlamaCppService::new(dir.path().to_path_buf());
    
    // Should handle invalid JSON gracefully
    assert!(service.get_config().await.is_none());
}
