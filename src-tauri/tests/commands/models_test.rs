use crate::common;

use llama_desktop_lib::commands::models::{save_model_library, test_utils};

#[tokio::test]
async fn test_scan_models_directory_empty() {
    let dir = common::temp_dir();
    let models =
        test_utils::scan_models_directory_for_test(dir.path().to_str().unwrap().to_string(), "")
            .unwrap();

    assert!(models.is_empty());
}

#[tokio::test]
async fn test_scan_models_directory_with_manifest() {
    let dir = common::temp_dir();
    let manifests_dir = dir.path().join("manifests/test/lib/model/v1");
    std::fs::create_dir_all(&manifests_dir).unwrap();

    let manifest_path = manifests_dir.join("manifest.json");
    let manifest = common::create_test_model_manifest();
    llama_desktop_lib::utils::save_json(&manifest_path, &manifest).unwrap();

    let models =
        test_utils::scan_models_directory_for_test(dir.path().to_str().unwrap().to_string(), "")
            .unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].provider, "test");
}

#[tokio::test]
async fn test_save_and_load_model_library() {
    let dir = common::temp_dir();
    let library_path = dir.path().join("modelLibrary.json");

    let library = vec![common::create_test_model_info()];

    save_model_library(library_path.to_str().unwrap().to_string(), library.clone())
        .await
        .unwrap();
    let loaded =
        test_utils::load_model_library_for_test(library_path.to_str().unwrap().to_string(), "")
            .await
            .unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].full_identifier, "test:model:v1");
}

#[tokio::test]
async fn test_load_model_library_nonexistent() {
    let dir = common::temp_dir();
    let path = dir.path().join("nonexistent.json");

    let result =
        test_utils::load_model_library_for_test(path.to_str().unwrap().to_string(), "").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_save_model_library_creates_dirs() {
    let dir = common::temp_dir();
    let path = dir.path().join("nested/deep/modelLibrary.json");

    let library = vec![];
    save_model_library(path.to_str().unwrap().to_string(), library)
        .await
        .unwrap();

    assert!(path.exists());
}
