import { describe, it, expect, vi, beforeEach } from "vitest";
import { settingsStore } from "$lib/stores/settings.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("settingsStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("initializes with default settings", () => {
    expect(settingsStore.settings).toBeDefined();
    expect(settingsStore.settings.modelsDirectory).toBeNull();
  });

  it("updates settings", async () => {
    const newSettings = {
      modelsDirectory: "/test/models",
      llamaCppPath: "/test/llama",
    };

    await settingsStore.update(newSettings);
    expect(settingsStore.settings.modelsDirectory).toBe("/test/models");
  });

  it("clears error messages", () => {
    settingsStore.error = "test error";
    settingsStore.error = null;
    expect(settingsStore.error).toBeNull();
  });
});
