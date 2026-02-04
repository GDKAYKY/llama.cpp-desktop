<script lang="ts">
  import { onMount } from "svelte";
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import type { McpServerConfig, ToolDefinition, ResourceDefinition } from "$lib/types/backend";
  import { openPath } from "@tauri-apps/plugin-opener";
  import {
    Plug,
    Server,
    Link,
    Unlink,
    Plus,
    Save,
    Trash2,
    RefreshCw,
    List,
    KeyRound,
    Cable,
    FileCode,
  } from "lucide-svelte";

  let selectedId = $state<string | null>(null);
  let saving = $state(false);
  let message = $state<{ type: string; text: string }>({ type: "", text: "" });

  let form = $state({
    id: "",
    name: "",
    enabled: true,
    transport: "stdio",
    command: "",
    args: "",
    cwd: "",
    env: "",
    url: "",
    headers: "",
    tool_allowlist: "",
    resource_allowlist: "",
  });

  onMount(async () => {
    await mcpStore.init();
    if (mcpStore.servers.length > 0) {
      selectServer(mcpStore.servers[0]);
    }
  });

  function resetForm() {
    form = {
      id: "",
      name: "",
      enabled: true,
      transport: "stdio",
      command: "",
      args: "",
      cwd: "",
      env: "",
      url: "",
      headers: "",
      tool_allowlist: "",
      resource_allowlist: "",
    };
    selectedId = null;
  }

  function selectServer(server: McpServerConfig) {
    selectedId = server.id;
    form = {
      id: server.id,
      name: server.name,
      enabled: server.enabled,
      transport: server.transport,
      command: server.command ?? "",
      args: (server.args ?? []).join("\n"),
      cwd: server.cwd ?? "",
      env: mapToText(server.env ?? {}),
      url: server.url ?? "",
      headers: mapToText(server.headers ?? {}),
      tool_allowlist: (server.tool_allowlist ?? []).join("\n"),
      resource_allowlist: (server.resource_allowlist ?? []).join("\n"),
    };
  }

  function mapToText(map: Record<string, string>) {
    return Object.entries(map)
      .map(([k, v]) => `${k}=${v}`)
      .join("\n");
  }

  function textToMap(value: string) {
    const map: Record<string, string> = {};
    value
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean)
      .forEach((line) => {
        const idx = line.indexOf("=");
        if (idx > 0) {
          const key = line.slice(0, idx).trim();
          const val = line.slice(idx + 1).trim();
          if (key) map[key] = val;
        }
      });
    return map;
  }

  function textToList(value: string) {
    return value
      .split(/\r?\n|,/)
      .map((entry) => entry.trim())
      .filter(Boolean);
  }

  function buildServer(): McpServerConfig {
    const args = form.transport === "stdio" ? textToList(form.args) : [];
    const env = form.transport === "stdio" ? textToMap(form.env) : {};
    const headers = form.transport === "http_sse" ? textToMap(form.headers) : {};
    const toolAllow = textToList(form.tool_allowlist);
    const resourceAllow = textToList(form.resource_allowlist);
    return {
      id: form.id.trim(),
      name: form.name.trim(),
      enabled: form.enabled,
      transport: form.transport as "stdio" | "http_sse",
      command: form.transport === "stdio" ? form.command.trim() || null : null,
      args: form.transport === "stdio" ? (args.length ? args : null) : null,
      cwd: form.transport === "stdio" ? form.cwd.trim() || null : null,
      env:
        form.transport === "stdio"
          ? Object.keys(env).length
            ? env
            : null
          : null,
      url: form.transport === "http_sse" ? form.url.trim() || null : null,
      headers:
        form.transport === "http_sse"
          ? Object.keys(headers).length
            ? headers
            : null
          : null,
      tool_allowlist: toolAllow.length ? toolAllow : null,
      resource_allowlist: resourceAllow.length ? resourceAllow : null,
    };
  }

  function validateForm() {
    if (!form.id.trim() || !form.name.trim()) {
      return "ID and Name are required.";
    }
    if (form.transport === "stdio" && !form.command.trim()) {
      return "Command is required for stdio transport.";
    }
    if (form.transport === "http_sse" && !form.url.trim()) {
      return "URL is required for HTTP/SSE transport.";
    }
    return null;
  }

  function showMessage(type: string, text: string) {
    message = { type, text };
    setTimeout(() => {
      message = { type: "", text: "" };
    }, 5000);
  }

  async function handleSave() {
    const error = validateForm();
    if (error) {
      showMessage("error", error);
      return;
    }
    saving = true;
    try {
      const server = buildServer();
      if (selectedId) {
        await mcpStore.updateServer(server);
        showMessage("success", "Server updated.");
      } else {
        await mcpStore.addServer(server);
        showMessage("success", "Server added.");
        selectedId = server.id;
      }
      await mcpStore.refreshStatus(selectedId ?? undefined);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showMessage("error", msg);
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    if (!selectedId) return;
    if (!confirm("Remove this MCP server?")) return;
    try {
      await mcpStore.removeServer(selectedId);
      showMessage("success", "Server removed.");
      resetForm();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showMessage("error", msg);
    }
  }

  async function handleConnect() {
    if (!selectedId) return;
    const status = mcpStore.statusMap[selectedId];
    try {
      if (status?.connected) {
        await mcpStore.disconnect(selectedId);
        showMessage("success", "Disconnected.");
      } else {
        await mcpStore.connect(selectedId);
        showMessage("success", "Connected.");
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showMessage("error", msg);
    }
  }

  async function refreshTools() {
    if (!selectedId) return;
    try {
      await mcpStore.listTools(selectedId);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showMessage("error", msg);
    }
  }

  async function refreshResources() {
    if (!selectedId) return;
    try {
      await mcpStore.listResources(selectedId);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showMessage("error", msg);
    }
  }

  function prettyJson(value: ToolDefinition | ResourceDefinition) {
    return JSON.stringify(value, null, 2);
  }

  async function handleOpenConfig() {
    if (!mcpStore.configPath) return;
    try {
      await openPath(mcpStore.configPath);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showMessage("error", msg);
    }
  }
</script>

<div class="mx-auto h-full max-w-[1200px] overflow-y-auto px-6 py-10 text-foreground">
  <div class="mb-8 flex items-start justify-between border-b border-border/60 pb-6">
    <div>
      <h1 class="text-3xl font-bold tracking-tight leading-none">MCP Servers</h1>
      <p class="mt-1 text-sm text-muted-foreground leading-normal">
        Manage local and remote Model Context Protocol servers.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button
        class="inline-flex items-center gap-2 rounded-lg border border-border bg-transparent px-4 py-2 text-sm font-medium transition-colors hover:bg-muted disabled:opacity-50"
        onclick={handleOpenConfig}
        disabled={!mcpStore.configPath}
      >
        <FileCode size={16} />
        Editar mcp.json
      </button>
      <button
        class="inline-flex items-center gap-2 rounded-lg border border-border bg-transparent px-4 py-2 text-sm font-medium transition-colors hover:bg-muted"
        onclick={resetForm}
      >
        <Plus size={16} />
        New Server
      </button>
    </div>
  </div>

  {#if message.text}
    <div
      class={`mb-5 rounded-lg border px-4 py-3 text-sm ${
        message.type === "success"
          ? "border-green-500/30 bg-green-500/10 text-green-400"
          : "border-red-500/30 bg-red-500/10 text-red-400"
      }`}
    >
      {message.text}
    </div>
  {/if}

  {#if mcpStore.configPath}
    <div class="mb-6 text-xs text-muted-foreground">
      Config path: <span class="font-mono">{mcpStore.configPath}</span>
    </div>
  {/if}

  <div class="grid gap-6 lg:grid-cols-[320px_1fr]">
    <section class="rounded-xl border border-border/60 bg-card p-5 shadow-sm">
      <div class="mb-4 flex items-center gap-3 border-b border-border/40 pb-3">
        <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500">
          <Server size={18} />
        </div>
        <div>
          <h2 class="text-base font-semibold leading-tight">Servers</h2>
          <p class="text-xs text-muted-foreground">Select a server to edit</p>
        </div>
      </div>

      {#if mcpStore.loading}
        <div class="py-4 text-sm text-muted-foreground">Loading...</div>
      {:else if mcpStore.servers.length === 0}
        <div class="py-4 text-sm text-muted-foreground">No MCP servers configured.</div>
      {:else}
        <div class="flex flex-col gap-2">
          {#each mcpStore.servers as server}
            {@const status = mcpStore.statusMap[server.id]}
            <button
              class={`flex w-full items-center justify-between rounded-lg border px-3 py-2 text-left text-sm transition-colors ${
                selectedId === server.id
                  ? "border-primary/60 bg-primary/10 text-foreground"
                  : "border-border/60 hover:bg-muted"
              }`}
              onclick={() => selectServer(server)}
            >
              <div class="flex flex-col gap-1">
                <div class="flex items-center gap-2">
                  <span
                    class={`h-2 w-2 rounded-full ${
                      status?.connected ? "bg-emerald-500" : "bg-muted-foreground/40"
                    }`}
                  ></span>
                  <span class="font-medium">{server.name}</span>
                </div>
                <span class="text-xs text-muted-foreground font-mono">{server.id}</span>
              </div>
              <div class="text-xs text-muted-foreground">
                {server.transport === "stdio" ? "Local" : "Remote"}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </section>

    <section class="rounded-xl border border-border/60 bg-card p-6 shadow-sm">
      <div class="mb-6 flex items-center justify-between border-b border-border/40 pb-4">
        <div class="flex items-center gap-3">
          <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-amber-500/10 text-amber-500">
            <Plug size={18} />
          </div>
          <div>
            <h2 class="text-lg font-semibold leading-tight">
              {selectedId ? "Edit MCP Server" : "Add MCP Server"}
            </h2>
            <p class="text-xs text-muted-foreground">Configure transport and access</p>
          </div>
        </div>
        {#if selectedId}
          <div class="flex items-center gap-2">
            <button
              class="inline-flex items-center gap-2 rounded-lg border border-border bg-transparent px-3 py-2 text-xs font-medium transition-colors hover:bg-muted"
              onclick={handleConnect}
            >
              {#if mcpStore.statusMap[selectedId]?.connected}
                <Unlink size={14} />
                Disconnect
              {:else}
                <Link size={14} />
                Connect
              {/if}
            </button>
            <button
              class="inline-flex items-center gap-2 rounded-lg border border-border bg-transparent px-3 py-2 text-xs font-medium transition-colors hover:bg-muted"
              onclick={handleDelete}
            >
              <Trash2 size={14} />
              Delete
            </button>
          </div>
        {/if}
      </div>

      <div class="grid gap-4 md:grid-cols-2">
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-muted-foreground">ID</label>
          <input
            class="w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
            bind:value={form.id}
            placeholder="unique-id"
            disabled={selectedId !== null}
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-xs font-medium text-muted-foreground">Name</label>
          <input
            class="w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
            bind:value={form.name}
            placeholder="My MCP Server"
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-xs font-medium text-muted-foreground">Transport</label>
          <select
            class="w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
            bind:value={form.transport}
          >
            <option value="stdio">Local (stdio)</option>
            <option value="http_sse">Remote (HTTP/SSE)</option>
          </select>
        </div>

        <div class="flex items-center gap-2">
          <input type="checkbox" bind:checked={form.enabled} id="enabled" />
          <label for="enabled" class="text-xs font-medium text-muted-foreground">
            Enabled
          </label>
        </div>
      </div>

      {#if form.transport === "stdio"}
        <div class="mt-6 grid gap-4 md:grid-cols-2">
          <div class="space-y-1.5 md:col-span-2">
            <label class="text-xs font-medium text-muted-foreground">Command</label>
            <input
              class="w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
              bind:value={form.command}
              placeholder="path/to/server-binary"
            />
          </div>
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-muted-foreground">Args (one per line)</label>
            <textarea
              class="h-24 w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-xs font-mono outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
              bind:value={form.args}
              placeholder="--flag\n--port=8080"
            />
          </div>
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-muted-foreground">Environment (key=value)</label>
            <textarea
              class="h-24 w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-xs font-mono outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
              bind:value={form.env}
              placeholder="API_KEY=secret"
            />
          </div>
          <div class="space-y-1.5 md:col-span-2">
            <label class="text-xs font-medium text-muted-foreground">Working Directory</label>
            <input
              class="w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
              bind:value={form.cwd}
              placeholder="/path/to/cwd"
            />
          </div>
        </div>
      {:else}
        <div class="mt-6 grid gap-4 md:grid-cols-2">
          <div class="space-y-1.5 md:col-span-2">
            <label class="text-xs font-medium text-muted-foreground">Server URL</label>
            <input
              class="w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
              bind:value={form.url}
              placeholder="https://mcp.example.com"
            />
          </div>
          <div class="space-y-1.5 md:col-span-2">
            <label class="text-xs font-medium text-muted-foreground">Headers (key=value)</label>
            <textarea
              class="h-24 w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-xs font-mono outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
              bind:value={form.headers}
              placeholder="Authorization=Bearer ..."
            />
          </div>
        </div>
      {/if}

      <div class="mt-6 grid gap-4 md:grid-cols-2">
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-muted-foreground flex items-center gap-2">
            <KeyRound size={14} />
            Tool Allowlist (one per line)
          </label>
          <textarea
            class="h-24 w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-xs font-mono outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
            bind:value={form.tool_allowlist}
            placeholder="tool.one\ntool.two"
          />
        </div>
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-muted-foreground flex items-center gap-2">
            <Cable size={14} />
            Resource Allowlist (one per line)
          </label>
          <textarea
            class="h-24 w-full rounded-md border border-border bg-muted/50 px-3 py-2 text-xs font-mono outline-none focus:border-primary focus:ring-1 focus:ring-primary/20"
            bind:value={form.resource_allowlist}
            placeholder="file:///path\nmcp://resource"
          />
        </div>
      </div>

      <div class="mt-6 flex items-center gap-2">
        <button
          class="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:opacity-50"
          onclick={handleSave}
          disabled={saving}
        >
          {#if saving}
            <div class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
          {:else}
            <Save size={16} />
          {/if}
          Save
        </button>
      </div>

      {#if selectedId}
        <div class="mt-8 grid gap-6 lg:grid-cols-2">
          <div class="rounded-lg border border-border/60 bg-background p-4">
            <div class="mb-3 flex items-center justify-between">
              <div class="flex items-center gap-2 text-sm font-semibold">
                <List size={14} />
                Tools
              </div>
              <button
                class="inline-flex items-center gap-2 rounded-md border border-border bg-transparent px-3 py-1.5 text-xs font-medium transition-colors hover:bg-muted"
                onclick={refreshTools}
              >
                <RefreshCw size={12} />
                Refresh
              </button>
            </div>
            {#if mcpStore.toolsMap[selectedId]?.length}
              <div class="flex flex-col gap-3">
                {#each mcpStore.toolsMap[selectedId] as tool}
                  <div class="rounded-md border border-border/60 bg-muted/40 p-3">
                    <div class="text-sm font-semibold">
                      {tool.name ?? "Unnamed tool"}
                    </div>
                    {#if tool.description}
                      <div class="text-xs text-muted-foreground mt-1">
                        {tool.description}
                      </div>
                    {/if}
                    <pre class="mt-3 rounded bg-background/60 p-2 text-[11px] text-muted-foreground overflow-x-auto">{prettyJson(tool)}</pre>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-xs text-muted-foreground">No tools loaded.</div>
            {/if}
          </div>

          <div class="rounded-lg border border-border/60 bg-background p-4">
            <div class="mb-3 flex items-center justify-between">
              <div class="flex items-center gap-2 text-sm font-semibold">
                <List size={14} />
                Resources
              </div>
              <button
                class="inline-flex items-center gap-2 rounded-md border border-border bg-transparent px-3 py-1.5 text-xs font-medium transition-colors hover:bg-muted"
                onclick={refreshResources}
              >
                <RefreshCw size={12} />
                Refresh
              </button>
            </div>
            {#if mcpStore.resourcesMap[selectedId]?.length}
              <div class="flex flex-col gap-3">
                {#each mcpStore.resourcesMap[selectedId] as resource}
                  <div class="rounded-md border border-border/60 bg-muted/40 p-3">
                    <div class="text-sm font-semibold">
                      {resource.name ?? resource.uri ?? "Unnamed resource"}
                    </div>
                    {#if resource.description}
                      <div class="text-xs text-muted-foreground mt-1">
                        {resource.description}
                      </div>
                    {/if}
                    <pre class="mt-3 rounded bg-background/60 p-2 text-[11px] text-muted-foreground overflow-x-auto">{prettyJson(resource)}</pre>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-xs text-muted-foreground">No resources loaded.</div>
            {/if}
          </div>
        </div>
      {/if}
    </section>
  </div>
</div>
