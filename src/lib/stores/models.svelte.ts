import { scanModelsDirectory, loadModelLibrary, saveModelLibrary, selectModelsDirectory } from '$lib/services/models';
import { removeModelByIdentifier, removeModelByManifestPath } from '$lib/services/models_delete';
import { downloadModelFromRegistry } from '$lib/services/model_downloads';
import { settingsStore } from './settings.svelte';
import type { Model } from '$lib/types/models';
import { listen } from '@tauri-apps/api/event';

export interface DownloadProgress {
  reference: string;
  downloaded: number;
  total: number;
  speed: number;
}

class ModelsStore {
  models = $state<Model[]>([]);
  selectedModel = $state<Model | null>(null);
  isLoading = $state(false);
  isDownloading = $state(false);
  error = $state<string | null>(null);
  successMessage = $state('');
  downloads = $state<Record<string, DownloadProgress>>({});

  constructor() {
    this.setupListeners();
  }

  async setupListeners() {
    try {
      await listen<DownloadProgress>('download-progress', (event) => {
        this.downloads[event.payload.reference] = event.payload;
      });
    } catch (err) {
      console.error('Failed to setup download listeners:', err);
    }
  }

  get modelsRoot() {
    return settingsStore.settings.modelsDirectory;
  }

  get libraryPath() {
    return this.modelsRoot ? `${this.modelsRoot}/modelLibrary.json` : '';
  }

  async selectDirectory() {
    try {
      this.error = null;
      this.successMessage = '';
      const selected = await selectModelsDirectory();
      if (selected) {
        settingsStore.settings.modelsDirectory = selected;
        await settingsStore.update({ modelsDirectory: selected });
        await this.loadLibrary();
      }
    } catch (err) {
      this.error = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
    }
  }

  async loadLibrary() {
    if (!this.libraryPath) return;

    try {
      this.isLoading = true;
      const existingModels = await loadModelLibrary(this.libraryPath);
      if (existingModels.length > 0) {
        this.models = existingModels;
      }
    } catch (err) {
      console.log('No existing library found or failed to load');
    } finally {
      this.isLoading = false;
    }
  }

  async scan() {
    if (!this.modelsRoot) {
      this.error = 'Please select a models directory first';
      return;
    }

    try {
      this.isLoading = true;
      this.error = null;
      this.successMessage = '';
      this.models = await scanModelsDirectory(this.modelsRoot);

      if (this.models.length > 0) {
        await saveModelLibrary(this.libraryPath, this.models);
        this.successMessage = `Found and saved ${this.models.length} model(s)`;
      } else {
        this.error = 'No models found in the selected directory';
      }
    } catch (err) {
      this.error = `Failed to scan directory: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      this.isLoading = false;
    }
  }

  async download(modelReference: string) {
    if (!this.modelsRoot) {
      this.error = 'Please select a models directory first';
      return;
    }

    const trimmed = modelReference.trim();
    if (!trimmed) {
      this.error = 'Please enter a model reference';
      return;
    }

    try {
      this.isDownloading = true;
      this.error = null;
      this.successMessage = '';

      this.downloads[trimmed] = {
        reference: trimmed,
        downloaded: 0,
        total: 100,
        speed: 0
      };

      downloadModelFromRegistry(this.modelsRoot, trimmed).then(async (model) => {
        const existingIndex = this.models.findIndex((m) => m.full_identifier === model.full_identifier);

        if (existingIndex >= 0) {
          this.models[existingIndex] = model;
        } else {
          this.models = [model, ...this.models];
        }

        await saveModelLibrary(this.libraryPath, this.models);
        this.successMessage = `Downloaded ${model.full_identifier}`;
        this.isDownloading = false;

        setTimeout(() => {
          const newDownloads = { ...this.downloads };
          delete newDownloads[trimmed];
          this.downloads = newDownloads;
        }, 2000);
      }).catch(err => {
        this.error = `Failed to download model: ${err instanceof Error ? err.message : String(err)}`;
        this.isDownloading = false;

        setTimeout(() => {
          const newDownloads = { ...this.downloads };
          delete newDownloads[trimmed];
          this.downloads = newDownloads;
        }, 2000);
      });
    } catch (err) {
      this.error = `Failed to initiate download: ${err instanceof Error ? err.message : String(err)}`;
      this.isDownloading = false;
    }
  }

  async remove(model: Model) {
    if (!this.modelsRoot) {
      this.error = 'Please select a models directory first';
      return;
    }

    try {
      this.isLoading = true;
      this.error = null;
      this.successMessage = '';

      const success = model.manifest_path
        ? await removeModelByManifestPath(model.manifest_path, this.modelsRoot)
        : await removeModelByIdentifier(model.full_identifier, this.modelsRoot);

      if (success) {
        // Remove the model from the store
        this.models = this.models.filter(m => m.full_identifier !== model.full_identifier);
        await saveModelLibrary(this.libraryPath, this.models);
        this.successMessage = `Successfully removed model: ${model.full_identifier}`;
      } else {
        this.error = `Failed to fully remove model: ${model.full_identifier}. Some files may remain.`;
      }
    } catch (err) {
      this.error = `Failed to remove model: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      this.isLoading = false;
    }
  }

  async refresh() {
    if (!this.modelsRoot) return;
    await this.loadLibrary();
    if (this.models.length === 0) {
        await this.scan();
    }
  }

  selectModel(model: Model) {
    this.selectedModel = model;
    this.successMessage = '';
  }

  clearMessages() {
    this.error = null;
    this.successMessage = '';
  }
}

export const modelsStore = new ModelsStore();
export type { Model };
