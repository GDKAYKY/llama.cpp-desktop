import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

beforeAll(() => {
  vi.stubGlobal("window", { __TAURI_INTERNALS__: true });
});

describe("model_downloads service", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("downloadModelFromRegistry calls invoke with correct args", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { downloadModelFromRegistry } = await import(
      "$lib/services/model_downloads"
    );

    const mockModel = {
      name: "llama3",
      version: "latest",
      provider: "meta",
      library: "ollama",
      full_identifier: "meta:llama3:latest",
      manifest_data: {
        config: { mediaType: "application/json", digest: "sha256:abc", size: 100 },
        layers: [],
      },
    };

    vi.mocked(invoke).mockResolvedValue(mockModel);

    const result = await downloadModelFromRegistry("/models", "llama3:latest");

    expect(invoke).toHaveBeenCalledWith("download_model_from_registry", {
      modelsRoot: "/models",
      modelReference: "llama3:latest",
    });
    expect(result).toEqual(mockModel);
  });

  it("propagates errors from backend", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { downloadModelFromRegistry } = await import(
      "$lib/services/model_downloads"
    );

    vi.mocked(invoke).mockRejectedValue(new Error("Download failed"));

    await expect(
      downloadModelFromRegistry("/models", "bad:ref")
    ).rejects.toThrow("Download failed");
  });
});
