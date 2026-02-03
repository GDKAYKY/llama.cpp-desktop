use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::Path;

/// Read and deserialize JSON from a file
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON from {}: {}", path.display(), e))
}

/// Serialize and write JSON to a file
pub fn save_json<T: Serialize>(path: &Path, data: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize data: {}", e))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create parent directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    fs::write(path, json).map_err(|e| format!("Failed to write to file {}: {}", path.display(), e))
}
