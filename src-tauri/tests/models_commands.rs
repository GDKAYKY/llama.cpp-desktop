use std::fs;
use std::path::Path;

use llama_desktop_lib::commands::models::test_utils::{
    digest_to_blob_filename_for_test, find_model_blob_path_for_test,
    parse_model_manifest_sync_for_test, parse_model_path_for_test,
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

#[test]
fn parse_model_path_valid() {
    let path = Path::new("models")
        .join("manifests")
        .join("provider")
        .join("lib")
        .join("name")
        .join("version")
        .join("manifest.json");
    let (provider, library, name, version) =
        parse_model_path_for_test(path.to_string_lossy().as_ref()).expect("parse");
    assert_eq!(provider, "provider");
    assert_eq!(library, "lib");
    assert_eq!(name, "name");
    assert_eq!(version, "version");
}

#[test]
fn parse_model_path_requires_manifests_segment() {
    let path = Path::new("models")
        .join("provider")
        .join("lib")
        .join("name")
        .join("version");
    let err = parse_model_path_for_test(path.to_string_lossy().as_ref()).expect_err("expected error");
    assert!(err.contains("manifests"));
}

#[test]
fn parse_model_path_requires_full_structure() {
    let path = Path::new("models").join("manifests").join("provider").join("lib");
    let err = parse_model_path_for_test(path.to_string_lossy().as_ref()).expect_err("expected error");
    assert!(err.contains("Invalid path structure"));
}

#[test]
fn digest_to_blob_filename_replaces_colon() {
    let name = digest_to_blob_filename_for_test("sha256:abc");
    assert_eq!(name, "sha256-abc");
}

#[test]
fn find_model_blob_path_returns_path_when_exists() {
    let dir = tempdir().expect("tempdir");
    let blobs_dir = dir.path().join("blobs");
    fs::create_dir_all(&blobs_dir).expect("create blobs");
    let blob_path = blobs_dir.join("sha256-abc123");
    fs::write(&blob_path, "model").expect("write blob");
    let found = find_model_blob_path_for_test(dir.path(), "sha256:abc123");
    assert_eq!(found, blob_path.to_str().map(|s| s.to_string()));
}

#[test]
fn find_model_blob_path_returns_none_when_missing() {
    let dir = tempdir().expect("tempdir");
    let found = find_model_blob_path_for_test(dir.path(), "sha256:missing");
    assert!(found.is_none());
}

#[test]
fn parse_model_manifest_sync_success() {
    let dir = tempdir().expect("tempdir");
    let manifest_dir = dir.path().join("manifests/provider/lib/name/version");
    fs::create_dir_all(&manifest_dir).expect("create manifests");
    let manifest_path = manifest_dir.join("manifest.json");

    let manifest = sample_manifest();
    llama_desktop_lib::utils::save_json(&manifest_path, &manifest).expect("save manifest");

    let blobs_dir = dir.path().join("blobs");
    fs::create_dir_all(&blobs_dir).expect("create blobs");
    let blob_path = blobs_dir.join("sha256-abc123");
    fs::write(&blob_path, "model").expect("write blob");

    let result = parse_model_manifest_sync_for_test(
        manifest_path.to_string_lossy().to_string(),
        dir.path().to_string_lossy().to_string(),
    )
    .expect("parse");
    assert_eq!(result.provider, "provider");
    assert_eq!(result.library, "lib");
    assert_eq!(result.name, "name");
    assert_eq!(result.version, "version");
    assert_eq!(result.full_identifier, "provider:name:version");
    assert_eq!(result.model_file_path, blob_path.to_str().map(|s| s.to_string()));
}

#[test]
fn parse_model_manifest_sync_missing_layer_errors() {
    let dir = tempdir().expect("tempdir");
    let manifest_dir = dir.path().join("manifests/provider/lib/name/version");
    fs::create_dir_all(&manifest_dir).expect("create manifests");
    let manifest_path = manifest_dir.join("manifest.json");

    let mut manifest = sample_manifest();
    manifest.layers = vec![ManifestLayer {
        media_type: "application/vnd.ollama.image.other".to_string(),
        digest: "sha256:nope".to_string(),
        size: 1,
    }];
    llama_desktop_lib::utils::save_json(&manifest_path, &manifest).expect("save manifest");

    let err = parse_model_manifest_sync_for_test(
        manifest_path.to_string_lossy().to_string(),
        dir.path().to_string_lossy().to_string(),
    )
    .expect_err("expected error");
    assert!(err.contains("No model layer"));
}

#[test]
fn parse_model_manifest_sync_handles_missing_blob() {
    let dir = tempdir().expect("tempdir");
    let manifest_dir = dir.path().join("manifests/provider/lib/name/version");
    fs::create_dir_all(&manifest_dir).expect("create manifests");
    let manifest_path = manifest_dir.join("manifest.json");

    let manifest = sample_manifest();
    llama_desktop_lib::utils::save_json(&manifest_path, &manifest).expect("save manifest");

    let result = parse_model_manifest_sync_for_test(
        manifest_path.to_string_lossy().to_string(),
        dir.path().to_string_lossy().to_string(),
    )
    .expect("parse");
    assert!(result.model_file_path.is_none());
}
