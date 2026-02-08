use llama_desktop_lib::commands::models::{
    digest_to_blob_filename, find_model_blob_path, parse_model_manifest_sync, parse_model_path,
};
use llama_desktop_lib::models::{ManifestConfig, ManifestLayer, ModelManifest};
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
        layers: vec![ManifestLayer {
            media_type: "application/vnd.ollama.image.model".to_string(),
            digest: "sha256:abc123".to_string(),
            size: 42,
        }],
    }
}

#[test]
fn parse_model_path_errors_when_missing_manifests() {
    let err = parse_model_path("/models/provider/lib/name/version").expect_err("expected error");
    assert!(err.contains("manifests"));
}

#[test]
fn parse_model_path_errors_when_too_short() {
    let err = parse_model_path("/models/manifests/provider/lib").expect_err("expected error");
    assert!(err.contains("Invalid path structure"));
}

#[test]
fn digest_to_blob_filename_rewrites_prefix() {
    assert_eq!(
        digest_to_blob_filename("sha256:60e05f2"),
        "sha256-60e05f2"
    );
}

#[test]
fn find_model_blob_path_returns_path_when_exists() {
    let dir = tempdir().expect("tempdir");
    let blobs_dir = dir.path().join("blobs");
    std::fs::create_dir_all(&blobs_dir).expect("create blobs");
    std::fs::write(blobs_dir.join("sha256-abc123"), "model").expect("write blob");

    let found = find_model_blob_path(dir.path(), "sha256:abc123");
    assert!(found.is_some());
}

#[test]
fn find_model_blob_path_returns_none_when_missing() {
    let dir = tempdir().expect("tempdir");
    let found = find_model_blob_path(dir.path(), "sha256:missing");
    assert!(found.is_none());
}

#[test]
fn parse_model_manifest_sync_errors_without_model_layer() {
    let dir = tempdir().expect("tempdir");
    let manifest_dir = dir
        .path()
        .join("manifests/provider/lib/name/version");
    std::fs::create_dir_all(&manifest_dir).expect("create manifests");
    let manifest_path = manifest_dir.join("manifest.json");

    let mut manifest = sample_manifest();
    manifest.layers[0].media_type = "application/vnd.ollama.image.other".to_string();
    llama_desktop_lib::utils::save_json(&manifest_path, &manifest).expect("save manifest");

    let err = parse_model_manifest_sync(
        manifest_path.to_string_lossy().to_string(),
        dir.path().to_string_lossy().to_string(),
    )
    .expect_err("expected error");
    assert!(err.contains("No model layer"));
}
