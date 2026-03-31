import { describe, it, expect, vi, beforeEach } from 'vitest';
import { uiStore } from '$lib/stores/ui.svelte';

describe('uiStore', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('initializes with default state', () => {
        expect(uiStore.sidebarOpen).toBeDefined();
        expect(uiStore.currentPage).toBeDefined();
    });

    it('toggles sidebar', () => {
        const initial = uiStore.sidebarOpen;
        uiStore.toggleSidebar();
        expect(uiStore.sidebarOpen).toBe(!initial);
    });

    it('sets current page', () => {
        uiStore.setPage('settings');
        expect(uiStore.currentPage).toBe('settings');
    });
});
