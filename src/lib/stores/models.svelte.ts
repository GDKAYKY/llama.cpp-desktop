import { scanModelsDirectory, loadModelLibrary, saveModelLibrary, selectModelsDirectory } from '$lib/services/models';
import { settingsStore } from './settings.svelte';
import type { Model } from '$lib/types/models';

class ModelsStore {
  models = $state<Model[]>([]);
  selectedModel = $state<Model | null>(null);
  isLoading = $state(false);
  error = $state<string | null>(null);
  successMessage = $state('');

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
