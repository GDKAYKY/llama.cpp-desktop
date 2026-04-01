import { invokeCommand } from "$infrastructure/ipc";
import type {
  McpConfig,
  McpServerConfig,
  McpServerStatus,
  ResourceDefinition,
  ToolDefinition,
} from "$lib/types/backend";

class McpStore {
  servers = $state<McpServerConfig[]>([]);
  userServers = $state<McpServerConfig[]>([]);
  defaultServers = $state<McpServerConfig[]>([]);
  statusMap = $state<Record<string, McpServerStatus>>({});
  toolsMap = $state<Record<string, ToolDefinition[]>>({});
  resourcesMap = $state<Record<string, ResourceDefinition[]>>({});
  configPath = $state<string | null>(null);
  loading = $state(false);
  error = $state<string | null>(null);

  async init() {
    await this.loadConfig();
    await this.loadDefaultConfig();
    await this.refreshStatus();
    await this.loadCapabilitiesForConnected();
    await this.loadConfigPath();
  }

  async loadConfig() {
    try {
      this.loading = true;
      this.error = null;
      const config = (await invokeCommand("load_mcp_config")) as McpConfig;
      this.userServers = config.servers ?? [];
      this.rebuildServers();
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    } finally {
      this.loading = false;
    }
  }

  async loadDefaultConfig() {
    try {
      const config = (await invokeCommand(
        "load_default_mcp_config",
      )) as McpConfig;
      this.defaultServers = config.servers ?? [];
      this.rebuildServers();
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    }
  }

  rebuildServers() {
    const map = new Map<string, McpServerConfig>();
    for (const server of this.defaultServers) {
      map.set(server.id, server);
    }
    for (const server of this.userServers) {
      map.set(server.id, server);
    }
    this.servers = Array.from(map.values());
  }

  async loadConfigPath() {
    try {
      const path = await invokeCommand("get_mcp_config_path_string");
      this.configPath = path as string;
    } catch {
      this.configPath = null;
    }
  }

  async refreshStatus(id?: string) {
    try {
      const statusList = (await invokeCommand(
        "status",
        id ? { id } : {},
      )) as McpServerStatus[];
      const next: Record<string, McpServerStatus> = {};
      for (const status of statusList) {
        next[status.id] = status;
      }
      this.statusMap = next;
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    }
  }

  async loadCapabilitiesForConnected() {
    const connectedIds = Object.values(this.statusMap)
      .filter((status) => status.connected)
      .map((status) => status.id);
    if (connectedIds.length === 0) {
      return;
    }

    const results = await Promise.allSettled(
      connectedIds.map(async (id) => {
        await this.listTools(id);
        await this.listResources(id);
      }),
    );

    for (const result of results) {
      if (result.status === "rejected") {
        this.error =
          result.reason instanceof Error
            ? result.reason.message
            : String(result.reason);
        break;
      }
    }
  }

  async addServer(server: McpServerConfig) {
    await invokeCommand("add_server", { server });
    await this.loadConfig();
    await this.refreshStatus();
  }

  async updateServer(server: McpServerConfig) {
    await invokeCommand("update_server", { server });
    await this.loadConfig();
    await this.refreshStatus(server.id);
  }

  async removeServer(id: string) {
    await invokeCommand("remove_server", { id });
    await this.loadConfig();
    await this.refreshStatus();
  }

  async connect(id: string) {
    await invokeCommand("connect", { id });
    const [toolsResult, resourcesResult] = await Promise.allSettled([
      this.listTools(id),
      this.listResources(id),
    ]);
    if (toolsResult.status === "rejected") {
      this.error =
        toolsResult.reason instanceof Error
          ? toolsResult.reason.message
          : String(toolsResult.reason);
    } else if (resourcesResult.status === "rejected") {
      this.error =
        resourcesResult.reason instanceof Error
          ? resourcesResult.reason.message
          : String(resourcesResult.reason);
    }
    await this.refreshStatus(id);
  }

  async disconnect(id: string) {
    await invokeCommand("disconnect", { id });
    await this.refreshStatus(id);
  }

  async listTools(id: string) {
    const tools = (await invokeCommand("list_tools", {
      id,
    })) as ToolDefinition[];
    this.toolsMap = { ...this.toolsMap, [id]: tools };
    return tools;
  }

  async listResources(id: string) {
    const resources = (await invokeCommand("list_resources", {
      id,
    })) as ResourceDefinition[];
    this.resourcesMap = { ...this.resourcesMap, [id]: resources };
    return resources;
  }
}

export const mcpStore = new McpStore();
