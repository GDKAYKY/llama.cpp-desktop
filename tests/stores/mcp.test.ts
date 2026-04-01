import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";
import { mcpStore } from "$lib/stores/mcp.svelte";
import type { McpServerConfig } from "$lib/types/backend";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("mcpStore", () => {
  beforeAll(() => {
    vi.stubGlobal("window", { __TAURI_INTERNALS__: true });
  });

  beforeEach(() => {
    vi.clearAllMocks();
    mcpStore.servers = [];
    mcpStore.userServers = [];
    mcpStore.defaultServers = [];
    mcpStore.statusMap = {};
    mcpStore.toolsMap = {};
    mcpStore.resourcesMap = {};
    mcpStore.configPath = null;
    mcpStore.loading = false;
    mcpStore.error = null;
  });

  it("initializes with empty servers", () => {
    expect(mcpStore.servers).toEqual([]);
  });

  it("sets servers list", () => {
    const servers: McpServerConfig[] = [
      {
        id: "server1",
        name: "Test Server 1",
        enabled: true,
        transport: "stdio",
        command: "",
      },
      {
        id: "server2",
        name: "Test Server 2",
        enabled: false,
        transport: "stdio",
        command: "",
      },
    ];

    mcpStore.servers = servers;
    expect(mcpStore.servers.length).toBe(2);
  });

  it("sets loading state", () => {
    mcpStore.loading = true;
    expect(mcpStore.loading).toBe(true);
  });

  it("sets error message", () => {
    mcpStore.error = "Connection failed";
    expect(mcpStore.error).toBe("Connection failed");
  });

  it("clears error", () => {
    mcpStore.error = "Test error";
    mcpStore.error = null;
    expect(mcpStore.error).toBeNull();
  });

  describe("init", () => {
    it("calls all loading methods", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({ servers: [] });

      await mcpStore.init();

      expect(invoke).toHaveBeenCalledWith("load_mcp_config", {});
      expect(invoke).toHaveBeenCalledWith("load_default_mcp_config", {});
      expect(invoke).toHaveBeenCalledWith("status", {});
      expect(invoke).toHaveBeenCalledWith("get_mcp_config_path_string", {});
    });
  });

  describe("rebuildServers", () => {
    it("merges default and user servers, user overrides default", () => {
      mcpStore.defaultServers = [
        {
          id: "s1",
          name: "Default S1",
          enabled: true,
          transport: "stdio",
          command: "cmd1",
        },
        {
          id: "s2",
          name: "Default S2",
          enabled: true,
          transport: "stdio",
          command: "cmd2",
        },
      ];
      mcpStore.userServers = [
        {
          id: "s1",
          name: "User S1",
          enabled: false,
          transport: "http_sse",
          url: "http://localhost",
        },
      ];

      mcpStore.rebuildServers();

      expect(mcpStore.servers).toHaveLength(2);
      const s1 = mcpStore.servers.find((s) => s.id === "s1");
      expect(s1?.name).toBe("User S1");
      expect(s1?.transport).toBe("http_sse");
    });

    it("includes all default servers when no user servers", () => {
      mcpStore.defaultServers = [
        {
          id: "s1",
          name: "Default S1",
          enabled: true,
          transport: "stdio",
          command: "cmd1",
        },
      ];
      mcpStore.userServers = [];

      mcpStore.rebuildServers();

      expect(mcpStore.servers).toHaveLength(1);
      expect(mcpStore.servers[0].name).toBe("Default S1");
    });
  });

  describe("loadConfig", () => {
    it("loads user servers from backend", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const mockConfig = {
        servers: [
          {
            id: "s1",
            name: "User Server",
            enabled: true,
            transport: "stdio",
            command: "test",
          },
        ],
      };
      vi.mocked(invoke).mockResolvedValue(mockConfig);

      await mcpStore.loadConfig();

      expect(invoke).toHaveBeenCalledWith("load_mcp_config", {});
      expect(mcpStore.userServers).toHaveLength(1);
      expect(mcpStore.loading).toBe(false);
      expect(mcpStore.error).toBeNull();
    });

    it("handles missing servers array", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({ servers: undefined });

      await mcpStore.loadConfig();

      expect(mcpStore.userServers).toEqual([]);
    });

    it("sets error on failure", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Config load failed"));

      await mcpStore.loadConfig();

      expect(mcpStore.error).toBe("Config load failed");
      expect(mcpStore.loading).toBe(false);
    });

    it("handles non-Error rejection", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue("string error");

      await mcpStore.loadConfig();

      expect(mcpStore.error).toBe("string error");
    });
  });

  describe("loadDefaultConfig", () => {
    it("loads default servers from backend", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const mockConfig = {
        servers: [
          {
            id: "d1",
            name: "Default",
            enabled: true,
            transport: "stdio",
            command: "default-cmd",
          },
        ],
      };
      vi.mocked(invoke).mockResolvedValue(mockConfig);

      await mcpStore.loadDefaultConfig();

      expect(invoke).toHaveBeenCalledWith("load_default_mcp_config", {});
      expect(mcpStore.defaultServers).toHaveLength(1);
    });

    it("handles missing servers array", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({});

      await mcpStore.loadDefaultConfig();

      expect(mcpStore.defaultServers).toEqual([]);
    });

    it("sets error on failure", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Default config failed"));

      await mcpStore.loadDefaultConfig();

      expect(mcpStore.error).toBe("Default config failed");
    });
  });

  describe("loadConfigPath", () => {
    it("loads config path from backend", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue("/home/.config/mcp.json");

      await mcpStore.loadConfigPath();

      expect(mcpStore.configPath).toBe("/home/.config/mcp.json");
    });

    it("sets null on failure", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Not found"));

      await mcpStore.loadConfigPath();

      expect(mcpStore.configPath).toBeNull();
    });
  });

  describe("refreshStatus", () => {
    it("fetches status for all servers", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const mockStatuses = [
        { id: "s1", connected: true, tools_cached: 5, resources_cached: 2 },
        { id: "s2", connected: false, tools_cached: 0, resources_cached: 0 },
      ];
      vi.mocked(invoke).mockResolvedValue(mockStatuses);

      await mcpStore.refreshStatus();

      expect(invoke).toHaveBeenCalledWith("status", {});
      expect(mcpStore.statusMap["s1"].connected).toBe(true);
      expect(mcpStore.statusMap["s2"].connected).toBe(false);
    });

    it("fetches status for a specific server", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue([
        { id: "s1", connected: true, tools_cached: 3, resources_cached: 1 },
      ]);

      await mcpStore.refreshStatus("s1");

      expect(invoke).toHaveBeenCalledWith("status", { id: "s1" });
    });

    it("sets error on failure", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockRejectedValue(new Error("Status fetch failed"));

      await mcpStore.refreshStatus();

      expect(mcpStore.error).toBe("Status fetch failed");
    });
  });

  describe("listTools", () => {
    it("fetches tools and updates toolsMap", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const mockTools = [{ name: "tool1" }, { name: "tool2" }];
      vi.mocked(invoke).mockResolvedValue(mockTools);

      const result = await mcpStore.listTools("s1");

      expect(invoke).toHaveBeenCalledWith("list_tools", { id: "s1" });
      expect(result).toEqual(mockTools);
      expect(mcpStore.toolsMap["s1"]).toEqual(mockTools);
    });
  });

  describe("listResources", () => {
    it("fetches resources and updates resourcesMap", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const mockResources = [{ name: "res1" }];
      vi.mocked(invoke).mockResolvedValue(mockResources);

      const result = await mcpStore.listResources("s1");

      expect(invoke).toHaveBeenCalledWith("list_resources", { id: "s1" });
      expect(result).toEqual(mockResources);
      expect(mcpStore.resourcesMap["s1"]).toEqual(mockResources);
    });
  });

  describe("addServer", () => {
    it("calls backend and refreshes", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue(undefined);
      // loadConfig returns empty, refreshStatus returns empty
      vi.mocked(invoke).mockResolvedValue({ servers: [] });

      const server: McpServerConfig = {
        id: "new",
        name: "New Server",
        enabled: true,
        transport: "stdio",
        command: "test",
      };

      await mcpStore.addServer(server);

      expect(invoke).toHaveBeenCalledWith("add_server", { server });
    });
  });

  describe("updateServer", () => {
    it("calls backend and refreshes", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({ servers: [] });

      const server: McpServerConfig = {
        id: "s1",
        name: "Updated",
        enabled: true,
        transport: "stdio",
        command: "test",
      };

      await mcpStore.updateServer(server);

      expect(invoke).toHaveBeenCalledWith("update_server", { server });
    });
  });

  describe("removeServer", () => {
    it("calls backend and refreshes", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue({ servers: [] });

      await mcpStore.removeServer("s1");

      expect(invoke).toHaveBeenCalledWith("remove_server", { id: "s1" });
    });
  });

  describe("connect", () => {
    it("calls connect and loads capabilities", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // connect
        .mockResolvedValueOnce([]) // list_tools
        .mockResolvedValueOnce([]) // list_resources
        .mockResolvedValueOnce([]); // status

      await mcpStore.connect("s1");

      expect(invoke).toHaveBeenCalledWith("connect", { id: "s1" });
    });

    it("sets error if tools loading fails", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // connect
        .mockRejectedValueOnce(new Error("Tools failed")) // list_tools
        .mockResolvedValueOnce([]) // list_resources
        .mockResolvedValueOnce([]); // status

      await mcpStore.connect("s1");

      expect(mcpStore.error).toBe("Tools failed");
    });

    it("sets error if resources loading fails", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke)
        .mockResolvedValueOnce(undefined) // connect
        .mockResolvedValueOnce([]) // list_tools
        .mockRejectedValueOnce(new Error("Resources failed")) // list_resources
        .mockResolvedValueOnce([]); // status

      await mcpStore.connect("s1");

      expect(mcpStore.error).toBe("Resources failed");
    });
  });

  describe("disconnect", () => {
    it("calls disconnect and refreshes status", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      vi.mocked(invoke).mockResolvedValue([]);

      await mcpStore.disconnect("s1");

      expect(invoke).toHaveBeenCalledWith("disconnect", { id: "s1" });
    });
  });

  describe("loadCapabilitiesForConnected", () => {
    it("loads tools and resources for connected servers", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      mcpStore.statusMap = {
        s1: { id: "s1", connected: true, tools_cached: 0, resources_cached: 0 },
        s2: {
          id: "s2",
          connected: false,
          tools_cached: 0,
          resources_cached: 0,
        },
      };
      vi.mocked(invoke)
        .mockResolvedValueOnce([{ name: "tool1" }]) // tools for s1
        .mockResolvedValueOnce([{ name: "res1" }]); // resources for s1

      await mcpStore.loadCapabilitiesForConnected();

      expect(invoke).toHaveBeenCalledWith("list_tools", { id: "s1" });
      expect(invoke).toHaveBeenCalledWith("list_resources", { id: "s1" });
      expect(invoke).not.toHaveBeenCalledWith("list_tools", { id: "s2" });
    });

    it("skips when no connected servers", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      mcpStore.statusMap = {
        s1: {
          id: "s1",
          connected: false,
          tools_cached: 0,
          resources_cached: 0,
        },
      };

      await mcpStore.loadCapabilitiesForConnected();

      expect(invoke).not.toHaveBeenCalled();
    });

    it("sets error on rejected promise", async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      mcpStore.statusMap = {
        s1: { id: "s1", connected: true, tools_cached: 0, resources_cached: 0 },
      };
      vi.mocked(invoke).mockRejectedValue(new Error("Capability load failed"));

      await mcpStore.loadCapabilitiesForConnected();

      expect(mcpStore.error).toBe("Capability load failed");
    });
  });
});
