import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

beforeAll(() => {
  vi.stubGlobal("window", { __TAURI_INTERNALS__: true });
});

describe("models service", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("IPC wrappers", () => {
    it("scanModelsDirectory calls backend", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { scanModelsDirectory } = await import("$lib/services/models");

      vi.mocked(invoke).mockResolvedValue([
        { name: "model1", provider: "test" },
      ]);

      const models = await scanModelsDirectory("/models");

      expect(invoke).toHaveBeenCalledWith("scan_models_directory", {
        modelsRoot: "/models",
      });
      expect(models).toHaveLength(1);
    });

    it("parseModelManifest calls backend with correct args", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { parseModelManifest } = await import("$lib/services/models");

      const mockModel = { name: "test-model", provider: "meta" };
      vi.mocked(invoke).mockResolvedValue(mockModel);

      const result = await parseModelManifest(
        "/models/manifests/meta/llama3/latest",
        "/models"
      );

      expect(invoke).toHaveBeenCalledWith("parse_model_manifest", {
        modelPath: "/models/manifests/meta/llama3/latest",
        modelsRoot: "/models",
      });
      expect(result).toEqual(mockModel);
    });

    it("loadModelLibrary calls backend", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { loadModelLibrary } = await import("$lib/services/models");

      vi.mocked(invoke).mockResolvedValue([]);

      await loadModelLibrary("/models/library.json");

      expect(invoke).toHaveBeenCalledWith("load_model_library", {
        libraryPath: "/models/library.json",
      });
    });

    it("saveModelLibrary calls backend", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { saveModelLibrary } = await import("$lib/services/models");

      vi.mocked(invoke).mockResolvedValue(undefined);

      await saveModelLibrary("/models/library.json", []);

      expect(invoke).toHaveBeenCalledWith("save_model_library", {
        libraryPath: "/models/library.json",
        models: [],
      });
    });
  });

  describe("selectModelsDirectory", () => {
    it("returns selected directory path", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { selectModelsDirectory } = await import("$lib/services/models");

      vi.mocked(open).mockResolvedValue("/selected/dir");

      const result = await selectModelsDirectory();

      expect(open).toHaveBeenCalledWith({
        directory: true,
        multiple: false,
        title: "Select Models Directory",
      });
      expect(result).toBe("/selected/dir");
    });

    it("returns null when user cancels", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { selectModelsDirectory } = await import("$lib/services/models");

      vi.mocked(open).mockResolvedValue(null);

      const result = await selectModelsDirectory();
      expect(result).toBeNull();
    });

    it("wraps dialog errors", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { selectModelsDirectory } = await import("$lib/services/models");

      vi.mocked(open).mockRejectedValue(new Error("Dialog unavailable"));

      await expect(selectModelsDirectory()).rejects.toThrow(
        "Failed to open directory dialog: Dialog unavailable"
      );
    });

    it("wraps non-Error dialog errors", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { selectModelsDirectory } = await import("$lib/services/models");

      vi.mocked(open).mockRejectedValue("raw string error");

      await expect(selectModelsDirectory()).rejects.toThrow(
        "Failed to open directory dialog: raw string error"
      );
    });
  });

  describe("selectLlamaDirectory", () => {
    it("returns selected directory path", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { selectLlamaDirectory } = await import("$lib/services/models");

      vi.mocked(open).mockResolvedValue("/selected/llama");

      const result = await selectLlamaDirectory();

      expect(open).toHaveBeenCalledWith({
        directory: true,
        multiple: false,
        title: "Select Llama Directory",
      });
      expect(result).toBe("/selected/llama");
    });

    it("returns null when user cancels", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { selectLlamaDirectory } = await import("$lib/services/models");

      vi.mocked(open).mockResolvedValue(null);

      const result = await selectLlamaDirectory();
      expect(result).toBeNull();
    });

    it("wraps dialog errors", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { selectLlamaDirectory } = await import("$lib/services/models");

      vi.mocked(open).mockRejectedValue(new Error("Platform error"));

      await expect(selectLlamaDirectory()).rejects.toThrow(
        "Failed to open directory dialog: Platform error"
      );
    });

    it("wraps non-Error dialog errors", async () => {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { selectLlamaDirectory } = await import("$lib/services/models");

      vi.mocked(open).mockRejectedValue(42);

      await expect(selectLlamaDirectory()).rejects.toThrow(
        "Failed to open directory dialog: 42"
      );
    });
  });

  describe("formatModelIdentifier", () => {
    it("formats provider:name:version", async () => {
      const { formatModelIdentifier } = await import("$lib/services/models");

      expect(formatModelIdentifier("meta", "llama3", "latest")).toBe(
        "meta:llama3:latest"
      );
    });

    it("works with empty strings", async () => {
      const { formatModelIdentifier } = await import("$lib/services/models");

      expect(formatModelIdentifier("", "", "")).toBe("::");
    });
  });

  describe("parseModelIdentifier", () => {
    it("parses valid identifier", async () => {
      const { parseModelIdentifier } = await import("$lib/services/models");

      const result = parseModelIdentifier("meta:llama3:latest");

      expect(result).toEqual({
        provider: "meta",
        name: "llama3",
        version: "latest",
      });
    });

    it("throws on invalid format with too few parts", async () => {
      const { parseModelIdentifier } = await import("$lib/services/models");

      expect(() => parseModelIdentifier("meta:llama3")).toThrow(
        "Invalid model identifier format"
      );
    });

    it("throws on invalid format with too many parts", async () => {
      const { parseModelIdentifier } = await import("$lib/services/models");

      expect(() => parseModelIdentifier("a:b:c:d")).toThrow(
        "Invalid model identifier format"
      );
    });

    it("throws on empty string", async () => {
      const { parseModelIdentifier } = await import("$lib/services/models");

      expect(() => parseModelIdentifier("")).toThrow(
        "Invalid model identifier format"
      );
    });
  });

  describe("getModelBlobPath", () => {
    it("builds blob path replacing colon with dash", async () => {
      const { getModelBlobPath } = await import("$lib/services/models");

      const result = getModelBlobPath("/models", "sha256:abc123");

      expect(result).toBe("/models/blobs/sha256-abc123");
    });

    it("handles digest without colon", async () => {
      const { getModelBlobPath } = await import("$lib/services/models");

      const result = getModelBlobPath("/models", "abc123");

      expect(result).toBe("/models/blobs/abc123");
    });
  });
});
