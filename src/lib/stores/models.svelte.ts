import { scanModelsDirectory } from '$lib/models';
import { settingsStore } from './settings.svelte';

export interface Model {
  name: string;
  version: string;
  provider: string;
  library: string;
  full_identifier: string;
  manifest: { layers: Array<{ size: number }> };
  model_file_path?: string;
}

class ModelsStore {
  models = $state<Model[]>([]);
  selectedModel = $state<Model | null>(null);
  isLoading = $state(false);
  error = $state<string | null>(null);

  async refresh() {
    const modelsDir = settingsStore.settings.models_directory;
    if (!modelsDir) {
      this.models = [];
      return;
    }

    try {
      this.isLoading = true;
      this.models = await scanModelsDirectory(modelsDir);
      
      // Select first model if none selected
      if (!this.selectedModel && this.models.length > 0) {
        this.selectedModel = this.models[0];
      }
      
      this.error = null;
    } catch (err) {
      console.error('Failed to scan models:', err);
      this.error = 'Failed to scan models directory';
    } finally {
      this.isLoading = false;
    }
  }

  selectModel(model: Model) {
    this.selectedModel = model;
  }
}

export const modelsStore = new ModelsStore();
