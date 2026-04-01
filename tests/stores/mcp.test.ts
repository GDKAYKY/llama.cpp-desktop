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
  });

  it("initializes with empty servers", () => {
    expect(mcpStore.servers).toEqual([]);
  });

  it("sets servers list", () => {
    const servers: McpServerConfig[] = [
      { id: "server1", name: "Test Server 1", enabled: true, transport: "stdio", command: "" },
      { id: "server2", name: "Test Server 2", enabled: false, transport: "stdio", command: "" },
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
});
