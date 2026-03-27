use std::fs;
use std::path::Path;
use tauri::command;
use tauri::AppHandle;

use crate::models::{ModelInfo, ModelLibrary, ModelManifest};
use futures::StreamExt;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::Instant;
use tauri::Emitter;
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Serialize)]
pub struct DownloadProgressPayload {
    pub reference: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed: f64,
}
const DEFAULT_REGISTRY: &str = "registry.ollama.ai";
const DEFAULT_LIBRARY: &str = "library";
const DEFAULT_VERSION: &str = "latest";
const MANIFEST_ACCEPT: &str = "application/vnd.ollama.manifest.v1+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json";
const HF_HOST: &str = "hf.co";
const HF_HOST_LONG: &str = "huggingface.co";
const HF_REVISION_DEFAULT: &str = "main";
const HF_MANIFEST_MEDIA_TYPE: &str = "application/vnd.ollama.manifest.v1+json";
const HF_MODEL_MEDIA_TYPE: &str = "application/vnd.ollama.image.model";
const HF_CONFIG_MEDIA_TYPE: &str = "application/vnd.ollama.image.config";

#[derive(Debug, Clone)]
struct ModelRef {
    registry: String,
    library: String,
    name: String,
    version: String,
}

#[derive(Debug, Clone)]
struct HfModelRef {
    repo_id: String,
    selector: Option<String>,
    revision: String,
    version_label: String,
}

#[derive(Debug, Deserialize)]
struct HfApiModel {
    siblings: Option<Vec<HfApiSibling>>,
}

#[derive(Debug, Deserialize)]
struct HfApiSibling {
    rfilename: String,
}

#[derive(Debug, Serialize)]
struct HfConfigMetadata {
    source: String,
    repo_id: String,
    revision: String,
    filename: String,
}

fn is_registry_host(value: &str) -> bool {
    value.contains('.') || value.contains(':') || value == "localhost"
}

fn is_hf_reference(value: &str) -> bool {
    let value = value.trim();
    value.starts_with("hf.co/")
        || value.starts_with("https://hf.co/")
        || value.starts_with("http://hf.co/")
        || value.starts_with("huggingface.co/")
        || value.starts_with("https://huggingface.co/")
        || value.starts_with("http://huggingface.co/")
}

fn ensure_clean_segment(segment: &str, label: &str) -> Result<(), String> {
    if segment.is_empty() {
        return Err(format!("{} cannot be empty", label));
    }
    if segment.contains('/') || segment.contains('\\') || segment.contains("..") {
        return Err(format!("{} contains invalid path characters", label));
    }
    Ok(())
}

fn split_name_version(value: &str) -> Result<(String, String), String> {
    let mut parts = value.rsplitn(2, ':');
    let version = parts.next().unwrap_or(DEFAULT_VERSION);
    let name = parts.next().unwrap_or("");

    let (name, version) = if name.is_empty() {
        (version.to_string(), DEFAULT_VERSION.to_string())
    } else {
        (name.to_string(), version.to_string())
    };

    ensure_clean_segment(&name, "Model name")?;
    ensure_clean_segment(&version, "Model version")?;

    Ok((name, version))
}

fn parse_model_reference(reference: &str) -> Result<ModelRef, String> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return Err("Model reference cannot be empty".to_string());
    }

    let mut parts: Vec<&str> = trimmed.split('/').collect();
    let mut registry = DEFAULT_REGISTRY.to_string();

    if parts.len() >= 2 && is_registry_host(parts[0]) {
        registry = parts.remove(0).to_string();
    }

    let (library, name_version) = match parts.len() {
        1 => (DEFAULT_LIBRARY.to_string(), parts[0]),
        2 => (parts[0].to_string(), parts[1]),
        _ => {
            return Err(
                "Invalid model reference. Expected registry/library/name:version or name:version"
                    .to_string(),
            );
        }
    };

    ensure_clean_segment(&registry, "Registry")?;
    ensure_clean_segment(&library, "Library")?;

    let (name, version) = split_name_version(name_version)?;

    Ok(ModelRef {
        registry,
        library,
        name,
        version,
    })
}

fn parse_hf_reference(reference: &str) -> Result<HfModelRef, String> {
    let mut trimmed = reference.trim();
    if trimmed.starts_with("https://") {
        trimmed = trimmed.trim_start_matches("https://");
    } else if trimmed.starts_with("http://") {
        trimmed = trimmed.trim_start_matches("http://");
    }

    if trimmed.starts_with("hf.co/") {
        trimmed = trimmed.trim_start_matches("hf.co/");
    } else if trimmed.starts_with("huggingface.co/") {
        trimmed = trimmed.trim_start_matches("huggingface.co/");
    } else {
        return Err(
            "Invalid Hugging Face reference. Expected hf.co/<org>/<repo>:<selector>".to_string(),
        );
    }

    let trimmed = trimmed.trim_start_matches('/');
    if trimmed.is_empty() {
        return Err("Hugging Face reference cannot be empty".to_string());
    }

    let mut parts = trimmed.rsplitn(2, ':');
    let selector = parts.next().unwrap_or("").to_string();
    let repo_id = parts.next().unwrap_or("").to_string();

    let (repo_id, selector) = if repo_id.is_empty() {
        (selector, None)
    } else {
        (repo_id, Some(selector))
    };

    let repo_id = repo_id.trim().to_string();
    if !repo_id.contains('/') {
        return Err("Hugging Face repo must be in the format <org>/<repo>".to_string());
    }

    let version_label = selector
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| HF_REVISION_DEFAULT.to_string());

    Ok(HfModelRef {
        repo_id,
        selector: selector.filter(|s| !s.is_empty()),
        revision: HF_REVISION_DEFAULT.to_string(),
        version_label,
    })
}

fn normalize_selector(value: &str) -> String {
    value.to_lowercase().replace('-', "_")
}

fn choose_hf_filename(files: &[String], selector: Option<&str>) -> Result<String, String> {
    let gguf_files: Vec<String> = files
        .iter()
        .filter(|f| f.to_lowercase().ends_with(".gguf"))
        .cloned()
        .collect();

    if gguf_files.is_empty() {
        return Err("No .gguf files found in the Hugging Face repository".to_string());
    }

    if let Some(selector) = selector {
        if selector.to_lowercase().ends_with(".gguf") {
            if gguf_files.iter().any(|f| f == selector) {
                return Ok(selector.to_string());
            }
            return Err(format!("File {} not found in repository", selector));
        }

        let normalized = normalize_selector(selector);
        let matches: Vec<String> = gguf_files
            .iter()
            .filter(|f| normalize_selector(f).contains(&normalized))
            .cloned()
            .collect();

        return match matches.len() {
            0 => Err("No GGUF files match the requested selector".to_string()),
            1 => Ok(matches[0].clone()),
            _ => Err(
                "Multiple GGUF files match the selector. Please specify the exact filename."
                    .to_string(),
            ),
        };
    }

    if gguf_files.len() == 1 {
        Ok(gguf_files[0].clone())
    } else {
        Err("Multiple GGUF files found. Please specify the exact filename.".to_string())
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

async fn write_blob_from_bytes(blobs_dir: &Path, bytes: &[u8]) -> Result<(String, u64), String> {
    tokio_fs::create_dir_all(blobs_dir)
        .await
        .map_err(|e| format!("Failed to create blobs directory: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = format!("sha256:{}", hex_encode(&hasher.finalize()));
    let blob_filename = digest_to_blob_filename(&digest);
    let blob_path = blobs_dir.join(blob_filename);

    if tokio_fs::metadata(&blob_path).await.is_err() {
        tokio_fs::write(&blob_path, bytes)
            .await
            .map_err(|e| format!("Failed to write blob {}: {}", digest, e))?;
    }

    Ok((digest, bytes.len() as u64))
}

async fn download_hf_file(
    app: &AppHandle,
    reference: &str,
    client: &reqwest::Client,
    url: &str,
    blobs_dir: &Path,
) -> Result<(String, u64), String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download model file: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download model file: HTTP {}",
            response.status()
        ));
    }

    tokio_fs::create_dir_all(blobs_dir)
        .await
        .map_err(|e| format!("Failed to create blobs directory: {}", e))?;

    let tmp_path = blobs_dir.join("hf-download.partial");
    let file = tokio_fs::File::create(&tmp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    let mut writer = tokio::io::BufWriter::with_capacity(1024 * 1024, file);

    let mut hasher = Sha256::new();
    let mut size: u64 = 0;

    let total_size = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();

    let mut last_emit = Instant::now();
    let mut speed_calc_start = Instant::now();
    let mut bytes_since_last_calc = 0u64;

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| format!("Failed while downloading model: {}", e))?;
        let len = bytes.len() as u64;
        size += len;
        bytes_since_last_calc += len;

        hasher.update(&bytes);
        writer
            .write_all(&bytes)
            .await
            .map_err(|e| format!("Failed to write model file: {}", e))?;

        if last_emit.elapsed().as_millis() >= 250 {
            let elapsed_sec = speed_calc_start.elapsed().as_secs_f64();
            let mut speed = 0.0;
            if elapsed_sec > 0.0 {
                speed = (bytes_since_last_calc as f64) / elapsed_sec;
            }

            let _ = app.emit(
                "download-progress",
                DownloadProgressPayload {
                    reference: reference.to_string(),
                    downloaded: size,
                    total: total_size.max(size),
                    speed,
                },
            );

            last_emit = Instant::now();
            speed_calc_start = Instant::now();
            bytes_since_last_calc = 0;
        }
    }

    writer
        .flush()
        .await
        .map_err(|e| format!("Failed to flush model file: {}", e))?;

    let digest = format!("sha256:{}", hex_encode(&hasher.finalize()));
    let blob_filename = digest_to_blob_filename(&digest);
    let blob_path = blobs_dir.join(&blob_filename);

    if tokio_fs::metadata(&blob_path).await.is_err() {
        tokio_fs::rename(&tmp_path, &blob_path)
            .await
            .map_err(|e| format!("Failed to finalize model file: {}", e))?;
    } else {
        tokio_fs::remove_file(&tmp_path)
            .await
            .map_err(|e| format!("Failed to cleanup temp file: {}", e))?;
    }

    Ok((digest, size))
}

async fn download_blob(
    app: &AppHandle,
    reference: &str,
    client: &reqwest::Client,
    registry: &str,
    repository: &str,
    digest: &str,
    size: u64,
    blobs_dir: &Path,
) -> Result<(), String> {
    let blob_filename = digest_to_blob_filename(digest);
    let blob_path = blobs_dir.join(&blob_filename);

    if let Ok(metadata) = tokio_fs::metadata(&blob_path).await {
        if size == 0 || metadata.len() == size {
            return Ok(());
        }
    }

    tokio_fs::create_dir_all(blobs_dir)
        .await
        .map_err(|e| format!("Failed to create blobs directory: {}", e))?;

    let url = format!("https://{}/v2/{}/blobs/{}", registry, repository, digest);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download blob {}: {}", digest, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download blob {}: HTTP {}",
            digest,
            response.status()
        ));
    }

    let tmp_path = blob_path.with_extension("partial");
    let file = tokio_fs::File::create(&tmp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    let mut writer = tokio::io::BufWriter::with_capacity(1024 * 1024, file);

    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;

    let mut last_emit = Instant::now();
    let mut speed_calc_start = Instant::now();
    let mut bytes_since_last_calc = 0u64;

    while let Some(chunk) = stream.next().await {
        let bytes =
            chunk.map_err(|e| format!("Failed while downloading blob {}: {}", digest, e))?;
        let len = bytes.len() as u64;
        downloaded += len;
        bytes_since_last_calc += len;

        writer
            .write_all(&bytes)
            .await
            .map_err(|e| format!("Failed to write blob {}: {}", digest, e))?;

        if last_emit.elapsed().as_millis() >= 250 {
            let elapsed_sec = speed_calc_start.elapsed().as_secs_f64();
            let mut speed = 0.0;
            if elapsed_sec > 0.0 {
                speed = (bytes_since_last_calc as f64) / elapsed_sec;
            }

            let _ = app.emit(
                "download-progress",
                DownloadProgressPayload {
                    reference: reference.to_string(),
                    downloaded,
                    total: size.max(downloaded),
                    speed,
                },
            );

            last_emit = Instant::now();
            speed_calc_start = Instant::now();
            bytes_since_last_calc = 0;
        }
    }

    writer
        .flush()
        .await
        .map_err(|e| format!("Failed to flush blob {}: {}", digest, e))?;

    if size > 0 {
        let metadata = tokio_fs::metadata(&tmp_path)
            .await
            .map_err(|e| format!("Failed to verify blob {}: {}", digest, e))?;
        if metadata.len() != size {
            return Err(format!(
                "Downloaded blob {} size mismatch (expected {}, got {})",
                digest,
                size,
                metadata.len()
            ));
        }
    }

    if tokio_fs::metadata(&blob_path).await.is_ok() {
        tokio_fs::remove_file(&blob_path)
            .await
            .map_err(|e| format!("Failed to replace existing blob: {}", e))?;
    }

    tokio_fs::rename(&tmp_path, &blob_path)
        .await
        .map_err(|e| format!("Failed to finalize blob {}: {}", digest, e))?;

    Ok(())
}

async fn download_model_from_hf(
    app: AppHandle,
    model_reference: String,
    models_root: String,
) -> Result<ModelInfo, String> {
    let model_ref = parse_hf_reference(&model_reference)?;

    let client = reqwest::Client::builder()
        .user_agent("llama-desktop")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let api_url = format!("https://{}/api/models/{}", HF_HOST_LONG, model_ref.repo_id);
    let api_response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("Failed to query Hugging Face API: {}", e))?;

    if !api_response.status().is_success() {
        return Err(format!(
            "Failed to query Hugging Face API: HTTP {}",
            api_response.status()
        ));
    }

    let api_model: HfApiModel = api_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Hugging Face response: {}", e))?;

    let files: Vec<String> = api_model
        .siblings
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.rfilename)
        .collect();

    let filename = choose_hf_filename(&files, model_ref.selector.as_deref())?;

    let file_url = format!(
        "https://{}/{}/resolve/{}/{}",
        HF_HOST_LONG, model_ref.repo_id, model_ref.revision, filename
    );

    let blobs_dir = PathBuf::from(&models_root).join("blobs");
    let (model_digest, model_size) =
        download_hf_file(&app, &model_reference, &client, &file_url, &blobs_dir).await?;

    let config_metadata = HfConfigMetadata {
        source: HF_HOST.to_string(),
        repo_id: model_ref.repo_id.clone(),
        revision: model_ref.revision.clone(),
        filename: filename.clone(),
    };

    let config_bytes = serde_json::to_vec(&config_metadata)
        .map_err(|e| format!("Failed to serialize config metadata: {}", e))?;
    let (config_digest, config_size) = write_blob_from_bytes(&blobs_dir, &config_bytes).await?;

    let manifest = ModelManifest {
        schema_version: 2,
        media_type: HF_MANIFEST_MEDIA_TYPE.to_string(),
        config: crate::models::ManifestConfig {
            media_type: HF_CONFIG_MEDIA_TYPE.to_string(),
            digest: config_digest,
            size: config_size,
        },
        layers: vec![crate::models::ManifestLayer {
            media_type: HF_MODEL_MEDIA_TYPE.to_string(),
            digest: model_digest,
            size: model_size,
        }],
    };

    let mut repo_parts = model_ref.repo_id.splitn(2, '/');
    let library = repo_parts.next().unwrap_or_default().to_string();
    let name = repo_parts.next().unwrap_or_default().to_string();
    ensure_clean_segment(&library, "Library")?;
    ensure_clean_segment(&name, "Name")?;

    let manifest_path = PathBuf::from(&models_root)
        .join("manifests")
        .join(HF_HOST)
        .join(&library)
        .join(&name)
        .join(&model_ref.version_label)
        .join("manifest.json");

    crate::utils::save_json(&manifest_path, &manifest)?;

    // Tenta baixar o chat template jinja automaticamente para conveniência do usuário
    let _ = crate::services::templates::ensure_hf_chat_template(
        &app,
        &model_ref.repo_id,
        Some(&model_ref.revision),
    )
    .await;

    let manifest_path_str = manifest_path
        .to_str()
        .ok_or_else(|| "Failed to build manifest path".to_string())?;

    parse_model_manifest_sync(manifest_path_str.to_string(), models_root)
}

/// Parse model path and extract provider, library, name, and version
/// Expected format: {modelsRoot}/manifests/{provider}/{library}/{name}/{version}
/// The format is based on Ollama Models!
pub fn parse_model_path(model_path: &str) -> Result<(String, String, String, String), String> {
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
pub fn digest_to_blob_filename(digest: &str) -> String {
    digest.replace(':', "-")
}

/// Find the model file blob path
pub fn find_model_blob_path(models_root: &Path, digest: &str) -> Option<String> {
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
        manifest_data: manifest,
        model_file_path,
        manifest_path: Some(model_path),
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

pub fn parse_model_manifest_sync(
    model_path: String,
    models_root: String,
) -> Result<ModelInfo, String> {
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
        manifest_data: manifest,
        model_file_path,
        manifest_path: Some(model_path),
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

#[command]
pub async fn download_model_from_registry(
    app: AppHandle,
    model_reference: String,
    models_root: String,
) -> Result<ModelInfo, String> {
    if is_hf_reference(&model_reference) {
        return download_model_from_hf(app, model_reference, models_root).await;
    }

    let model_ref = parse_model_reference(&model_reference)?;
    let repository = format!("{}/{}", model_ref.library, model_ref.name);

    let client = reqwest::Client::builder()
        .user_agent("llama-desktop")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let manifest_url = format!(
        "https://{}/v2/{}/manifests/{}",
        model_ref.registry, repository, model_ref.version
    );

    let manifest_response = client
        .get(&manifest_url)
        .header(USER_AGENT, "llama-desktop")
        .header(ACCEPT, MANIFEST_ACCEPT)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch manifest: {}", e))?;

    if !manifest_response.status().is_success() {
        return Err(format!(
            "Failed to fetch manifest: HTTP {}",
            manifest_response.status()
        ));
    }

    let manifest: ModelManifest = manifest_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse manifest JSON: {}", e))?;

    let manifest_path = PathBuf::from(&models_root)
        .join("manifests")
        .join(&model_ref.registry)
        .join(&model_ref.library)
        .join(&model_ref.name)
        .join(&model_ref.version)
        .join("manifest.json");

    crate::utils::save_json(&manifest_path, &manifest)?;

    let blobs_dir = PathBuf::from(&models_root).join("blobs");

    download_blob(
        &app,
        &model_reference,
        &client,
        &model_ref.registry,
        &repository,
        &manifest.config.digest,
        manifest.config.size,
        &blobs_dir,
    )
    .await?;

    for layer in &manifest.layers {
        download_blob(
            &app,
            &model_reference,
            &client,
            &model_ref.registry,
            &repository,
            &layer.digest,
            layer.size,
            &blobs_dir,
        )
        .await?;
    }

    let manifest_path_str = manifest_path
        .to_str()
        .ok_or_else(|| "Failed to build manifest path".to_string())?;

    parse_model_manifest_sync(manifest_path_str.to_string(), models_root)
}

/// Removes a model by deleting its associated blob files and the manifest file
#[command]
pub async fn remove_model_by_manifest_path(
    manifest_path: String,
    models_root: String,
) -> Result<bool, String> {
    use std::fs;

    // Parse the manifest to get the layers/blob information
    let manifest: ModelManifest = crate::utils::read_json(Path::new(&manifest_path))
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    // Get the models root path
    let models_root_path = Path::new(&models_root);

    // Remove all blob files referenced in the manifest (config + layers)
    let mut all_removed = true;

    // Remove config blob
    let config_blob_path = find_model_blob_path(models_root_path, &manifest.config.digest);
    if let Some(path) = config_blob_path {
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Warning: Failed to remove config blob {}: {}", path, e);
            all_removed = false;
        } else {
            println!("Removed config blob: {}", path);
        }
    }

    // Remove layer blobs
    for layer in &manifest.layers {
        let layer_blob_path = find_model_blob_path(models_root_path, &layer.digest);
        if let Some(path) = layer_blob_path {
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("Warning: Failed to remove layer blob {}: {}", path, e);
                all_removed = false;
            } else {
                println!("Removed layer blob: {}", path);
            }
        }
    }

    // Remove the manifest file itself
    if let Err(e) = fs::remove_file(&manifest_path) {
        eprintln!(
            "Warning: Failed to remove manifest file {}: {}",
            manifest_path, e
        );
        all_removed = false;
    } else {
        println!("Removed manifest file: {}", manifest_path);
    }

    // Attempt to clean up empty directories after removing the files
    cleanup_empty_dirs(Path::new(&manifest_path))?;

    Ok(all_removed)
}

/// Removes a model by its full identifier (e.g., "registry.ollama.ai:llama3:latest")
#[command]
pub async fn remove_model_by_identifier(
    full_identifier: String,
    models_root: String,
) -> Result<bool, String> {
    let models = scan_models_directory(models_root.clone()).await?;
    let model = models
        .into_iter()
        .find(|m| m.full_identifier == full_identifier)
        .ok_or_else(|| format!("Model with identifier {} not found", full_identifier))?;

    if let Some(manifest_path) = model.manifest_path {
        return remove_model_by_manifest_path(manifest_path, models_root).await;
    }

    let manifest_path = PathBuf::from(&models_root)
        .join("manifests")
        .join(&model.provider)
        .join(&model.library)
        .join(&model.name)
        .join(&model.version)
        .join("manifest.json");

    let manifest_path_str = manifest_path
        .to_str()
        .ok_or_else(|| "Failed to build manifest path".to_string())?;

    remove_model_by_manifest_path(manifest_path_str.to_string(), models_root).await
}

/// Attempts to remove empty directories up to the manifest file
fn cleanup_empty_dirs(file_path: &Path) -> Result<(), String> {
    // Navigate up from the manifest file to clean up empty directories
    let mut current_path = file_path.parent();

    while let Some(path) = current_path {
        // Stop cleaning up if we've reached the "manifests" directory or beyond
        if path.file_name().map_or(false, |name| name == "manifests") {
            break;
        }

        // Check if directory is empty before attempting to remove
        if let Ok(entries) = fs::read_dir(path) {
            if entries.count() == 0 {
                if let Err(e) = fs::remove_dir(path) {
                    eprintln!(
                        "Warning: Could not remove empty directory {:?}: {}",
                        path, e
                    );
                    // If we can't remove parent, stop going up the chain
                    break;
                } else {
                    println!("Removed empty directory: {:?}", path);
                }
            } else {
                // Directory is not empty, stop going up the chain
                break;
            }
        }

        current_path = path.parent();
    }

    Ok(())
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
