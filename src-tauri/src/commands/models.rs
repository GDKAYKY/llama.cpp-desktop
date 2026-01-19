use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestLayer {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestConfig {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: ManifestConfig,
    pub layers: Vec<ManifestLayer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub provider: String,
    pub library: String,
    pub name: String,
    pub version: String,
    pub manifest: ModelManifest,
    pub model_file_path: Option<String>,
    pub full_identifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelLibrary {
    pub models: Vec<ModelInfo>,
}

/// Parse model path and extract provider, library, name, and version
/// Expected format: {modelsRoot}/manifests/{provider}/{library}/{name}/{version}
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

    // Read the manifest file
    let manifest_content = fs::read_to_string(&model_path)
        .map_err(|e| format!("Failed to read manifest file: {}", e))?;

    // Parse JSON
    let manifest: ModelManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest JSON: {}", e))?;

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

#[command]
pub async fn scan_models_directory(models_root: String) -> Result<Vec<ModelInfo>, String> {
    let manifests_path = Path::new(&models_root).join("manifests");

    if !manifests_path.exists() {
        return Err("Manifests directory not found".to_string());
    }

    let mut models = Vec::new();

    // Walk through the manifests directory structure
    if let Ok(providers) = fs::read_dir(&manifests_path) {
        for provider_entry in providers.flatten() {
            if let Ok(libraries) = fs::read_dir(provider_entry.path()) {
                for library_entry in libraries.flatten() {
                    if let Ok(model_names) = fs::read_dir(library_entry.path()) {
                        for model_entry in model_names.flatten() {
                            if let Ok(versions) = fs::read_dir(model_entry.path()) {
                                for version_entry in versions.flatten() {
                                    let manifest_path = version_entry.path();
                                    if manifest_path.is_file() {
                                        if let Some(path_str) = manifest_path.to_str() {
                                            match parse_model_manifest(
                                                path_str.to_string(),
                                                models_root.clone(),
                                            )
                                            .await
                                            {
                                                Ok(model_info) => models.push(model_info),
                                                Err(e) => {
                                                    eprintln!("Error parsing {}: {}", path_str, e)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
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

    let json = serde_json::to_string_pretty(&library)
        .map_err(|e| format!("Failed to serialize model library: {}", e))?;

    fs::write(&library_path, json)
        .map_err(|e| format!("Failed to write model library file: {}", e))?;

    Ok(())
}

#[command]
pub async fn load_model_library(library_path: String) -> Result<Vec<ModelInfo>, String> {
    if !Path::new(&library_path).exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&library_path)
        .map_err(|e| format!("Failed to read model library: {}", e))?;

    let library: ModelLibrary = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse model library: {}", e))?;

    Ok(library.models)
}
