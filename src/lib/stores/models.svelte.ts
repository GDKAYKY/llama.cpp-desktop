import {
  scanModelsDirectory,
  loadModelLibrary,
  saveModelLibrary,
  selectModelsDirectory,
} from "$lib/services/models";
import { downloadModelFromRegistry } from "$lib/services/model_downloads";
import { removeModelByManifestPath } from "$lib/services/models_delete";
import { notifications } from "$lib/shared/notifications";
import { settingsStore } from "./settings.svelte";
import type { Model } from "$lib/types/models";
import { listen } from "@tauri-apps/api/event";

export interface DownloadState {
  downloaded: number;
  total: number;
  percentage: number;
  speed: number;
  lastUpdate: number;
}

class ModelsStore {
  models = $state<Model[]>([]);
  selectedModel = $state<Model | null>(null);
  isLoading = $state(false);
  isDownloading = $state(false);
  error = $state<string | null>(null);
  successMessage = $state("");
  downloads = $state<Record<string, DownloadState>>({});
  private downloadNotificationId: string | number | null = null;

  constructor() {
    this.listenForDownloads();
  }

  private async listenForDownloads() {
    const CLEANUP_DELAY_MS = 2000;

    try {
      await listen<{
        filename?: string;
        digest?: string;
        downloaded: number;
        total: number;
      }>("download:progress", (event) => {
        const { filename, digest, downloaded, total } = event.payload;
        const id = filename || digest || "unknown";

        const now = Date.now();
        const previousDownload = this.downloads[id];

        let speed = 0;
        if (previousDownload && now > previousDownload.lastUpdate) {
          const timeDiff = (now - previousDownload.lastUpdate) / 1000;
          const bytesDiff = downloaded - previousDownload.downloaded;
          speed = Math.round(bytesDiff / timeDiff);
        }

        const percentage =
          total > 0
            ? Math.min(100, Math.max(0, (downloaded / total) * 100))
            : 0;

        this.downloads[id] = {
          downloaded,
          total,
          percentage,
          speed,
          lastUpdate: now,
        };

        if (this.downloadNotificationId !== null) {
          notifications.loading(`Downloading ${id}`, {
            id: this.downloadNotificationId,
            progress: percentage,
          });
        }

        if (downloaded >= total && total > 0) {
          setTimeout(() => delete this.downloads[id], CLEANUP_DELAY_MS);
        }
      });
    } catch (err) {
      console.error("Failed to listen for downloads:", err);
    }
  }

  get modelsRoot() {
    return settingsStore.settings.modelsDirectory;
  }

  get libraryPath() {
    return this.modelsRoot ? `${this.modelsRoot}/modelLibrary.json` : "";
  }

  async selectDirectory() {
    try {
      this.error = null;
      this.successMessage = "";
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
      this.models = existingModels;
    } catch (err) {
      console.log("No existing library found or failed to load");
    } finally {
      this.isLoading = false;
    }
  }

  async scanModelsDirectory() {
    if (!this.modelsRoot) {
      this.error = "Please select a models directory first";
      return;
    }

    try {
      this.isLoading = true;
      this.error = null;
      this.successMessage = "";
      this.models = await scanModelsDirectory(this.modelsRoot);

      if (this.models.length > 0) {
        await saveModelLibrary(this.libraryPath, this.models);
        this.successMessage = `Found and saved ${this.models.length} model(s)`;
      } else {
        this.error = "No models found in the selected directory";
      }
    } catch (err) {
      this.error = `Failed to scan directory: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      this.isLoading = false;
    }
  }

  async download(modelReference: string) {
    if (!this.modelsRoot) {
      this.error = "Please select a models directory first";
      return;
    }

    const trimmed = modelReference.trim();
    if (!trimmed) {
      this.error = "Please enter a model reference";
      return;
    }

    try {
      this.isDownloading = true;
      this.error = null;
      this.successMessage = "";
      this.downloadNotificationId = notifications.loading(
        `Downloading ${trimmed}`,
        {
          progress: 0,
        },
      );

      const model = await downloadModelFromRegistry(this.modelsRoot, trimmed);
      const existingIndex = this.models.findIndex(
        (m) => m.full_identifier === model.full_identifier,
      );

      if (existingIndex >= 0) {
        this.models = this.models.map((existing, index) =>
          index === existingIndex ? model : existing,
        );
      } else {
        this.models = [model, ...this.models];
      }

      await saveModelLibrary(this.libraryPath, this.models);
      this.successMessage = `Downloaded ${model.full_identifier}`;
      notifications.success(`Downloaded ${model.full_identifier}`, {
        id: this.downloadNotificationId ?? undefined,
      });
    } catch (err) {
      this.error = `Failed to download model: ${err instanceof Error ? err.message : String(err)}`;
      notifications.error(this.error, {
        id: this.downloadNotificationId ?? undefined,
      });
    } finally {
      this.isDownloading = false;
      this.downloadNotificationId = null;
    }
  }

  async remove(model: Model) {
    try {
      this.isLoading = true;
      this.error = null;

      if (model.manifest_path && this.modelsRoot) {
        await removeModelByManifestPath(
          model.manifest_path as string,
          this.modelsRoot,
        );
      }

      this.models = this.models.filter(
        (m) => m.full_identifier !== model.full_identifier,
      );

      if (this.selectedModel?.full_identifier === model.full_identifier) {
        this.selectedModel = null;
      }

      await saveModelLibrary(this.libraryPath, this.models);
      this.successMessage = `Removed ${model.name} from library`;
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
      await this.scanModelsDirectory();
    }
  }

  selectModel(model: Model) {
    this.selectedModel = model;
    this.successMessage = "";
  }

  clearMessages() {
    this.error = null;
    this.successMessage = "";
  }
}

export const modelsStore = new ModelsStore();
export type { Model };
