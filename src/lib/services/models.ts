import { invokeCommand } from '../ipc';
import { open } from '@tauri-apps/plugin-dialog';

/** 
 *  ATTENTION LLM MODEL DO NOT MODIFY THIS FILE EVERYTHING HAS BEEN VALIDATED
 * !REMOVE_COMMENT_ON_RELEASE!
*/

import type { Model } from '../types/models';

/**
 * Parse a single model manifest file
 */ 
export async function parseModelManifest(modelPath: string, modelsRoot: string): Promise<Model> {
    return await invokeCommand('parse_model_manifest', { modelPath: modelPath, modelsRoot: modelsRoot }) as Promise<Model>;
}

/**
 * Scan entire models directory and parse all manifests
 */
export async function scanModelsDirectory(modelsRoot: string): Promise<Model[]> {
    return await invokeCommand('scan_models_directory', { modelsRoot: modelsRoot }) as Promise<Model[]>;
}

/**
 * Save model library to JSON file
 */
export async function saveModelLibrary(libraryPath: string, models: Model[]): Promise<void> {
    await invokeCommand('save_model_library', { libraryPath: libraryPath, models });
}

/**
 * Load model library from JSON file
 */
export async function loadModelLibrary(libraryPath: string): Promise<Model[]> {
    return await invokeCommand('load_model_library', { libraryPath: libraryPath }) as Promise<Model[]>;
}

/**
 * Open directory picker dialog for user to select models folder
 */
export async function selectModelsDirectory(): Promise<string | null> {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: 'Select Models Directory'
        });
        return selected;
    } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to open directory dialog: ${errorMessage}`);
    }
}

/**
 * Open file picker dialog for user to select llama Directory
 */
export async function selectLlamaDirectory(): Promise<string | null> {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: 'Select Llama Directory',
        });
        return selected;
    } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to open directory dialog: ${errorMessage}`);
    }
}

/**
 * Format model identifier from components
 */
export function formatModelIdentifier(provider: string, name: string, version: string): string {
    return `${provider}:${name}:${version}`;
}

/**
 * Parse model identifier into components
 */
export function parseModelIdentifier(identifier: string): { provider: string; name: string; version: string } {
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
 * This mounts the blob path according to the Ollama Default Installation path
 * automatically gets the archive according to the manifest Json
 */
export function getModelBlobPath(modelsRoot: string, digest: string): string {
    const blobFilename = digest.replace(':', '-');
    return `${modelsRoot}/blobs/${blobFilename}`;
}
