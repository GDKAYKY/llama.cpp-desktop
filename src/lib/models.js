import { invokeCommand } from './ipc.js';
import { open } from '@tauri-apps/plugin-dialog';

/**
 * Parse a single model manifest file
 * @param {string} modelPath - Path to the manifest file
 * @param {string} modelsRoot - Root directory where models are stored
 * @returns {Promise<Object>} Model information
 */
export async function parseModelManifest(modelPath, modelsRoot) {
    return await invokeCommand('parse_model_manifest', { modelPath, modelsRoot });
}

/**
 * Scan entire models directory and parse all manifests
 * @param {string} modelsRoot - Root directory where models are stored
 * @returns {Promise<Array>} Array of model information objects
 */
export async function scanModelsDirectory(modelsRoot) {
    return await invokeCommand('scan_models_directory', { modelsRoot });
}

/**
 * Save model library to JSON file
 * @param {string} libraryPath - Path where to save the library
 * @param {Array} models - Array of model information objects
 * @returns {Promise<void>}
 */
export async function saveModelLibrary(libraryPath, models) {
    return await invokeCommand('save_model_library', { libraryPath, models });
}

/**
 * Load model library from JSON file
 * @param {string} libraryPath - Path to the library file
 * @returns {Promise<Array>} Array of model information objects
 */
export async function loadModelLibrary(libraryPath) {
    return await invokeCommand('load_model_library', { libraryPath });
}

/**
 * Open directory picker dialog for user to select models folder
 * @returns {Promise<string|null>} Selected directory path or null if cancelled
 */
export async function selectModelsDirectory() {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: 'Select Models Directory'
        });
        return selected;
    } catch (err) {
        console.error('Dialog error:', err);
        throw new Error(`Failed to open directory dialog: ${err.message || err}`);
    }
}

/**
 * Format model identifier from components
 * @param {string} provider - Model provider (e.g., "registry.ollama.ai")
 * @param {string} name - Model name (e.g., "qwen2.5-coder")
 * @param {string} version - Model version (e.g., "7b")
 * @returns {string} Formatted identifier (e.g., "registry.ollama.ai:qwen2.5-coder:7b")
 */
export function formatModelIdentifier(provider, name, version) {
    return `${provider}:${name}:${version}`;
}

/**
 * Parse model identifier into components
 * @param {string} identifier - Full model identifier
 * @returns {Object} Object with provider, name, and version
 */
export function parseModelIdentifier(identifier) {
    const parts = identifier.split(':');
    if (parts.length !== 3) {
        throw new Error('Invalid model identifier format. Expected: provider:name:version');
    }
    return {
        provider: parts[0],
        name: parts[1],
        version: parts[2]
    };
}

/**
 * Get model file path from digest
 * @param {string} modelsRoot - Root directory where models are stored
 * @param {string} digest - Digest hash (e.g., "sha256:60e05f2...")
 * @returns {string} Path to blob file
 */
export function getModelBlobPath(modelsRoot, digest) {
    const blobFilename = digest.replace(':', '-');
    return `${modelsRoot}/blobs/${blobFilename}`;
}
