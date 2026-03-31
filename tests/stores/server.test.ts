import { describe, it, expect, vi, beforeEach } from 'vitest';
import { serverStore } from '$lib/stores/server.svelte';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

describe('serverStore', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('initializes with default state', () => {
        expect(serverStore.isRunning).toBe(false);
        expect(serverStore.isLoading).toBe(false);
    });

    it('sets running state', () => {
        serverStore.isRunning = true;
        expect(serverStore.isRunning).toBe(true);
    });

    it('sets loading state', () => {
        serverStore.isLoading = true;
        expect(serverStore.isLoading).toBe(true);
    });

    it('sets error message', () => {
        serverStore.error = 'Test error';
        expect(serverStore.error).toBe('Test error');
    });

    it('clears error', () => {
        serverStore.error = 'Test error';
        serverStore.clearError();
        expect(serverStore.error).toBeNull();
    });
});
