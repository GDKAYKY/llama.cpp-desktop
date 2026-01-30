use llama_desktop_lib::commands::models::{
    load_model_library, parse_model_manifest, save_model_library,
};
use llama_desktop_lib::models::{ManifestConfig, ManifestLayer, ModelInfo, ModelManifest};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_model_manifest_parsing() {
    let testname = "test_model_manifest_parsing";

    // 1. Setup a temporary directory for our test environment
    let dir = tempdir().expect("Failed to create temp dir");
    let root_path = dir.path();

    // 2. Create the Ollama-like directory structure
    let manifest_dir = root_path.join("manifests/registry.ollama.ai/library/llama3");
    fs::create_dir_all(&manifest_dir).expect("Failed to create manifest dir");
    let manifest_path = manifest_dir.join("latest");

    // 3. Create the blobs directory
    let blobs_dir = root_path.join("blobs");
    fs::create_dir_all(&blobs_dir).expect("Failed to create blobs dir");

    // 4. Mock manifest content
    let model_digest = "sha256:60e05f212f026038317a94420e7f41530777085750d5e1f7bd8cc5961d1d86d5";
    let blob_filename = "sha256-60e05f212f026038317a94420e7f41530777085750d5e1f7bd8cc5961d1d86d5";
    let blob_path = blobs_dir.join(blob_filename);

    // Create a dummy blob file
    fs::write(&blob_path, "dummy model data").expect("Failed to write blob file");

    let manifest_json = format!(
        r#"{{
        "schemaVersion": 2,
        "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
        "config": {{
            "mediaType": "application/vnd.docker.container.image.v1+json",
            "digest": "sha256:dcee128a1ea55610667e4529241517409254b1f6920f666b6c0e86b24508e6f0",
            "size": 3942
        }},
        "layers": [
            {{
                "mediaType": "application/vnd.ollama.image.model",
                "digest": "{}",
                "size": 4661196144
            }}
        ]
    }}"#,
        model_digest
    );

    fs::write(&manifest_path, manifest_json).expect("Failed to write manifest file");

    // 5. Test parse_model_manifest
    let model_info = parse_model_manifest(
        manifest_path.to_str().unwrap().to_string(),
        root_path.to_str().unwrap().to_string(),
    )
    .await
    .expect("Failed to parse model manifest");

    // Verify parsed data
    assert_eq!(model_info.provider, "registry.ollama.ai");
    assert_eq!(model_info.library, "library");
    assert_eq!(model_info.name, "llama3");
    assert_eq!(model_info.version, "latest");
    assert_eq!(
        model_info.full_identifier,
        "registry.ollama.ai:llama3:latest"
    );
    assert!(model_info.model_file_path.is_some());
    assert!(model_info
        .model_file_path
        .as_ref()
        .unwrap()
        .contains(blob_filename));

    println!("\x1b[30;42m SUCESS \x1b[0m {}", testname);
}

#[tokio::test]
async fn test_model_library_persistence() {
    let testname = "test_model_library_persistence";

    let dir = tempdir().expect("Failed to create temp dir");
    let library_json_path = dir.path().join("ModelLibrary.json");

    // Mock ModelInfo
    let model_info = ModelInfo {
        provider: "test-provider".to_string(),
        library: "test-library".to_string(),
        name: "test-model".to_string(),
        version: "1.0".to_string(),
        full_identifier: "test-provider:test-model:1.0".to_string(),
        model_file_path: Some("/path/to/model".to_string()),
        manifest: ModelManifest {
            schema_version: 2,
            media_type: "test".to_string(),
            config: ManifestConfig {
                media_type: "test".to_string(),
                digest: "test-digest".to_string(),
                size: 100,
            },
            layers: vec![ManifestLayer {
                media_type: "test".to_string(),
                digest: "test-layer-digest".to_string(),
                size: 200,
            }],
        },
    };

    let models = vec![model_info];

    // Test save
    save_model_library(
        library_json_path.to_str().unwrap().to_string(),
        models.clone(),
    )
    .await
    .expect("Failed to save model library");

    assert!(library_json_path.exists());

    // Test load
    let loaded_models = load_model_library(library_json_path.to_str().unwrap().to_string())
        .await
        .expect("Failed to load model library");

    assert_eq!(loaded_models.len(), 1);
    assert_eq!(
        loaded_models[0].full_identifier,
        "test-provider:test-model:1.0"
    );

    println!("\x1b[30;42m SUCESS \x1b[0m {}", testname);
}
