use std::fs;
use std::path::Path;
use tauri::command;

use crate::models::{ModelInfo, ModelLibrary, ModelManifest};

/// Parse model path and extract provider, library, name, and version
/// Expected format: {modelsRoot}/manifests/{provider}/{library}/{name}/{version}
/// The format is based on Ollama Models!
fn parse_model_path(model_path: &str) -> Result<(String, String, String, String), String> {
    let path = Path::new(model_path);

    // Get the path components
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Find "manifests" in the path
    let manifests_idx = components
        .iter()
        .position(|&c| c == "manifests")
        .ok_or_else(|| "Path must contain 'manifests' folder".to_string())?;

    // Extract provider, library, name, version after "manifests"
    if components.len() < manifests_idx + 5 {
        return Err(
            "Invalid path structure. Expected: .../manifests/{provider}/{library}/{name}/{version}"
                .to_string(),
        );
    }

    let provider = components[manifests_idx + 1].to_string();
    let library = components[manifests_idx + 2].to_string();
    let name = components[manifests_idx + 3].to_string();
    let version = components[manifests_idx + 4].to_string();

    Ok((provider, library, name, version))
}

/// Convert digest to blob file name format
/// sha256:60e05f2... -> sha256-60e05f2...
fn digest_to_blob_filename(digest: &str) -> String {
    digest.replace(':', "-")
}

/// Find the model file blob path
fn find_model_blob_path(models_root: &Path, digest: &str) -> Option<String> {
    let blob_filename = digest_to_blob_filename(digest);
    let blob_path = models_root.join("blobs").join(&blob_filename);

    if blob_path.exists() {
        blob_path.to_str().map(|s| s.to_string())
    } else {
        None
    }
}

#[command]
pub async fn parse_model_manifest(
    model_path: String,
    models_root: String,
) -> Result<ModelInfo, String> {
    // Parse the path to extract components
    let (provider, library, name, version) = parse_model_path(&model_path)?;

    // Read and parse manifest file
    let manifest: ModelManifest = crate::utils::read_json(Path::new(&model_path))?;

    // Find the model file (first layer with model mediaType)
    let model_layer = manifest
        .layers
        .iter()
        .find(|layer| layer.media_type.contains("ollama.image.model"))
        .ok_or_else(|| "No model layer found in manifest".to_string())?;

    // Find the blob file path
    let models_root_path = Path::new(&models_root);
    let model_file_path = find_model_blob_path(models_root_path, &model_layer.digest);

    // Create full identifier
    let full_identifier = format!("{}:{}:{}", provider, name, version);

    Ok(ModelInfo {
        provider: provider.clone(),
        library,
        name: name.clone(),
        version: version.clone(),
        manifest,
        model_file_path,
        full_identifier,
    })
}

fn process_version_entry(
    version_entry: fs::DirEntry,
    models_root: &str,
    models: &mut Vec<ModelInfo>,
) {
    let manifest_path = version_entry.path();
    let candidate = if manifest_path.is_file() {
        Some(manifest_path)
    } else if manifest_path.is_dir() {
        let manifest_file = manifest_path.join("manifest.json");
        if manifest_file.is_file() {
            Some(manifest_file)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(path) = candidate {
        if let Some(path_str) = path.to_str() {
            match parse_model_manifest_sync(path_str.to_string(), models_root.to_string()) {
                Ok(model_info) => models.push(model_info),
                Err(e) => eprintln!("Error parsing {}: {}", path_str, e),
            }
        }
    }
}

fn process_model_entry(model_entry: fs::DirEntry, models_root: &str, models: &mut Vec<ModelInfo>) {
    if let Ok(versions) = fs::read_dir(model_entry.path()) {
        for version_entry in versions.flatten() {
            process_version_entry(version_entry, models_root, models);
        }
    }
}

fn process_library_entry(
    library_entry: fs::DirEntry,
    models_root: &str,
    models: &mut Vec<ModelInfo>,
) {
    if let Ok(model_names) = fs::read_dir(library_entry.path()) {
        for model_entry in model_names.flatten() {
            process_model_entry(model_entry, models_root, models);
        }
    }
}

fn process_provider_entry(
    provider_entry: fs::DirEntry,
    models_root: &str,
    models: &mut Vec<ModelInfo>,
) {
    if let Ok(libraries) = fs::read_dir(provider_entry.path()) {
        for library_entry in libraries.flatten() {
            process_library_entry(library_entry, models_root, models);
        }
    }
}

fn parse_model_manifest_sync(model_path: String, models_root: String) -> Result<ModelInfo, String> {
    let (provider, library, name, version) = parse_model_path(&model_path)?;
    let manifest: ModelManifest = crate::utils::read_json(Path::new(&model_path))?;

    let model_layer = manifest
        .layers
        .iter()
        .find(|layer| layer.media_type.contains("ollama.image.model"))
        .ok_or_else(|| "No model layer found in manifest".to_string())?;

    let models_root_path = Path::new(&models_root);
    let model_file_path = find_model_blob_path(models_root_path, &model_layer.digest);
    let full_identifier = format!("{}:{}:{}", provider, name, version);

    Ok(ModelInfo {
        provider: provider.clone(),
        library,
        name: name.clone(),
        version: version.clone(),
        manifest,
        model_file_path,
        full_identifier,
    })
}

#[command]
pub async fn scan_models_directory(models_root: String) -> Result<Vec<ModelInfo>, String> {
    let manifests_path = Path::new(&models_root).join("manifests");

    if !manifests_path.exists() {
        return Err("Manifests directory not found".to_string());
    }

    let mut models = Vec::new();

    if let Ok(providers) = fs::read_dir(&manifests_path) {
        for provider_entry in providers.flatten() {
            process_provider_entry(provider_entry, &models_root, &mut models);
        }
    }

    Ok(models)
}

#[command]
pub async fn save_model_library(
    library_path: String,
    models: Vec<ModelInfo>,
) -> Result<(), String> {
    let library = ModelLibrary { models };

    crate::utils::save_json(Path::new(&library_path), &library)?;

    Ok(())
}

#[command]
pub async fn load_model_library(library_path: String) -> Result<Vec<ModelInfo>, String> {
    if !Path::new(&library_path).exists() {
        return Ok(Vec::new());
    }

    let library: ModelLibrary = crate::utils::read_json(Path::new(&library_path))?;

    Ok(library.models)
}

#[cfg(any(test, debug_assertions))]
pub mod test_utils {
    use super::*;

    pub fn parse_model_path_for_test(
        model_path: &str,
    ) -> Result<(String, String, String, String), String> {
        parse_model_path(model_path)
    }

    pub fn digest_to_blob_filename_for_test(digest: &str) -> String {
        digest_to_blob_filename(digest)
    }

    pub fn find_model_blob_path_for_test(models_root: &Path, digest: &str) -> Option<String> {
        find_model_blob_path(models_root, digest)
    }

    pub fn parse_model_manifest_sync_for_test(
        model_path: String,
        models_root: String,
    ) -> Result<ModelInfo, String> {
        parse_model_manifest_sync(model_path, models_root)
    }
}
