import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

beforeAll(() => {
  vi.stubGlobal("window", { __TAURI_INTERNALS__: true });
});

describe("settingsStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("initializes with default settings", async () => {
    const { settingsStore } = await import("$lib/stores/settings.svelte");
    expect(settingsStore.settings).toBeDefined();
    expect(settingsStore.settings.modelsDirectory).toBeDefined();
  });

  it("has correct default values", async () => {
    const { settingsStore } = await import("$lib/stores/settings.svelte");
    expect(settingsStore.settings.theme).toBe("dark");
    expect(settingsStore.settings.language).toBe("en");
    expect(settingsStore.settings.maxTokens).toBe(2048);
    expect(settingsStore.settings.contextSize).toBe(8192);
    expect(settingsStore.settings.temperature).toBe(0.7);
    expect(settingsStore.settings.autoSaveChat).toBe(true);
    expect(settingsStore.settings.chatHistoryLimit).toBe(50);
    expect(settingsStore.settings.serverPort).toBe(8080);
  });

  it("updates settings and calls saveConfig", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { settingsStore } = await import("$lib/stores/settings.svelte");

    vi.mocked(invoke).mockResolvedValue(undefined);

    await settingsStore.update({ modelsDirectory: "/test/models" });

    expect(settingsStore.settings.modelsDirectory).toBe("/test/models");
    expect(invoke).toHaveBeenCalledWith(
      "save_config",
      expect.objectContaining({
        config: expect.objectContaining({ modelsDirectory: "/test/models" }),
      })
    );
  });

  it("merges partial updates with existing settings", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { settingsStore } = await import("$lib/stores/settings.svelte");

    vi.mocked(invoke).mockResolvedValue(undefined);

    await settingsStore.update({ theme: "light" });

    expect(settingsStore.settings.theme).toBe("light");
    expect(settingsStore.settings.language).toBe("en");
  });

  it("sets error on update failure", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { settingsStore } = await import("$lib/stores/settings.svelte");

    vi.mocked(invoke).mockRejectedValue(new Error("Write failed"));

    await settingsStore.update({ theme: "light" });

    expect(settingsStore.error).toBe("Failed to update settings");
  });

  it("clears error messages", async () => {
    const { settingsStore } = await import("$lib/stores/settings.svelte");
    settingsStore.error = "test error";
    settingsStore.error = null;
    expect(settingsStore.error).toBeNull();
  });

  it("init loads config from backend", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { settingsStore } = await import("$lib/stores/settings.svelte");

    const customConfig = {
      modelsDirectory: "/custom/models",
      llamaDirectory: "/custom/llama",
      theme: "light",
      language: "pt",
      maxTokens: 4096,
      contextSize: 16384,
      temperature: 0.5,
      autoSaveChat: false,
      chatHistoryLimit: 100,
      serverPort: 9090,
      webSearchProvider: "tavily" as const,
      webSearchMcpId: null,
      chatHeaderStyle: "capsule" as const,
    };
    vi.mocked(invoke).mockResolvedValue(customConfig);

    await settingsStore.init();

    expect(settingsStore.settings.modelsDirectory).toBe("/custom/models");
    expect(settingsStore.settings.theme).toBe("light");
    expect(settingsStore.isLoading).toBe(false);
    expect(settingsStore.error).toBeNull();
  });

  it("init sets error on failure", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { settingsStore } = await import("$lib/stores/settings.svelte");

    vi.mocked(invoke).mockRejectedValue(new Error("Load failed"));

    await settingsStore.init();

    expect(settingsStore.error).toBe("Failed to load settings");
    expect(settingsStore.isLoading).toBe(false);
  });

  it("reset calls resetConfig and updates settings", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { settingsStore } = await import("$lib/stores/settings.svelte");

    const resetResult = {
      modelsDirectory: null,
      llamaDirectory: null,
      theme: "dark",
      language: "en",
      maxTokens: 2048,
      contextSize: 8192,
      temperature: 0.7,
      autoSaveChat: true,
      chatHistoryLimit: 50,
      serverPort: 8080,
      webSearchProvider: "tavily" as const,
      webSearchMcpId: null,
      chatHeaderStyle: "default" as const,
    };
    vi.mocked(invoke).mockResolvedValue(resetResult);

    // First change a setting
    settingsStore.settings.theme = "light";
    await settingsStore.reset();

    expect(invoke).toHaveBeenCalledWith("reset_config", {});
    expect(settingsStore.settings.theme).toBe("dark");
    expect(settingsStore.error).toBeNull();
  });

  it("reset sets error on failure", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { settingsStore } = await import("$lib/stores/settings.svelte");

    vi.mocked(invoke).mockRejectedValue(new Error("Reset failed"));

    await settingsStore.reset();

    expect(settingsStore.error).toBe("Failed to reset settings");
  });
});
