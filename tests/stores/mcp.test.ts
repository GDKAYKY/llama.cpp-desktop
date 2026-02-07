import { describe, it, expect, vi, beforeEach } from 'vitest';

describe('mcp store', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
  });

  it('loads config and status', async () => {
    const invokeCommand = vi.fn()
      .mockResolvedValueOnce({ servers: [{ id: 'one' }] }) // load_mcp_config
      .mockResolvedValueOnce([{ id: 'one', connected: false }]) // mcp_status
      .mockResolvedValueOnce('/path/mcp.json'); // get_mcp_config_path_string

    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { mcpStore } = await import('../../src/lib/stores/mcp.svelte');

    await mcpStore.init();
    expect(mcpStore.servers.length).toBe(1);
    expect(mcpStore.statusMap.one.connected).toBe(false);
    expect(mcpStore.configPath).toBe('/path/mcp.json');
  });

  it('handles add/update/remove/connect/disconnect', async () => {
    const invokeCommand = vi.fn().mockResolvedValue(undefined);
    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { mcpStore } = await import('../../src/lib/stores/mcp.svelte');

    await mcpStore.addServer({ id: 'one' } as any);
    await mcpStore.updateServer({ id: 'one' } as any);
    await mcpStore.removeServer('one');
    await mcpStore.connect('one');
    await mcpStore.disconnect('one');

    expect(invokeCommand).toHaveBeenCalled();
  });

  it('lists tools and resources', async () => {
    const invokeCommand = vi.fn()
      .mockResolvedValueOnce([{ name: 'tool' }])
      .mockResolvedValueOnce([{ uri: 'res' }]);
    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { mcpStore } = await import('../../src/lib/stores/mcp.svelte');

    const tools = await mcpStore.listTools('one');
    const resources = await mcpStore.listResources('one');
    expect(tools).toEqual([{ name: 'tool' }]);
    expect(resources).toEqual([{ uri: 'res' }]);
  });
});
