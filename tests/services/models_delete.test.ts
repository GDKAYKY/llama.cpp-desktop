import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

beforeAll(() => {
  vi.stubGlobal("window", { __TAURI_INTERNALS__: true });
});

describe("models_delete service", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("removeModelByIdentifier", () => {
    it("calls invoke with correct args and returns result", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { removeModelByIdentifier } = await import(
        "$lib/services/models_delete"
      );

      vi.mocked(invoke).mockResolvedValue(true);

      const result = await removeModelByIdentifier(
        "meta:llama3:latest",
        "/models"
      );

      expect(invoke).toHaveBeenCalledWith("remove_model_by_identifier", {
        fullIdentifier: "meta:llama3:latest",
        modelsRoot: "/models",
      });
      expect(result).toBe(true);
    });

    it("returns false when model not found", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { removeModelByIdentifier } = await import(
        "$lib/services/models_delete"
      );

      vi.mocked(invoke).mockResolvedValue(false);

      const result = await removeModelByIdentifier(
        "unknown:model:v1",
        "/models"
      );

      expect(result).toBe(false);
    });

    it("propagates errors", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { removeModelByIdentifier } = await import(
        "$lib/services/models_delete"
      );

      vi.mocked(invoke).mockRejectedValue(new Error("Permission denied"));

      await expect(
        removeModelByIdentifier("meta:llama3:latest", "/models")
      ).rejects.toThrow("Permission denied");
    });
  });

  describe("removeModelByManifestPath", () => {
    it("calls invoke with correct args and returns result", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { removeModelByManifestPath } = await import(
        "$lib/services/models_delete"
      );

      vi.mocked(invoke).mockResolvedValue(true);

      const result = await removeModelByManifestPath(
        "/models/manifests/meta/llama3/latest",
        "/models"
      );

      expect(invoke).toHaveBeenCalledWith("remove_model_by_manifest_path", {
        manifestPath: "/models/manifests/meta/llama3/latest",
        modelsRoot: "/models",
      });
      expect(result).toBe(true);
    });

    it("returns false when manifest not found", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { removeModelByManifestPath } = await import(
        "$lib/services/models_delete"
      );

      vi.mocked(invoke).mockResolvedValue(false);

      const result = await removeModelByManifestPath(
        "/bad/path",
        "/models"
      );

      expect(result).toBe(false);
    });
  });
});
