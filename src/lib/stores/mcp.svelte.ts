import { invokeCommand } from '$infrastructure/ipc';
import type { McpConfig, McpServerConfig, McpServerStatus, ResourceDefinition, ToolDefinition } from '$lib/types/backend';

class McpStore {
    servers = $state<McpServerConfig[]>([]);
    statusMap = $state<Record<string, McpServerStatus>>({});
    toolsMap = $state<Record<string, ToolDefinition[]>>({});
    resourcesMap = $state<Record<string, ResourceDefinition[]>>({});
    configPath = $state<string | null>(null);
    loading = $state(false);
    error = $state<string | null>(null);

    async init() {
        await this.loadConfig();
        await this.refreshStatus();
        await this.loadConfigPath();
    }

    async loadConfig() {
        try {
            this.loading = true;
            this.error = null;
            const config = await invokeCommand('load_mcp_config') as McpConfig;
            this.servers = config.servers ?? [];
        } catch (err) {
            this.error = err instanceof Error ? err.message : String(err);
        } finally {
            this.loading = false;
        }
    }

    async loadConfigPath() {
        try {
            const path = await invokeCommand('get_mcp_config_path_string');
            this.configPath = path as string;
        } catch {
            this.configPath = null;
        }
    }

    async refreshStatus(id?: string) {
        try {
            const statusList = await invokeCommand('mcp_status', id ? { id } : {}) as McpServerStatus[];
            const next: Record<string, McpServerStatus> = {};
            for (const status of statusList) {
                next[status.id] = status;
            }
            this.statusMap = next;
        } catch (err) {
            this.error = err instanceof Error ? err.message : String(err);
        }
    }

    async addServer(server: McpServerConfig) {
        await invokeCommand('mcp_add_server', { server });
        await this.loadConfig();
        await this.refreshStatus();
    }

    async updateServer(server: McpServerConfig) {
        await invokeCommand('mcp_update_server', { server });
        await this.loadConfig();
        await this.refreshStatus(server.id);
    }

    async removeServer(id: string) {
        await invokeCommand('mcp_remove_server', { id });
        await this.loadConfig();
        await this.refreshStatus();
    }

    async connect(id: string) {
        await invokeCommand('mcp_connect', { id });
        await this.refreshStatus(id);
    }

    async disconnect(id: string) {
        await invokeCommand('mcp_disconnect', { id });
        await this.refreshStatus(id);
    }

    async listTools(id: string) {
        const tools = await invokeCommand('mcp_tools_list', { id }) as ToolDefinition[];
        this.toolsMap = { ...this.toolsMap, [id]: tools };
        return tools;
    }

    async listResources(id: string) {
        const resources = await invokeCommand('mcp_resources_list', { id }) as ResourceDefinition[];
        this.resourcesMap = { ...this.resourcesMap, [id]: resources };
        return resources;
    }
}

export const mcpStore = new McpStore();
