import { invokeCommand } from '../infrastructure/ipc';
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
 * Remove a model by its manifest path, including all associated blob files
 */
export async function removeModelByManifestPath(manifestPath: string, modelsRoot: string): Promise<boolean> {
    return await invokeCommand('remove_model_by_manifest_path', { 
        manifestPath: manifestPath, 
        modelsRoot: modelsRoot 
    }) as Promise<boolean>;
}

/**
 * Remove a model by its full identifier
 */
export async function removeModelByIdentifier(fullIdentifier: string, modelsRoot: string): Promise<boolean> {
    // First, we need to find the manifest path for this model
    const allModels = await scanModelsDirectory(modelsRoot);
    const model = allModels.find(m => m.full_identifier === fullIdentifier);
    
    if (!model) {
        throw new Error(`Model with identifier ${fullIdentifier} not found`);
    }
    
    // Derive the manifest path from the model's properties
    // From the Rust code, the format is: {modelsRoot}/manifests/{provider}/{library}/{name}/{version}/manifest.json
    const manifestPath = `${modelsRoot}/manifests/${model.provider}/${model.library}/${model.name}/${model.version}/manifest.json`;
    return await removeModelByManifestPath(manifestPath, modelsRoot);
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
            title: 'Select Llama Directory'
        });

        return selected;
    } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        throw new Error(`Failed to open directory dialog: ${errorMessage}`);
    }
}