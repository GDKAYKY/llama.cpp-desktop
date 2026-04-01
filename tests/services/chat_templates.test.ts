import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

beforeAll(() => {
  vi.stubGlobal("window", { __TAURI_INTERNALS__: true });
});

describe("chat_templates service", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("ensureChatTemplate", () => {
    it("calls invoke with correct command and args", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { ensureChatTemplate } = await import(
        "$lib/services/chat_templates"
      );

      vi.mocked(invoke).mockResolvedValue("/cache/template.jinja");

      const result = await ensureChatTemplate(
        "meta-llama/Llama-3.2-1B-Instruct"
      );

      expect(invoke).toHaveBeenCalledWith("ensure_chat_template", {
        hfRepo: "meta-llama/Llama-3.2-1B-Instruct",
      });
      expect(result).toBe("/cache/template.jinja");
    });

    it("propagates errors from backend", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { ensureChatTemplate } = await import(
        "$lib/services/chat_templates"
      );

      vi.mocked(invoke).mockRejectedValue(new Error("Network error"));

      await expect(
        ensureChatTemplate("invalid/repo")
      ).rejects.toThrow("Network error");
    });
  });

  describe("startLlamaServer", () => {
    it("calls invoke with all required parameters", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { startLlamaServer } = await import(
        "$lib/services/chat_templates"
      );

      vi.mocked(invoke).mockResolvedValue("12345");

      const result = await startLlamaServer({
        binaryPath: "/usr/bin/llama-server",
        modelPath: "/models/model.gguf",
        port: 8080,
        ctxSize: 4096,
        nGpuLayers: 35,
      });

      expect(invoke).toHaveBeenCalledWith("start_llama_server", {
        binaryPath: "/usr/bin/llama-server",
        modelPath: "/models/model.gguf",
        port: 8080,
        ctxSize: 4096,
        nGpuLayers: 35,
        parallel: null,
        chatTemplate: null,
        chatTemplateFile: null,
      });
      expect(result).toBe("12345");
    });

    it("passes optional parameters when provided", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { startLlamaServer } = await import(
        "$lib/services/chat_templates"
      );

      vi.mocked(invoke).mockResolvedValue("12345");

      await startLlamaServer({
        binaryPath: "/usr/bin/llama-server",
        modelPath: "/models/model.gguf",
        port: 8080,
        ctxSize: 4096,
        nGpuLayers: 35,
        parallel: 4,
        chatTemplate: "chatml",
        chatTemplatePath: "/cache/template.jinja",
      });

      expect(invoke).toHaveBeenCalledWith("start_llama_server", {
        binaryPath: "/usr/bin/llama-server",
        modelPath: "/models/model.gguf",
        port: 8080,
        ctxSize: 4096,
        nGpuLayers: 35,
        parallel: 4,
        chatTemplate: "chatml",
        chatTemplateFile: "/cache/template.jinja",
      });
    });
  });

  describe("ensureTemplateAndStartServer", () => {
    it("chains ensureChatTemplate and startLlamaServer", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { ensureTemplateAndStartServer } = await import(
        "$lib/services/chat_templates"
      );

      vi.mocked(invoke)
        .mockResolvedValueOnce("/cache/template.jinja")
        .mockResolvedValueOnce("67890");

      const result = await ensureTemplateAndStartServer(
        "meta-llama/Llama-3.2-1B-Instruct",
        {
          binaryPath: "/usr/bin/llama-server",
          modelPath: "/models/model.gguf",
          port: 8080,
          ctxSize: 4096,
          nGpuLayers: 35,
        }
      );

      expect(invoke).toHaveBeenCalledTimes(2);
      expect(invoke).toHaveBeenNthCalledWith(1, "ensure_chat_template", {
        hfRepo: "meta-llama/Llama-3.2-1B-Instruct",
      });
      expect(invoke).toHaveBeenNthCalledWith(2, "start_llama_server", {
        binaryPath: "/usr/bin/llama-server",
        modelPath: "/models/model.gguf",
        port: 8080,
        ctxSize: 4096,
        nGpuLayers: 35,
        parallel: null,
        chatTemplate: null,
        chatTemplateFile: "/cache/template.jinja",
      });
      expect(result).toBe("67890");
    });

    it("fails if ensureChatTemplate fails", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { ensureTemplateAndStartServer } = await import(
        "$lib/services/chat_templates"
      );

      vi.mocked(invoke).mockRejectedValue(new Error("Template not found"));

      await expect(
        ensureTemplateAndStartServer("bad/repo", {
          binaryPath: "/bin/llama",
          modelPath: "/models/m.gguf",
          port: 8080,
          ctxSize: 4096,
          nGpuLayers: 35,
        })
      ).rejects.toThrow("Template not found");
    });
  });
});
