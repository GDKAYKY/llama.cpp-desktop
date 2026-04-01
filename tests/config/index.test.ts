import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";
import { DEFAULT_CONFIG } from "$lib/config/defaultConfig";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

beforeAll(() => {
  vi.stubGlobal("window", { __TAURI_INTERNALS__: true });
});

describe("config module", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("DEFAULT_CONFIG", () => {
    it("has modelsDirectory as null", () => {
      expect(DEFAULT_CONFIG.modelsDirectory).toBeNull();
    });

    it("has llamaDirectory as null", () => {
      expect(DEFAULT_CONFIG.llamaDirectory).toBeNull();
    });

    it("is a valid config object", () => {
      expect(DEFAULT_CONFIG).toBeDefined();
      expect(typeof DEFAULT_CONFIG).toBe("object");
    });

    it("has expected default values", () => {
      expect(DEFAULT_CONFIG.theme).toBe("dark");
      expect(DEFAULT_CONFIG.language).toBe("en");
      expect(DEFAULT_CONFIG.maxTokens).toBe(2048);
      expect(DEFAULT_CONFIG.contextSize).toBe(8192);
      expect(DEFAULT_CONFIG.temperature).toBe(0.7);
      expect(DEFAULT_CONFIG.autoSaveChat).toBe(true);
      expect(DEFAULT_CONFIG.chatHistoryLimit).toBe(50);
      expect(DEFAULT_CONFIG.serverPort).toBe(8080);
      expect(DEFAULT_CONFIG.webSearchProvider).toBe("tavily");
      expect(DEFAULT_CONFIG.webSearchMcpId).toBeNull();
      expect(DEFAULT_CONFIG.chatHeaderStyle).toBe("default");
    });
  });

  describe("loadConfig", () => {
    it("calls invoke and returns config", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { loadConfig } = await import("$lib/config/index");

      const mockConfig = { ...DEFAULT_CONFIG, theme: "light" };
      vi.mocked(invoke).mockResolvedValue(mockConfig);

      const result = await loadConfig();

      expect(invoke).toHaveBeenCalledWith("load_config", {});
      expect(result).toEqual(mockConfig);
    });

    it("propagates errors", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { loadConfig } = await import("$lib/config/index");

      vi.mocked(invoke).mockRejectedValue(new Error("File not found"));

      await expect(loadConfig()).rejects.toThrow("File not found");
    });
  });

  describe("saveConfig", () => {
    it("calls invoke with config object", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { saveConfig } = await import("$lib/config/index");

      vi.mocked(invoke).mockResolvedValue(undefined);

      await saveConfig(DEFAULT_CONFIG);

      expect(invoke).toHaveBeenCalledWith("save_config", {
        config: DEFAULT_CONFIG,
      });
    });

    it("propagates errors", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { saveConfig } = await import("$lib/config/index");

      vi.mocked(invoke).mockRejectedValue(new Error("Write failed"));

      await expect(saveConfig(DEFAULT_CONFIG)).rejects.toThrow("Write failed");
    });
  });

  describe("resetConfig", () => {
    it("calls invoke and returns reset config", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { resetConfig } = await import("$lib/config/index");

      vi.mocked(invoke).mockResolvedValue(DEFAULT_CONFIG);

      const result = await resetConfig();

      expect(invoke).toHaveBeenCalledWith("reset_config", {});
      expect(result).toEqual(DEFAULT_CONFIG);
    });
  });

  describe("getConfigPath", () => {
    it("calls invoke and returns path", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { getConfigPath } = await import("$lib/config/index");

      vi.mocked(invoke).mockResolvedValue("/home/user/.config/llama/config.json");

      const result = await getConfigPath();

      expect(invoke).toHaveBeenCalledWith("get_config_path_string", {});
      expect(result).toBe("/home/user/.config/llama/config.json");
    });
  });
});
