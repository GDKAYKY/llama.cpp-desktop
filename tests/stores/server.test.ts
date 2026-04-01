import { describe, it, expect, vi, beforeEach, beforeAll, afterEach } from "vitest";
import { serverStore } from "$lib/stores/server.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("serverStore", () => {
  beforeAll(() => {
    vi.stubGlobal("window", {
      __TAURI_INTERNALS__: true,
      location: { pathname: "/models" },
    });
  });

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    serverStore.isRunning = false;
    serverStore.isHealthy = false;
    serverStore.error = null;
    serverStore.isChecking = false;
    serverStore.isStarting = false;
    serverStore.currentConfig = null;
    serverStore.serverMetrics = null;
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("initializes with default state", () => {
    expect(serverStore.isRunning).toBe(false);
    expect(serverStore.isHealthy).toBe(false);
    expect(serverStore.error).toBeNull();
    expect(serverStore.isStarting).toBe(false);
    expect(serverStore.currentConfig).toBeNull();
    expect(serverStore.serverMetrics).toBeNull();
  });

  it("sets running state", () => {
    serverStore.isRunning = true;
    expect(serverStore.isRunning).toBe(true);
  });

  it("sets starting state", () => {
    serverStore.isStarting = true;
    expect(serverStore.isStarting).toBe(true);
  });

  it("sets error message", () => {
    serverStore.error = "Test error";
    expect(serverStore.error).toBe("Test error");
  });

  it("clears error", () => {
    serverStore.error = "Test error";
    serverStore.error = null;
    expect(serverStore.error).toBeNull();
  });

  describe("startServer", () => {
    it("starts server and updates state", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue("pid-123");

      await serverStore.startServer(
        "/bin/llama",
        "/models/model.gguf",
        8000,
        4096,
        33,
        1,
      );

      expect(invoke).toHaveBeenCalledWith("start_llama_server", {
        binaryPath: "/bin/llama",
        modelPath: "/models/model.gguf",
        port: 8000,
        ctxSize: 4096,
        nGpuLayers: 33,
        parallel: 1,
        chatTemplate: undefined,
        chatTemplateFile: undefined,
      });
      expect(serverStore.isRunning).toBe(true);
      expect(serverStore.isStarting).toBe(false);
      expect(serverStore.currentConfig).toBeDefined();
      expect(serverStore.currentConfig?.port).toBe(8000);
    });

    it("skips if already starting", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      serverStore.isStarting = true;

      await serverStore.startServer("/bin/llama", "/models/m.gguf");

      expect(invoke).not.toHaveBeenCalled();
    });

    it("skips if same config is already running", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      serverStore.isRunning = true;
      serverStore.currentConfig = {
        llama_cpp_path: "/bin/llama" as any,
        model_path: "/models/m.gguf" as any,
        port: 8000,
        ctx_size: 4096,
        n_gpu_layers: 33,
        parallel: 1,
        chat_template: null,
        chat_template_file: null,
      };

      await serverStore.startServer(
        "/bin/llama",
        "/models/m.gguf",
        8000,
        4096,
        33,
        1,
      );

      expect(invoke).not.toHaveBeenCalled();
    });

    it("sets error on failure", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Start failed"));

      await serverStore.startServer("/bin/llama", "/models/m.gguf");

      expect(serverStore.error).toBe("Start failed");
      expect(serverStore.isRunning).toBe(false);
      expect(serverStore.isStarting).toBe(false);
    });

    it("handles non-Error rejection", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue("raw error");

      await serverStore.startServer("/bin/llama", "/models/m.gguf");

      expect(serverStore.error).toBe("raw error");
    });

    it("passes chat template parameters", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue("pid-456");

      await serverStore.startServer(
        "/bin/llama",
        "/models/m.gguf",
        8000,
        4096,
        33,
        1,
        "chatml",
        "/cache/template.jinja",
      );

      expect(invoke).toHaveBeenCalledWith("start_llama_server", expect.objectContaining({
        chatTemplate: "chatml",
        chatTemplateFile: "/cache/template.jinja",
      }));
      expect(serverStore.currentConfig?.chat_template).toBe("chatml");
      expect(serverStore.currentConfig?.chat_template_file).toBe("/cache/template.jinja");
    });
  });

  describe("stopServer", () => {
    it("stops server and resets state", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue(undefined);

      serverStore.isRunning = true;
      serverStore.isHealthy = true;
      serverStore.currentConfig = { llama_cpp_path: "x" } as any;

      await serverStore.stopServer();

      expect(invoke).toHaveBeenCalledWith("stop_llama_server", {});
      expect(serverStore.isRunning).toBe(false);
      expect(serverStore.isHealthy).toBe(false);
      expect(serverStore.currentConfig).toBeNull();
      expect(serverStore.serverMetrics).toBeNull();
    });

    it("sets error on failure", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Stop failed"));

      await serverStore.stopServer();

      expect(serverStore.error).toBe("Stop failed");
    });
  });

  describe("checkHealth", () => {
    it("sets healthy when detail reports healthy", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({ healthy: true });

      serverStore.isRunning = true;
      await serverStore.checkHealth();

      expect(serverStore.isHealthy).toBe(true);
      expect(serverStore.error).toBeNull();
    });

    it("sets error when detail reports unhealthy", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({
        healthy: false,
        error: "Connection refused",
        url: "http://localhost:8000",
      });

      serverStore.isRunning = true;
      await serverStore.checkHealth();

      expect(serverStore.isHealthy).toBe(false);
      expect(serverStore.error).toContain("Connection refused");
      expect(serverStore.error).toContain("http://localhost:8000");
    });

    it("sets generic error when no error detail", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({ healthy: false });

      serverStore.isRunning = true;
      await serverStore.checkHealth();

      expect(serverStore.error).toBe("Server health check failed");
    });

    it("resets health when server not running", async () => {
      serverStore.isRunning = false;
      await serverStore.checkHealth();

      expect(serverStore.isHealthy).toBe(false);
      expect(serverStore.error).toBeNull();
    });

    it("sets error on exception", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Network error"));

      serverStore.isRunning = true;
      await serverStore.checkHealth();

      expect(serverStore.isHealthy).toBe(false);
      expect(serverStore.error).toBe("Network error");
    });
  });

  describe("checkRunning", () => {
    it("detects running server and loads config", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const mockConfig = {
        llama_cpp_path: "/bin/llama",
        model_path: "/models/m.gguf",
        port: 8000,
        ctx_size: 4096,
        parallel: 1,
        n_gpu_layers: 33,
      };
      vi.mocked(invoke)
        .mockResolvedValueOnce(true)
        .mockResolvedValueOnce(mockConfig);

      await serverStore.checkRunning();

      expect(serverStore.isRunning).toBe(true);
      expect(serverStore.currentConfig).toEqual(mockConfig);
    });

    it("detects stopped server", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue(false);

      await serverStore.checkRunning();

      expect(serverStore.isRunning).toBe(false);
      expect(serverStore.currentConfig).toBeNull();
    });

    it("handles error during check", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Check failed"));

      await serverStore.checkRunning();

      expect(serverStore.isRunning).toBe(false);
      expect(serverStore.currentConfig).toBeNull();
    });
  });

  describe("fetchMetrics", () => {
    it("fetches metrics when running", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const mockMetrics = { cpu_usage: 50, mem_usage: 30 };
      vi.mocked(invoke).mockResolvedValue(mockMetrics);

      serverStore.isRunning = true;
      await serverStore.fetchMetrics();

      expect(invoke).toHaveBeenCalledWith("get_server_metrics", {});
      expect(serverStore.serverMetrics).toEqual(mockMetrics);
    });

    it("skips when not running", async () => {
      const { invoke } = await import("@tauri-apps/api/core");

      serverStore.isRunning = false;
      await serverStore.fetchMetrics();

      expect(invoke).not.toHaveBeenCalled();
    });

    it("handles error silently", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Metrics error"));

      serverStore.isRunning = true;
      await serverStore.fetchMetrics();

      // Should not throw
    });
  });

  describe("getStatus", () => {
    it("returns full status object", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke)
        .mockResolvedValueOnce(true)                      // is_server_running
        .mockResolvedValueOnce({ port: 8000 })            // get_llama_config
        .mockResolvedValueOnce({ healthy: true });        // check_server_health_detail

      const status = await serverStore.getStatus();

      expect(status.isRunning).toBe(true);
      expect(status.isHealthy).toBe(true);
      expect(status.error).toBeNull();
      expect(status.currentConfig).toBeDefined();
    });

    it("returns stopped status", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue(false);

      const status = await serverStore.getStatus();

      expect(status.isRunning).toBe(false);
      expect(status.currentConfig).toBeNull();
    });
  });

  describe("health monitoring", () => {
    it("starts monitoring and calls checkHealth", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({ healthy: true });

      serverStore.isRunning = true;
      serverStore.startHealthMonitoring();

      vi.advanceTimersByTime(2000);

      expect(invoke).toHaveBeenCalledWith("check_server_health_detail", {});
    });

    it("stops monitoring when server is not running", async () => {
      serverStore.isRunning = false;
      serverStore.startHealthMonitoring();

      vi.advanceTimersByTime(2000);

      // The interval should be cleared
      expect(serverStore.isRunning).toBe(false);
    });

    it("fetches metrics when on models page", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({ healthy: true });

      vi.stubGlobal("window", {
        __TAURI_INTERNALS__: true,
        location: { pathname: "/models" },
      });
      
      serverStore.isRunning = true;
      serverStore.startHealthMonitoring();

      await vi.advanceTimersByTimeAsync(2000);

      expect(invoke).toHaveBeenCalledWith("get_server_metrics", {});
    });
  });
});
