use serde::{Deserialize, Serialize};

/// Represents a single layer in a container-style model manifest.
/// In the context of AI models, layers usually point to GGUF model files
/// or other binary blobs stored in the registry.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestLayer {
    /// The MIME type of the layer content (e.g., application/vnd.ollama.image.model).
    #[serde(rename = "mediaType")]
    pub media_type: String,

    /// The unique SHA256 digest of the content, used for content-addressable storage.
    pub digest: String,

    /// The size of the layer in bytes.
    pub size: u64,
}

/// Metadata configuration for a model manifest.
/// Defines how the model should be executed or its specific hardware requirements.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifestConfig {
    /// The MIME type of the config JSON (e.g., application/vnd.ollama.image.config).
    #[serde(rename = "mediaType")]
    pub media_type: String,

    /// The SHA256 digest of the configuration metadata file.
    pub digest: String,

    /// The size of the configuration file in bytes.
    pub size: u64,
}

/// The root structure of an OCI-compliant model manifest file.
/// This matches the format used by the Ollama registry and local manifests.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelManifest {
    /// Schema version for the manifest (usually 2).
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,

    /// The overall media type of the manifest.
    #[serde(rename = "mediaType")]
    pub media_type: String,

    /// Core configuration for this model version.
    pub config: ManifestConfig,

    /// The data layers associated with this model (binaries, license files, etc).
    pub layers: Vec<ManifestLayer>,
}

/// Combined metadata and file session information for an installed model.
/// This is used throughout the application to manage model references.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    /// The distribution provider (e.g., registry.ollama.ai, hf.co).
    pub provider: String,

    /// The repository or user namespace for the model.
    pub library: String,

    /// The core name of the model.
    pub name: String,

    /// The specific tag or version identifier.
    pub version: String,

    /// The parsed manifest data from the manifest.json file.
    #[serde(rename = "manifest_data", alias = "manifest")]
    pub manifest_data: ModelManifest,

    /// Absolute path to the model binary (blob) on the local filesystem.
    pub model_file_path: Option<String>,

    /// Absolute path to the manifest.json for this model.
    pub manifest_path: Option<String>,

    /// A human-readable identifier (e.g., "registry.ollama.ai:llama3:latest").
    pub full_identifier: String,
}

/// A container for a collection of models, typically used for local state persistence.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelLibrary {
    /// All models currently tracked in the user's library.
    pub models: Vec<ModelInfo>,
}
