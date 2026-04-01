use std::fs;
use std::path::Path;
use tauri::command;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager; // required to call .emit() on AppHandle

use crate::models::{ModelInfo, ModelLibrary, ModelManifest};
use futures::StreamExt;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_REGISTRY: &str = "registry.ollama.ai";
const DEFAULT_LIBRARY: &str = "library";
const DEFAULT_VERSION: &str = "latest";

// Multi-value Accept header required by Ollama's registry — order matters,
// OCI manifest format takes priority over Docker v2 schema.
const MANIFEST_ACCEPT: &str = "application/vnd.ollama.manifest.v1+json, \
    application/vnd.oci.image.manifest.v1+json, \
    application/vnd.docker.distribution.manifest.v2+json";

const HF_HOST: &str = "hf.co";
const HF_HOST_LONG: &str = "huggingface.co";

// Default branch used when the user doesn't supply a commit/revision selector.
const HF_REVISION_DEFAULT: &str = "main";

// Media types mimic Ollama's blob conventions so the manifest directory
// stays compatible with ollama CLI tooling.
const HF_MANIFEST_MEDIA_TYPE: &str = "application/vnd.ollama.manifest.v1+json";
const HF_MODEL_MEDIA_TYPE: &str = "application/vnd.ollama.image.model";
const HF_CONFIG_MEDIA_TYPE: &str = "application/vnd.ollama.image.config";

// Chunk size at which download progress events are emitted.
// 1 MiB keeps the event rate reasonable without hiding progress for large models.
const PROGRESS_EMIT_INTERVAL_BYTES: u64 = 1024 * 1024;

// ---------------------------------------------------------------------------
// Internal reference types — never cross the Tauri boundary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct ModelRef {
    registry: String,
    library: String,
    name: String,
    version: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HfModelRef {
    repo_id: String,
    // None means "pick the only GGUF, or fail if ambiguous".
    selector: Option<String>,
    // Actual Git ref used for download URL construction.
    revision: String,
    // Human-readable label used as the manifest version directory name.
    version_label: String,
}

// ---------------------------------------------------------------------------
// HuggingFace API response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct HfApiModel {
    siblings: Option<Vec<HfApiSibling>>,
}

#[derive(Debug, Deserialize)]
struct HfApiSibling {
    rfilename: String,
}

// Stored as the config blob so the runtime knows where a model came from.
#[derive(Debug, Serialize)]
struct HfConfigMetadata {
    source: String,
    repo_id: String,
    revision: String,
    filename: String,
}

// ---------------------------------------------------------------------------
// Reference parsing helpers
// ---------------------------------------------------------------------------

/// Returns true if the first path segment looks like a registry hostname
/// rather than a library name (e.g. "registry.ollama.ai", "localhost:5000").
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

/// Rejects segments that could escape the models directory via path traversal.
fn ensure_clean_segment(segment: &str, label: &str) -> Result<(), String> {
    if segment.is_empty() {
        return Err(format!("{} cannot be empty", label));
    }
    if segment.contains('/') || segment.contains('\\') || segment.contains("..") {
        return Err(format!("{} contains invalid path characters", label));
    }
    Ok(())
}

/// Splits "name:version" using the last colon so names with colons still work.
fn split_name_version(value: &str) -> Result<(String, String), String> {
    let mut parts = value.rsplitn(2, ':');
    let version = parts.next().unwrap_or(DEFAULT_VERSION);
    let name = parts.next().unwrap_or("");

    // rsplitn on a string without ':' yields the whole string as version
    // and an empty name — treat that as "name only, use default version".
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

    // Only the first segment is a registry host; library/name never contain dots.
    if parts.len() >= 2 && is_registry_host(parts[0]) {
        registry = parts.remove(0).to_string();
    }

    let (library, name_version) = match parts.len() {
        1 => (DEFAULT_LIBRARY.to_string(), parts[0]),
        2 => (parts[0].to_string(), parts[1]),
        _ => {
            return Err("Invalid model reference. Expected \
                 registry/library/name:version or name:version"
                .to_string());
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
    // Strip scheme prefix before processing — we don't need it beyond this point.
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
        return Err("Invalid Hugging Face reference. \
             Expected hf.co/<org>/<repo>:<selector>"
            .to_string());
    }

    let trimmed = trimmed.trim_start_matches('/');
    if trimmed.is_empty() {
        return Err("Hugging Face reference cannot be empty".to_string());
    }

    // Split on the last ':' — the right side is an optional file/quant selector.
    let mut parts = trimmed.rsplitn(2, ':');
    let selector = parts.next().unwrap_or("").to_string();
    let repo_id = parts.next().unwrap_or("").to_string();

    let (repo_id, selector) = if repo_id.is_empty() {
        // No ':' found — the whole string is the repo ID, no selector.
        (selector, None)
    } else {
        (repo_id, Some(selector))
    };

    let repo_id = repo_id.trim().to_string();
    if !repo_id.contains('/') {
        return Err("Hugging Face repo must be in the format <org>/<repo>".to_string());
    }

    // If the selector looks like a 40-char hex commit hash, use it as the
    // Git revision so the download URL resolves to that exact commit.
    // Otherwise treat it as a filename/quant selector and use the default branch.
    let (revision, version_label) = match &selector {
        Some(s) if is_git_commit_hash(s) => (s.clone(), s.chars().take(8).collect::<String>()),
        Some(s) => (HF_REVISION_DEFAULT.to_string(), s.clone()),
        None => (
            HF_REVISION_DEFAULT.to_string(),
            HF_REVISION_DEFAULT.to_string(),
        ),
    };

    Ok(HfModelRef {
        repo_id,
        selector: selector.filter(|s| !s.is_empty() && !is_git_commit_hash(s)),
        revision,
        version_label,
    })
}

/// A 40-character hex string is treated as a Git commit SHA.
fn is_git_commit_hash(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Normalise a user-supplied selector for fuzzy GGUF matching.
/// Dash-separated quant tags (e.g. "Q4-K-M") should match underscore variants.
fn normalize_selector(value: &str) -> String {
    value.to_lowercase().replace('-', "_")
}

// ---------------------------------------------------------------------------
// GGUF file selection
// ---------------------------------------------------------------------------

/// Result type for GGUF selection — Ambiguous carries the candidate list so
/// the frontend can present a picker instead of a raw error string.
#[derive(Debug)]
pub enum ChooseFileResult {
    Found(String),
    Ambiguous(Vec<String>),
}

/// Select a GGUF file from the repository file list, optionally filtered by
/// `selector`. Returns `Ambiguous` when multiple files match so callers can
/// surface the list to the user.
fn choose_hf_filename(
    files: &[String],
    selector: Option<&str>,
) -> Result<ChooseFileResult, String> {
    let gguf_files: Vec<String> = files
        .iter()
        .filter(|f| f.to_lowercase().ends_with(".gguf"))
        .cloned()
        .collect();

    if gguf_files.is_empty() {
        return Err("No .gguf files found in the Hugging Face repository".to_string());
    }

    if let Some(selector) = selector {
        // Exact filename match takes priority over fuzzy matching.
        if selector.to_lowercase().ends_with(".gguf") {
            if gguf_files.iter().any(|f| f == selector) {
                return Ok(ChooseFileResult::Found(selector.to_string()));
            }
            return Err(format!(
                "File '{}' not found in repository. Available GGUF files: {}",
                selector,
                gguf_files.join(", ")
            ));
        }

        let normalized = normalize_selector(selector);
        let matches: Vec<String> = gguf_files
            .iter()
            .filter(|f| normalize_selector(f).contains(&normalized))
            .cloned()
            .collect();

        return match matches.len() {
            0 => Err(format!(
                "No GGUF files match '{}'. Available: {}",
                selector,
                gguf_files.join(", ")
            )),
            1 => Ok(ChooseFileResult::Found(matches.into_iter().next().unwrap())),
            _ => Ok(ChooseFileResult::Ambiguous(matches)),
        };
    }

    if gguf_files.len() == 1 {
        Ok(ChooseFileResult::Found(
            gguf_files.into_iter().next().unwrap(),
        ))
    } else {
        Ok(ChooseFileResult::Ambiguous(gguf_files))
    }
}

// ---------------------------------------------------------------------------
// Crypto / encoding helpers
// ---------------------------------------------------------------------------

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

// ---------------------------------------------------------------------------
// Blob I/O
// ---------------------------------------------------------------------------

/// Write raw bytes as a content-addressed blob, computing the sha256 digest
/// in one pass. Skips the write if the blob already exists (idempotent).
///
/// Uses a temp-then-rename strategy so a crashed write never leaves a partial
/// blob at the canonical path.
async fn write_blob_from_bytes(blobs_dir: &Path, bytes: &[u8]) -> Result<(String, u64), String> {
    tokio_fs::create_dir_all(blobs_dir)
        .await
        .map_err(|e| format!("Failed to create blobs directory: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = format!("sha256:{}", hex_encode(&hasher.finalize()));
    let blob_filename = digest_to_blob_filename(&digest);
    let blob_path = blobs_dir.join(&blob_filename);

    // Blob already present — nothing to do.
    if tokio_fs::metadata(&blob_path).await.is_ok() {
        return Ok((digest, bytes.len() as u64));
    }

    // Write to a unique temp path so concurrent writers don't clobber each other.
    let tmp_path = blobs_dir.join(format!("{}.tmp", blob_filename));
    tokio_fs::write(&tmp_path, bytes)
        .await
        .map_err(|e| format!("Failed to write blob {}: {}", digest, e))?;

    // Atomic rename — on the same filesystem this is a directory entry swap,
    // never leaving a partial file at the canonical path.
    tokio_fs::rename(&tmp_path, &blob_path)
        .await
        .map_err(|e| format!("Failed to finalise blob {}: {}", digest, e))?;

    Ok((digest, bytes.len() as u64))
}

/// Stream a HuggingFace model file to disk, computing its sha256 on the fly.
/// Emits `download:progress` Tauri events every `PROGRESS_EMIT_INTERVAL_BYTES`
/// so the frontend can show a real progress bar.
///
/// Returns `(digest, byte_count)`. The digest can be used for manifest construction
/// — we don't receive an expected digest from HF, so we produce our own.
async fn download_hf_file(
    app: &AppHandle,
    client: &reqwest::Client,
    url: &str,
    filename: &str,
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

    // Content-Length is optional — surface it as total_bytes = 0 when absent.
    let total_bytes: u64 = response.content_length().unwrap_or(0);

    tokio_fs::create_dir_all(blobs_dir)
        .await
        .map_err(|e| format!("Failed to create blobs directory: {}", e))?;

    // Unique temp name avoids collision when two HF downloads run concurrently.
    let tmp_path = blobs_dir.join(format!(
        "hf-download-{}.partial",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
    ));

    let mut file = tokio_fs::File::create(&tmp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;
    let mut since_last_emit: u64 = 0;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| format!("Failed while downloading model: {}", e))?;

        let len = bytes.len() as u64;
        downloaded += len;
        since_last_emit += len;
        hasher.update(&bytes);

        file.write_all(&bytes)
            .await
            .map_err(|e| format!("Failed to write model file: {}", e))?;

        // Throttle events to avoid flooding the frontend IPC channel.
        if since_last_emit >= PROGRESS_EMIT_INTERVAL_BYTES {
            since_last_emit = 0;
            let _ = app.emit(
                "download:progress",
                serde_json::json!({
                    "filename": filename,
                    "downloaded": downloaded,
                    "total": total_bytes,
                }),
            );
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush model file: {}", e))?;

    // Final progress event so the frontend reaches 100 % even if the last
    // chunk didn't cross the emit threshold.
    let _ = app.emit(
        "download:progress",
        serde_json::json!({
            "filename": filename,
            "downloaded": downloaded,
            "total": total_bytes,
        }),
    );

    let digest = format!("sha256:{}", hex_encode(&hasher.finalize()));
    let blob_filename = digest_to_blob_filename(&digest);
    let blob_path = blobs_dir.join(&blob_filename);

    if tokio_fs::metadata(&blob_path).await.is_ok() {
        // A previous download produced the same file — discard the duplicate.
        tokio_fs::remove_file(&tmp_path)
            .await
            .map_err(|e| format!("Failed to cleanup temp file: {}", e))?;
    } else {
        tokio_fs::rename(&tmp_path, &blob_path)
            .await
            .map_err(|e| format!("Failed to finalise model file: {}", e))?;
    }

    Ok((digest, downloaded))
}

/// Download a single content-addressed blob from an Ollama-compatible registry.
/// Skips the download when a blob of the expected size already exists locally.
///
/// Verifies the downloaded file size against `size` (when non-zero). SHA-256
/// is intentionally not re-verified here because the registry delivers a
/// content-addressed URL — the digest IS the address.
async fn download_blob(
    app: &AppHandle,
    client: &reqwest::Client,
    registry: &str,
    repository: &str,
    digest: &str,
    size: u64,
    blobs_dir: &Path,
) -> Result<(), String> {
    let blob_filename = digest_to_blob_filename(digest);
    let blob_path = blobs_dir.join(&blob_filename);

    // Fast path — already cached and size matches.
    if let Ok(metadata) = tokio_fs::metadata(&blob_path).await {
        if size == 0 || metadata.len() == size {
            return Ok(());
        }
        // Size mismatch — cached blob is corrupt, re-download.
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

    let total_bytes: u64 = response.content_length().unwrap_or(size);

    // .partial suffix on the blob filename makes cleanup easy on restart.
    let tmp_path = blob_path.with_extension("partial");
    let mut file = tokio_fs::File::create(&tmp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut since_last_emit: u64 = 0;
    let short_digest = &digest[..digest.len().min(19)];

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes =
            chunk.map_err(|e| format!("Failed while downloading blob {}: {}", digest, e))?;

        let len = bytes.len() as u64;
        downloaded += len;
        since_last_emit += len;

        file.write_all(&bytes)
            .await
            .map_err(|e| format!("Failed to write blob {}: {}", digest, e))?;

        if since_last_emit >= PROGRESS_EMIT_INTERVAL_BYTES {
            since_last_emit = 0;
            let _ = app.emit(
                "download:progress",
                serde_json::json!({
                    "digest": short_digest,
                    "downloaded": downloaded,
                    "total": total_bytes,
                }),
            );
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush blob {}: {}", digest, e))?;

    // Verify size only when the manifest told us what to expect.
    if size > 0 {
        let metadata = tokio_fs::metadata(&tmp_path)
            .await
            .map_err(|e| format!("Failed to stat blob {}: {}", digest, e))?;

        if metadata.len() != size {
            // Remove corrupt partial file before returning so a retry starts clean.
            let _ = tokio_fs::remove_file(&tmp_path).await;
            return Err(format!(
                "Blob {} size mismatch (expected {}, got {})",
                digest,
                size,
                metadata.len()
            ));
        }
    }

    // Overwrite any stale blob at the canonical path.
    if tokio_fs::metadata(&blob_path).await.is_ok() {
        tokio_fs::remove_file(&blob_path)
            .await
            .map_err(|e| format!("Failed to remove stale blob: {}", e))?;
    }

    tokio_fs::rename(&tmp_path, &blob_path)
        .await
        .map_err(|e| format!("Failed to finalise blob {}: {}", digest, e))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// HuggingFace download orchestration
// ---------------------------------------------------------------------------

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

    // Query the HF API for the file list — we need siblings to pick the GGUF.
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

    // Resolve which GGUF to use, propagating the Ambiguous variant as a
    // user-facing error listing the candidates.
    let filename = match choose_hf_filename(&files, model_ref.selector.as_deref())? {
        ChooseFileResult::Found(f) => f,
        ChooseFileResult::Ambiguous(candidates) => {
            return Err(format!(
                "Multiple GGUF files found. Please specify one using the \
                 ':selector' suffix.\n\nAvailable files:\n  {}",
                candidates.join("\n  ")
            ));
        }
    };

    let file_url = format!(
        "https://{}/{}/resolve/{}/{}",
        HF_HOST_LONG, model_ref.repo_id, model_ref.revision, filename
    );

    let blobs_dir = PathBuf::from(&models_root).join("blobs");

    let (model_digest, model_size) =
        download_hf_file(&app, &client, &file_url, &filename, &blobs_dir).await?;

    // Build a small config blob recording provenance so the UI can display
    // "from hf.co/..." and tools can trace back to the source.
    let config_metadata = HfConfigMetadata {
        source: HF_HOST.to_string(),
        repo_id: model_ref.repo_id.clone(),
        revision: model_ref.revision.clone(),
        filename: filename.clone(),
    };

    let config_bytes = serde_json::to_vec(&config_metadata)
        .map_err(|e| format!("Failed to serialise config metadata: {}", e))?;

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

    // Derive library/name from the repo_id so the manifest directory mirrors
    // the Ollama layout: manifests/{host}/{org}/{repo}/{version}/manifest.json
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

    // Best-effort: pull the Jinja chat template for automatic prompt formatting.
    // Failure is non-fatal — the model still works without it.
    let _ = crate::services::templates::ensure_hf_chat_template(
        &app,
        &model_ref.repo_id,
        Some(&model_ref.revision),
    )
    .await;

    let manifest_path_str = manifest_path
        .to_str()
        .ok_or_else(|| "Failed to build manifest path string".to_string())?;

    let metadata_root = get_metadata_root(&app)?;
    parse_model_manifest_sync(manifest_path_str.to_string(), models_root, &metadata_root)
}

// ---------------------------------------------------------------------------
// Path parsing utilities
// ---------------------------------------------------------------------------

/// Decompose a manifest path into its `(provider, library, name, version)` tuple.
///
/// Expected layout (mirrors Ollama):
///   `{models_root}/manifests/{provider}/{library}/{name}/{version}[/manifest.json]`
pub fn parse_model_path(model_path: &str) -> Result<(String, String, String, String), String> {
    let path = Path::new(model_path);

    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    let manifests_idx = components
        .iter()
        .position(|&c| c == "manifests")
        .ok_or_else(|| "Path must contain a 'manifests' directory".to_string())?;

    // Need at least 4 more components after "manifests".
    if components.len() < manifests_idx + 5 {
        return Err("Invalid path structure. Expected: \
             .../manifests/{provider}/{library}/{name}/{version}"
            .to_string());
    }

    let provider = components[manifests_idx + 1].to_string();
    let library = components[manifests_idx + 2].to_string();
    let name = components[manifests_idx + 3].to_string();
    let version = components[manifests_idx + 4].to_string();

    Ok((provider, library, name, version))
}

/// Convert a digest string to the blob filename format used on disk.
/// `sha256:60e05f2...` → `sha256-60e05f2...`
pub fn digest_to_blob_filename(digest: &str) -> String {
    digest.replace(':', "-")
}

/// Resolve a digest to an absolute blob path, returning `None` if the blob
/// has not been downloaded yet.
pub fn find_model_blob_path(models_root: &Path, digest: &str) -> Option<String> {
    let blob_path = models_root
        .join("blobs")
        .join(digest_to_blob_filename(digest));

    if blob_path.exists() {
        blob_path.to_str().map(|s| s.to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Tauri commands — manifest parsing
// ---------------------------------------------------------------------------

#[command]
pub async fn parse_model_manifest(
    app: AppHandle,
    model_path: String,
    models_root: String,
) -> Result<ModelInfo, String> {
    let metadata_root = get_metadata_root(&app)?;
    parse_model_manifest_sync(model_path, models_root, &metadata_root)
}

/// Synchronous manifest parser shared between the async command and the
/// directory scanner (which already runs on a blocking thread pool).
pub fn parse_model_manifest_sync(
    model_path: String,
    models_root: String,
    metadata_root: &str,
) -> Result<ModelInfo, String> {
    let (provider, library, name, version) = parse_model_path(&model_path)?;
    let manifest: ModelManifest = crate::utils::read_json(Path::new(&model_path))?;

    // The model layer carries the actual GGUF — config layers are metadata only.
    let model_layer = manifest
        .layers
        .iter()
        .find(|layer| layer.media_type.contains("ollama.image.model"))
        .ok_or_else(|| "No model layer found in manifest".to_string())?;

    let models_root_path = Path::new(&models_root);
    let model_file_path = find_model_blob_path(models_root_path, &model_layer.digest);

    // provider:name:version uniquely identifies a model within the library.
    let full_identifier = format!("{}:{}:{}", provider, name, version);

    let tokenizer_metadata = model_file_path.as_ref().and_then(|path| {
        match get_or_extract_tokenizer_metadata(metadata_root, &full_identifier, path) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("[llama-desktop] Failed to read GGUF metadata: {}", err);
                None
            }
        }
    });

    Ok(ModelInfo {
        provider,
        library,
        name,
        version,
        manifest_data: manifest,
        tokenizer_metadata,
        manifest_path: Some(model_path),
        model_file_path,
        full_identifier,
    })
}

// ---------------------------------------------------------------------------
// GGUF metadata cache
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ModelsMetadataCache {
    models: std::collections::HashMap<String, Value>,
}

fn get_metadata_root(app: &AppHandle) -> Result<String, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;
    Ok(root.to_string_lossy().to_string())
}

fn models_metadata_path(metadata_root: &str) -> PathBuf {
    Path::new(metadata_root).join("models.json")
}

fn load_models_metadata_cache(metadata_root: &str) -> Result<ModelsMetadataCache, String> {
    let path = models_metadata_path(metadata_root);
    if !path.exists() {
        return Ok(ModelsMetadataCache::default());
    }
    crate::utils::read_json(&path)
}

fn save_models_metadata_cache(
    metadata_root: &str,
    cache: &ModelsMetadataCache,
) -> Result<(), String> {
    let path = models_metadata_path(metadata_root);
    crate::utils::save_json(&path, cache)
}

fn get_or_extract_tokenizer_metadata(
    metadata_root: &str,
    full_identifier: &str,
    model_file_path: &str,
) -> Result<Option<Value>, String> {
    let mut cache = load_models_metadata_cache(metadata_root)?;
    if let Some(found) = cache.models.get(full_identifier) {
        return Ok(Some(found.clone()));
    }

    let metadata = read_gguf_tokenizer_metadata(model_file_path)?;
    if metadata.is_empty() {
        return Ok(None);
    }

    let value = Value::Object(metadata);
    cache
        .models
        .insert(full_identifier.to_string(), value.clone());
    save_models_metadata_cache(metadata_root, &cache)?;
    Ok(Some(value))
}

// ---------------------------------------------------------------------------
// GGUF metadata reader (tokenizer.* keys)
// ---------------------------------------------------------------------------

const GGUF_MAGIC: &[u8; 4] = b"GGUF";

const GGUF_VALUE_TYPE_UINT8: u32 = 0;
const GGUF_VALUE_TYPE_INT8: u32 = 1;
const GGUF_VALUE_TYPE_UINT16: u32 = 2;
const GGUF_VALUE_TYPE_INT16: u32 = 3;
const GGUF_VALUE_TYPE_UINT32: u32 = 4;
const GGUF_VALUE_TYPE_INT32: u32 = 5;
const GGUF_VALUE_TYPE_FLOAT32: u32 = 6;
const GGUF_VALUE_TYPE_BOOL: u32 = 7;
const GGUF_VALUE_TYPE_STRING: u32 = 8;
const GGUF_VALUE_TYPE_ARRAY: u32 = 9;
const GGUF_VALUE_TYPE_UINT64: u32 = 10;
const GGUF_VALUE_TYPE_INT64: u32 = 11;
const GGUF_VALUE_TYPE_FLOAT64: u32 = 12;

fn read_gguf_tokenizer_metadata(path: &str) -> Result<Map<String, Value>, String> {
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open model file {}: {}", path, e))?;
    let mut reader = BufReader::new(file);

    let magic = read_bytes(&mut reader, 4)?;
    if magic.as_slice() != GGUF_MAGIC {
        return Err("Not a GGUF file".to_string());
    }

    let _version = read_u32(&mut reader)?;
    let _tensor_count = read_u64(&mut reader)?;
    let kv_count = read_u64(&mut reader)?;

    let mut result = Map::new();
    for _ in 0..kv_count {
        let key = read_string(&mut reader)?;
        let value_type = read_u32(&mut reader)?;
        let value = read_value(&mut reader, value_type)?;

        if key.starts_with("tokenizer.") {
            if key == "tokenizer.ggml.tokens"
                || key == "tokenizer.ggml.token_type"
                || key == "tokenizer.ggml.merges"
            {
                continue;
            }
            result.insert(key, value);
        }
    }

    Ok(result)
}

fn read_bytes(reader: &mut BufReader<std::fs::File>, len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .map_err(|e| format!("Failed to read bytes: {}", e))?;
    Ok(buf)
}

fn read_u32(reader: &mut BufReader<std::fs::File>) -> Result<u32, String> {
    let mut buf = [0u8; 4];
    reader
        .read_exact(&mut buf)
        .map_err(|e| format!("Failed to read u32: {}", e))?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64(reader: &mut BufReader<std::fs::File>) -> Result<u64, String> {
    let mut buf = [0u8; 8];
    reader
        .read_exact(&mut buf)
        .map_err(|e| format!("Failed to read u64: {}", e))?;
    Ok(u64::from_le_bytes(buf))
}

fn read_string(reader: &mut BufReader<std::fs::File>) -> Result<String, String> {
    let len = read_u64(reader)? as usize;
    let bytes = read_bytes(reader, len)?;
    String::from_utf8(bytes).map_err(|e| format!("Failed to parse string: {}", e))
}

fn read_value(reader: &mut BufReader<std::fs::File>, value_type: u32) -> Result<Value, String> {
    match value_type {
        GGUF_VALUE_TYPE_UINT8 => {
            let mut buf = [0u8; 1];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read u8: {}", e))?;
            Ok(Value::Number(serde_json::Number::from(buf[0] as u64)))
        }
        GGUF_VALUE_TYPE_INT8 => {
            let mut buf = [0u8; 1];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read i8: {}", e))?;
            Ok(Value::Number(serde_json::Number::from(buf[0] as i64)))
        }
        GGUF_VALUE_TYPE_UINT16 => {
            let mut buf = [0u8; 2];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read u16: {}", e))?;
            Ok(Value::Number(serde_json::Number::from(
                u16::from_le_bytes(buf) as u64,
            )))
        }
        GGUF_VALUE_TYPE_INT16 => {
            let mut buf = [0u8; 2];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read i16: {}", e))?;
            Ok(Value::Number(serde_json::Number::from(
                i16::from_le_bytes(buf) as i64,
            )))
        }
        GGUF_VALUE_TYPE_UINT32 => {
            let value = read_u32(reader)?;
            Ok(Value::Number(serde_json::Number::from(value)))
        }
        GGUF_VALUE_TYPE_INT32 => {
            let mut buf = [0u8; 4];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read i32: {}", e))?;
            Ok(Value::Number(serde_json::Number::from(
                i32::from_le_bytes(buf) as i64,
            )))
        }
        GGUF_VALUE_TYPE_FLOAT32 => {
            let mut buf = [0u8; 4];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read f32: {}", e))?;
            let value = f32::from_le_bytes(buf);
            serde_json::Number::from_f64(value as f64)
                .map(Value::Number)
                .ok_or_else(|| "Invalid f32 value".to_string())
        }
        GGUF_VALUE_TYPE_BOOL => {
            let mut buf = [0u8; 1];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read bool: {}", e))?;
            Ok(Value::Bool(buf[0] != 0))
        }
        GGUF_VALUE_TYPE_STRING => Ok(Value::String(read_string(reader)?)),
        GGUF_VALUE_TYPE_ARRAY => {
            let element_type = read_u32(reader)?;
            let len = read_u64(reader)? as usize;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(read_value(reader, element_type)?);
            }
            Ok(Value::Array(values))
        }
        GGUF_VALUE_TYPE_UINT64 => {
            let value = read_u64(reader)?;
            Ok(Value::Number(serde_json::Number::from(value)))
        }
        GGUF_VALUE_TYPE_INT64 => {
            let mut buf = [0u8; 8];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read i64: {}", e))?;
            Ok(Value::Number(serde_json::Number::from(i64::from_le_bytes(
                buf,
            ))))
        }
        GGUF_VALUE_TYPE_FLOAT64 => {
            let mut buf = [0u8; 8];
            reader
                .read_exact(&mut buf)
                .map_err(|e| format!("Failed to read f64: {}", e))?;
            let value = f64::from_le_bytes(buf);
            serde_json::Number::from_f64(value)
                .map(Value::Number)
                .ok_or_else(|| "Invalid f64 value".to_string())
        }
        other => Err(format!("Unsupported GGUF value type: {}", other)),
    }
}

// ---------------------------------------------------------------------------
// Directory scanner
// ---------------------------------------------------------------------------

// Four helpers below walk the manifests/ tree depth-first.
// Each level delegates downward and accumulates into the same Vec to avoid
// intermediate allocations.

fn process_version_entry(
    version_entry: fs::DirEntry,
    models_root: &str,
    metadata_root: &str,
    models: &mut Vec<ModelInfo>,
) {
    let manifest_path = version_entry.path();

    // Accept either a bare manifest file or a directory containing manifest.json.
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
            match parse_model_manifest_sync(
                path_str.to_string(),
                models_root.to_string(),
                metadata_root,
            ) {
                Ok(model_info) => models.push(model_info),
                // Log and continue — a single bad manifest shouldn't abort the scan.
                Err(e) => eprintln!("[llama-desktop] Error parsing {}: {}", path_str, e),
            }
        }
    }
}

fn process_model_entry(
    model_entry: fs::DirEntry,
    models_root: &str,
    metadata_root: &str,
    models: &mut Vec<ModelInfo>,
) {
    if let Ok(versions) = fs::read_dir(model_entry.path()) {
        for version_entry in versions.flatten() {
            process_version_entry(version_entry, models_root, metadata_root, models);
        }
    }
}

fn process_library_entry(
    library_entry: fs::DirEntry,
    models_root: &str,
    metadata_root: &str,
    models: &mut Vec<ModelInfo>,
) {
    if let Ok(model_names) = fs::read_dir(library_entry.path()) {
        for model_entry in model_names.flatten() {
            process_model_entry(model_entry, models_root, metadata_root, models);
        }
    }
}

fn process_provider_entry(
    provider_entry: fs::DirEntry,
    models_root: &str,
    metadata_root: &str,
    models: &mut Vec<ModelInfo>,
) {
    if let Ok(libraries) = fs::read_dir(provider_entry.path()) {
        for library_entry in libraries.flatten() {
            process_library_entry(library_entry, models_root, metadata_root, models);
        }
    }
}

#[command]
pub async fn scan_models_directory(
    app: AppHandle,
    models_root: String,
) -> Result<Vec<ModelInfo>, String> {
    let manifests_path = Path::new(&models_root).join("manifests");

    if !manifests_path.exists() {
        return Ok(Vec::new()); // Empty library is not an error — first launch is normal.
    }

    let mut models = Vec::new();
    let metadata_root = get_metadata_root(&app)?;

    if let Ok(providers) = fs::read_dir(&manifests_path) {
        for provider_entry in providers.flatten() {
            process_provider_entry(provider_entry, &models_root, &metadata_root, &mut models);
        }
    }

    Ok(models)
}

// ---------------------------------------------------------------------------
// Tauri commands — library persistence
// ---------------------------------------------------------------------------

#[command]
pub async fn save_model_library(
    library_path: String,
    models: Vec<ModelInfo>,
) -> Result<(), String> {
    crate::utils::save_json(Path::new(&library_path), &ModelLibrary { models })
}

#[command]
pub async fn load_model_library(
    app: AppHandle,
    library_path: String,
) -> Result<Vec<ModelInfo>, String> {
    if !Path::new(&library_path).exists() {
        // Return empty list rather than an error — caller decides if that's a problem.
        return Ok(Vec::new());
    }

    let mut library: ModelLibrary = crate::utils::read_json(Path::new(&library_path))?;
    let metadata_root = get_metadata_root(&app)?;
    if let Ok(cache) = load_models_metadata_cache(&metadata_root) {
        for model in &mut library.models {
            if model.tokenizer_metadata.is_none() {
                if let Some(found) = cache.models.get(&model.full_identifier) {
                    model.tokenizer_metadata = Some(found.clone());
                }
            }
        }
    }
    Ok(library.models)
}

// ---------------------------------------------------------------------------
// Tauri command — model download entry point
// ---------------------------------------------------------------------------

#[command]
pub async fn download_model_from_registry(
    app: AppHandle,
    model_reference: String,
    models_root: String,
) -> Result<ModelInfo, String> {
    // Route HuggingFace references through their own download path.
    if is_hf_reference(&model_reference) {
        return download_model_from_hf(app, model_reference, models_root).await;
    }

    let model_ref = parse_model_reference(&model_reference)?;
    let repository = format!("{}/{}", model_ref.library, model_ref.name);

    let client = reqwest::Client::builder()
        .user_agent("llama-desktop")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    // Fetch the manifest first — it describes which blobs we need to pull.
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

    // Persist the manifest before downloading blobs so a crash during download
    // leaves us with a restorable state on the next launch.
    let manifest_path = PathBuf::from(&models_root)
        .join("manifests")
        .join(&model_ref.registry)
        .join(&model_ref.library)
        .join(&model_ref.name)
        .join(&model_ref.version)
        .join("manifest.json");

    crate::utils::save_json(&manifest_path, &manifest)?;

    let blobs_dir = PathBuf::from(&models_root).join("blobs");

    // Config blob is usually tiny — download it first so failure here doesn't
    // waste time on a large model layer.
    download_blob(
        &app,
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
        .ok_or_else(|| "Failed to build manifest path string".to_string())?;

    let metadata_root = get_metadata_root(&app)?;
    parse_model_manifest_sync(manifest_path_str.to_string(), models_root, &metadata_root)
}

// ---------------------------------------------------------------------------
// Test utilities
// ---------------------------------------------------------------------------

// These wrappers exist so test call-sites are explicit about what they're
// reaching into. `pub use` would require the originals to be `pub(crate)` or
// higher; wrappers work regardless of the parent item's visibility.
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
        metadata_root: &str,
    ) -> Result<ModelInfo, String> {
        parse_model_manifest_sync(model_path, models_root, metadata_root)
    }

    pub fn scan_models_directory_for_test(
        models_root: String,
        metadata_root: &str,
    ) -> Result<Vec<ModelInfo>, String> {
        let manifests_path = Path::new(&models_root).join("manifests");
        if !manifests_path.exists() {
            return Ok(Vec::new());
        }

        let mut models = Vec::new();
        if let Ok(providers) = fs::read_dir(&manifests_path) {
            for provider_entry in providers.flatten() {
                process_provider_entry(provider_entry, &models_root, metadata_root, &mut models);
            }
        }

        Ok(models)
    }

    pub async fn load_model_library_for_test(
        library_path: String,
        metadata_root: &str,
    ) -> Result<Vec<ModelInfo>, String> {
        if !Path::new(&library_path).exists() {
            return Ok(Vec::new());
        }

        let mut library: ModelLibrary = crate::utils::read_json(Path::new(&library_path))?;
        if let Ok(cache) = load_models_metadata_cache(metadata_root) {
            for model in &mut library.models {
                if model.tokenizer_metadata.is_none() {
                    if let Some(found) = cache.models.get(&model.full_identifier) {
                        model.tokenizer_metadata = Some(found.clone());
                    }
                }
            }
        }
        Ok(library.models)
    }

    pub fn choose_hf_filename_for_test(
        files: &[String],
        selector: Option<&str>,
    ) -> Result<ChooseFileResult, String> {
        choose_hf_filename(files, selector)
    }

    #[allow(dead_code)]
    pub(crate) fn parse_hf_reference_for_test(reference: &str) -> Result<HfModelRef, String> {
        parse_hf_reference(reference)
    }
}
