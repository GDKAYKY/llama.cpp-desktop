use llama_desktop_lib::commands::models::{
    load_model_library, parse_model_manifest, save_model_library, scan_models_directory,
};
use llama_desktop_lib::models::{ManifestConfig, ManifestLayer, ModelInfo, ModelManifest};
use tempfile::tempdir;

fn sample_manifest() -> ModelManifest {
    ModelManifest {
        schema_version: 2,
        media_type: "application/vnd.ollama.manifest.v1+json".to_string(),
        config: ManifestConfig {
            media_type: "application/vnd.ollama.image.config".to_string(),
            digest: "sha256:config".to_string(),
            size: 10,
        },
        layers: vec![
            ManifestLayer {
                media_type: "application/vnd.ollama.image.model".to_string(),
                digest: "sha256:abc123".to_string(),
                size: 42,
            },
            ManifestLayer {
                media_type: "application/vnd.ollama.image.some".to_string(),
                digest: "sha256:def456".to_string(),
                size: 7,
            },
        ],
    }
}

#[tokio::test]
async fn scan_models_directory_collects_models() {
    let dir = tempdir().expect("tempdir");
    let manifest_dir = dir
        .path()
        .join("manifests/provider/lib/name/version");
    std::fs::create_dir_all(&manifest_dir).expect("create manifests");
    let manifest_path = manifest_dir.join("manifest.json");

    let manifest = sample_manifest();
    llama_desktop_lib::utils::save_json(&manifest_path, &manifest).expect("save manifest");

    let blobs_dir = dir.path().join("blobs");
    std::fs::create_dir_all(&blobs_dir).expect("create blobs");
    std::fs::write(blobs_dir.join("sha256-abc123"), "model").expect("write blob");

    let models = scan_models_directory(dir.path().to_string_lossy().to_string())
        .await
        .expect("scan");
    assert_eq!(models.len(), 1);
}

#[tokio::test]
async fn parse_model_manifest_async_success() {
    let dir = tempdir().expect("tempdir");
    let manifest_dir = dir
        .path()
        .join("manifests/provider/lib/name/version");
    std::fs::create_dir_all(&manifest_dir).expect("create manifests");
    let manifest_path = manifest_dir.join("manifest.json");

    let manifest = sample_manifest();
    llama_desktop_lib::utils::save_json(&manifest_path, &manifest).expect("save manifest");

    let blobs_dir = dir.path().join("blobs");
    std::fs::create_dir_all(&blobs_dir).expect("create blobs");
    std::fs::write(blobs_dir.join("sha256-abc123"), "model").expect("write blob");

    let model = parse_model_manifest(
        manifest_path.to_string_lossy().to_string(),
        dir.path().to_string_lossy().to_string(),
    )
    .await
    .expect("parse");
    assert_eq!(model.provider, "provider");
}

#[tokio::test]
async fn scan_models_directory_requires_manifests_dir() {
    let dir = tempdir().expect("tempdir");
    let err = scan_models_directory(dir.path().to_string_lossy().to_string())
        .await
        .expect_err("expected error");
    assert!(err.contains("Manifests directory not found"));
}

#[tokio::test]
async fn scan_models_directory_skips_invalid_manifest() {
    let dir = tempdir().expect("tempdir");
    let manifest_dir = dir
        .path()
        .join("manifests/provider/lib/name/version");
    std::fs::create_dir_all(&manifest_dir).expect("create manifests");
    let manifest_path = manifest_dir.join("manifest.json");
    std::fs::write(&manifest_path, "{invalid").expect("write bad");

    let models = scan_models_directory(dir.path().to_string_lossy().to_string())
        .await
        .expect("scan");
    assert!(models.is_empty());
}

#[tokio::test]
async fn save_and_load_model_library_round_trip() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("modelLibrary.json");
    let manifest = sample_manifest();
    let model = ModelInfo {
        provider: "provider".to_string(),
        library: "lib".to_string(),
        name: "name".to_string(),
        version: "version".to_string(),
        manifest,
        model_file_path: Some("path/to/model".to_string()),
        full_identifier: "provider:name:version".to_string(),
    };

    save_model_library(path.to_string_lossy().to_string(), vec![model.clone()])
        .await
        .expect("save");
    let loaded = load_model_library(path.to_string_lossy().to_string())
        .await
        .expect("load");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].full_identifier, model.full_identifier);
}

#[tokio::test]
async fn load_model_library_returns_empty_if_missing() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("missing.json");
    let loaded = load_model_library(path.to_string_lossy().to_string())
        .await
        .expect("load");
    assert!(loaded.is_empty());
}
