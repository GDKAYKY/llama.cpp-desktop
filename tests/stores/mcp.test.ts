import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mcpStore } from '$lib/stores/mcp.svelte';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

describe('mcpStore', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('initializes with empty servers', () => {
        expect(mcpStore.servers).toEqual([]);
    });

    it('sets servers list', () => {
        const servers = [
            { id: 'server1', name: 'Test Server 1', enabled: true },
            { id: 'server2', name: 'Test Server 2', enabled: false },
        ];
        
        mcpStore.servers = servers;
        expect(mcpStore.servers.length).toBe(2);
    });

    it('sets loading state', () => {
        mcpStore.isLoading = true;
        expect(mcpStore.isLoading).toBe(true);
    });

    it('sets error message', () => {
        mcpStore.error = 'Connection failed';
        expect(mcpStore.error).toBe('Connection failed');
    });

    it('clears error', () => {
        mcpStore.error = 'Test error';
        mcpStore.clearError();
        expect(mcpStore.error).toBeNull();
    });
});
