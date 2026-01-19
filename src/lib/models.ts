import { invokeCommand } from './ipc';
import { open } from '@tauri-apps/plugin-dialog';

type Model = {
    name: string;
    version: string;
    provider: string;
    library: string;
    full_identifier: string;
    manifest: { layers: Array<{ size: number }> };
    model_file_path?: string;
};

/**
 * Parse a single model manifest file
 */
export async function parseModelManifest(modelPath: string, modelsRoot: string): Promise<Model> {
    return await invokeCommand('parse_model_manifest', { modelPath, modelsRoot }) as Promise<Model>;
}

/**
 * Scan entire models directory and parse all manifests
 */
export async function scanModelsDirectory(modelsRoot: string): Promise<Model[]> {
    return await invokeCommand('scan_models_directory', { modelsRoot }) as Promise<Model[]>;
}

/**
 * Save model library to JSON file
 */
export async function saveModelLibrary(libraryPath: string, models: Model[]): Promise<void> {
    await invokeCommand('save_model_library', { libraryPath, models });
}

/**
 * Load model library from JSON file
 */
export async function loadModelLibrary(libraryPath: string): Promise<Model[]> {
    return await invokeCommand('load_model_library', { libraryPath }) as Promise<Model[]>;
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
        console.error('Dialog error:', err);
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
 * Get model file path from digest
 */
export function getModelBlobPath(modelsRoot: string, digest: string): string {
    const blobFilename = digest.replace(':', '-');
    return `${modelsRoot}/blobs/${blobFilename}`;
}
