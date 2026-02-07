import { describe, it, expect } from 'vitest';
import { uiStore } from '../../src/lib/stores/ui.svelte';

describe('ui store', () => {
  it('toggles sidebar state', () => {
    const initial = uiStore.isSidebarOpen;
    uiStore.toggleSidebar();
    expect(uiStore.isSidebarOpen).toBe(!initial);
  });
});
