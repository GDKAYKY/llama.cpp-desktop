import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const mockScanModelsDirectory = vi.fn().mockResolvedValue([]);
const mockLoadModelLibrary = vi.fn().mockResolvedValue([]);
const mockSaveModelLibrary = vi.fn().mockResolvedValue(undefined);
const mockSelectModelsDirectory = vi.fn().mockResolvedValue("/models");

vi.mock("$lib/services/models", () => ({
  scanModelsDirectory: (...args: unknown[]) => mockScanModelsDirectory(...args),
  loadModelLibrary: (...args: unknown[]) => mockLoadModelLibrary(...args),
  saveModelLibrary: (...args: unknown[]) => mockSaveModelLibrary(...args),
  selectModelsDirectory: (...args: unknown[]) => mockSelectModelsDirectory(...args),
}));

const mockDownloadModelFromRegistry = vi.fn();

vi.mock("$lib/services/model_downloads", () => ({
  downloadModelFromRegistry: (...args: unknown[]) => mockDownloadModelFromRegistry(...args),
}));

const mockSettingsStore = {
  settings: { modelsDirectory: "/models" },
  update: vi.fn().mockResolvedValue(undefined),
};

vi.mock("$lib/stores/settings.svelte", () => ({
  settingsStore: mockSettingsStore,
}));

function makeModel(overrides: Partial<{ name: string; full_identifier: string }> = {}) {
  return {
    name: overrides.name ?? "test-model",
    version: "latest",
    provider: "meta",
    library: "ollama",
    full_identifier: overrides.full_identifier ?? "meta:test-model:latest",
    manifest_data: {
      config: { mediaType: "application/json", digest: "sha256:abc", size: 100 },
      layers: [],
    },
  };
}

describe("modelsStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSettingsStore.settings.modelsDirectory = "/models";
    mockScanModelsDirectory.mockResolvedValue([]);
    mockLoadModelLibrary.mockResolvedValue([]);
    mockSaveModelLibrary.mockResolvedValue(undefined);
    mockSelectModelsDirectory.mockResolvedValue("/models");
    mockDownloadModelFromRegistry.mockResolvedValue(makeModel());
  });

  it("computes libraryPath from modelsRoot", async () => {
    const { modelsStore } = await import("$lib/stores/models.svelte");
    expect(modelsStore.libraryPath).toBe("/models/modelLibrary.json");
  });

  it("returns empty libraryPath when no modelsRoot", async () => {
    mockSettingsStore.settings.modelsDirectory = null as any;
    const { modelsStore } = await import("$lib/stores/models.svelte");
    expect(modelsStore.libraryPath).toBe("");
  });

  describe("scan", () => {
    it("sets error when scanning without modelsRoot", async () => {
      mockSettingsStore.settings.modelsDirectory = null as any;
      const { modelsStore } = await import("$lib/stores/models.svelte");

      await modelsStore.scan();

      expect(modelsStore.error).toBe("Please select a models directory first");
    });

    it("scans directory and saves library when models found", async () => {
      const models = [makeModel({ full_identifier: "meta:llama:v1" })];
      mockScanModelsDirectory.mockResolvedValue(models);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.scan();

      expect(mockScanModelsDirectory).toHaveBeenCalledWith("/models");
      expect(mockSaveModelLibrary).toHaveBeenCalledWith(
        "/models/modelLibrary.json",
        models
      );
      expect(modelsStore.models).toEqual(models);
      expect(modelsStore.successMessage).toContain("1 model(s)");
      expect(modelsStore.isLoading).toBe(false);
    });

    it("sets error when no models found", async () => {
      mockScanModelsDirectory.mockResolvedValue([]);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.scan();

      expect(modelsStore.error).toBe(
        "No models found in the selected directory"
      );
    });

    it("sets error on scan failure", async () => {
      mockScanModelsDirectory.mockRejectedValue(new Error("Scan failed"));

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.scan();

      expect(modelsStore.error).toContain("Failed to scan directory");
      expect(modelsStore.error).toContain("Scan failed");
      expect(modelsStore.isLoading).toBe(false);
    });

    it("handles non-Error scan failure", async () => {
      mockScanModelsDirectory.mockRejectedValue("raw error");

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.scan();

      expect(modelsStore.error).toContain("raw error");
    });
  });

  describe("loadLibrary", () => {
    it("loads existing library", async () => {
      const models = [makeModel()];
      mockLoadModelLibrary.mockResolvedValue(models);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.loadLibrary();

      expect(mockLoadModelLibrary).toHaveBeenCalledWith(
        "/models/modelLibrary.json"
      );
      expect(modelsStore.models).toEqual(models);
      expect(modelsStore.isLoading).toBe(false);
    });

    it("skips when no libraryPath", async () => {
      mockSettingsStore.settings.modelsDirectory = null as any;

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.loadLibrary();

      expect(mockLoadModelLibrary).not.toHaveBeenCalled();
    });

    it("keeps models empty when library returns empty array", async () => {
      mockLoadModelLibrary.mockResolvedValue([]);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.loadLibrary();

      expect(modelsStore.isLoading).toBe(false);
    });

    it("silently handles load failure", async () => {
      mockLoadModelLibrary.mockRejectedValue(new Error("Not found"));

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.loadLibrary();

      expect(modelsStore.isLoading).toBe(false);
    });
  });

  describe("selectDirectory", () => {
    it("selects directory and loads library", async () => {
      mockSelectModelsDirectory.mockResolvedValue("/new/models");
      mockLoadModelLibrary.mockResolvedValue([]);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.selectDirectory();

      expect(mockSelectModelsDirectory).toHaveBeenCalled();
      expect(mockSettingsStore.update).toHaveBeenCalledWith({
        modelsDirectory: "/new/models",
      });
    });

    it("does nothing when user cancels selection", async () => {
      mockSelectModelsDirectory.mockResolvedValue(null);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.selectDirectory();

      expect(mockSettingsStore.update).not.toHaveBeenCalled();
    });

    it("sets error on failure", async () => {
      mockSelectModelsDirectory.mockRejectedValue(
        new Error("Dialog cancelled")
      );

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.selectDirectory();

      expect(modelsStore.error).toContain("Failed to select directory");
    });

    it("handles non-Error failure", async () => {
      mockSelectModelsDirectory.mockRejectedValue("string error");

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.selectDirectory();

      expect(modelsStore.error).toContain("string error");
    });
  });

  describe("download", () => {
    it("downloads model and adds to list", async () => {
      const model = makeModel({ full_identifier: "meta:new-model:v1" });
      mockDownloadModelFromRegistry.mockResolvedValue(model);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.download("new-model:v1");

      expect(mockDownloadModelFromRegistry).toHaveBeenCalledWith(
        "/models",
        "new-model:v1"
      );
      expect(modelsStore.models).toContainEqual(model);
      expect(modelsStore.successMessage).toContain("meta:new-model:v1");
      expect(modelsStore.isDownloading).toBe(false);
    });

    it("updates existing model instead of duplicating", async () => {
      const existingModel = makeModel({ full_identifier: "meta:model:v1" });
      const updatedModel = makeModel({
        full_identifier: "meta:model:v1",
        name: "updated-model",
      });
      mockDownloadModelFromRegistry.mockResolvedValue(updatedModel);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      modelsStore.models = [existingModel];

      await modelsStore.download("model:v1");

      expect(modelsStore.models).toHaveLength(1);
      expect(modelsStore.models[0].name).toBe("updated-model");
    });

    it("sets error when no modelsRoot", async () => {
      mockSettingsStore.settings.modelsDirectory = null as any;

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.download("model:v1");

      expect(modelsStore.error).toBe(
        "Please select a models directory first"
      );
    });

    it("sets error when reference is empty", async () => {
      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.download("   ");

      expect(modelsStore.error).toBe("Please enter a model reference");
    });

    it("trims whitespace from reference", async () => {
      const model = makeModel();
      mockDownloadModelFromRegistry.mockResolvedValue(model);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.download("  model:v1  ");

      expect(mockDownloadModelFromRegistry).toHaveBeenCalledWith(
        "/models",
        "model:v1"
      );
    });

    it("sets error on download failure", async () => {
      mockDownloadModelFromRegistry.mockRejectedValue(
        new Error("Network timeout")
      );

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.download("model:v1");

      expect(modelsStore.error).toContain("Failed to download model");
      expect(modelsStore.error).toContain("Network timeout");
      expect(modelsStore.isDownloading).toBe(false);
    });

    it("handles non-Error download failure", async () => {
      mockDownloadModelFromRegistry.mockRejectedValue(500);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.download("model:v1");

      expect(modelsStore.error).toContain("500");
    });
  });

  describe("remove", () => {
    it("removes model from list and saves", async () => {
      const model = makeModel({ full_identifier: "meta:model:v1" });

      const { modelsStore } = await import("$lib/stores/models.svelte");
      modelsStore.models = [model];

      await modelsStore.remove(model);

      expect(modelsStore.models).toHaveLength(0);
      expect(mockSaveModelLibrary).toHaveBeenCalled();
      expect(modelsStore.successMessage).toContain("Removed");
      expect(modelsStore.isLoading).toBe(false);
    });

    it("clears selectedModel if removed model was selected", async () => {
      const model = makeModel({ full_identifier: "meta:selected:v1" });

      const { modelsStore } = await import("$lib/stores/models.svelte");
      modelsStore.models = [model];
      modelsStore.selectedModel = model;

      await modelsStore.remove(model);

      expect(modelsStore.selectedModel).toBeNull();
    });

    it("keeps selectedModel if different model removed", async () => {
      const model1 = makeModel({ full_identifier: "meta:m1:v1" });
      const model2 = makeModel({ full_identifier: "meta:m2:v1", name: "m2" });

      const { modelsStore } = await import("$lib/stores/models.svelte");
      modelsStore.models = [model1, model2];
      modelsStore.selectedModel = model1;

      await modelsStore.remove(model2);

      expect(modelsStore.selectedModel).toEqual(model1);
      expect(modelsStore.models).toHaveLength(1);
    });

    it("sets error on failure", async () => {
      mockSaveModelLibrary.mockRejectedValue(new Error("Save failed"));
      const model = makeModel();

      const { modelsStore } = await import("$lib/stores/models.svelte");
      modelsStore.models = [model];

      await modelsStore.remove(model);

      expect(modelsStore.error).toContain("Failed to remove model");
      expect(modelsStore.isLoading).toBe(false);
    });
  });

  describe("refresh", () => {
    it("loads library when modelsRoot is set", async () => {
      const models = [makeModel()];
      mockLoadModelLibrary.mockResolvedValue(models);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.refresh();

      expect(mockLoadModelLibrary).toHaveBeenCalled();
    });

    it("scans when library is empty after load", async () => {
      mockLoadModelLibrary.mockResolvedValue([]);
      mockScanModelsDirectory.mockResolvedValue([makeModel()]);

      const { modelsStore } = await import("$lib/stores/models.svelte");
      modelsStore.models = [];
      await modelsStore.refresh();

      expect(mockScanModelsDirectory).toHaveBeenCalledWith("/models");
    });

    it("skips when no modelsRoot", async () => {
      mockSettingsStore.settings.modelsDirectory = null as any;

      const { modelsStore } = await import("$lib/stores/models.svelte");
      await modelsStore.refresh();

      expect(mockLoadModelLibrary).not.toHaveBeenCalled();
    });
  });

  describe("selectModel", () => {
    it("sets selected model and clears success message", async () => {
      const model = makeModel();

      const { modelsStore } = await import("$lib/stores/models.svelte");
      modelsStore.successMessage = "previous message";

      modelsStore.selectModel(model);

      expect(modelsStore.selectedModel).toEqual(model);
      expect(modelsStore.successMessage).toBe("");
    });
  });

  describe("clearMessages", () => {
    it("clears error and success message", async () => {
      const { modelsStore } = await import("$lib/stores/models.svelte");
      modelsStore.error = "some error";
      modelsStore.successMessage = "some success";

      modelsStore.clearMessages();

      expect(modelsStore.error).toBeNull();
      expect(modelsStore.successMessage).toBe("");
    });
  });

  describe("download events", () => {
    it("updates progress when download:progress event is received", async () => {
      vi.resetModules();
      const { listen } = await import("@tauri-apps/api/event");
      const { modelsStore } = await import("$lib/stores/models.svelte");

      // Get the callback that was passed to listen during construction
      const call = vi.mocked(listen).mock.calls.find(c => c[0] === "download:progress");
      if (!call) throw new Error("listen not called with download:progress");
      const callback = call[1] as (event: { payload: any }) => void;

      // 1. Initial progress
      callback({
        payload: { filename: "test.gguf", downloaded: 100, total: 1000 }
      });

      expect(modelsStore.downloads["test.gguf"]).toBeDefined();
      expect(modelsStore.downloads["test.gguf"].downloaded).toBe(100);

      // 2. Further progress (speed calculation)
      // Mock Date.now to test speed
      const now = Date.now();
      vi.spyOn(Date, 'now').mockReturnValue(now + 1000); // 1 second later

      callback({
        payload: { filename: "test.gguf", downloaded: 300, total: 1000 }
      });

      expect(modelsStore.downloads["test.gguf"].downloaded).toBe(300);
      expect(modelsStore.downloads["test.gguf"].speed).toBe(200); // (300-100) / 1s

      // 3. Completion (should trigger timeout to clear)
      vi.useFakeTimers();
      callback({
        payload: { filename: "test.gguf", downloaded: 1000, total: 1000 }
      });

      expect(modelsStore.downloads["test.gguf"].downloaded).toBe(1000);
      vi.advanceTimersByTime(2000);
      expect(modelsStore.downloads["test.gguf"]).toBeUndefined();
      
      vi.useRealTimers();
      vi.restoreAllMocks();
    });
  });
});
