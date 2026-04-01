import { describe, it, expect, vi, beforeEach } from "vitest";
import { uiStore } from "$lib/stores/ui.svelte";

describe("uiStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("initializes with default state", () => {
    expect(uiStore.isSidebarOpen).toBeDefined();
  });

  it("toggles sidebar", () => {
    const initial = uiStore.isSidebarOpen;
    uiStore.toggleSidebar();
    expect(uiStore.isSidebarOpen).toBe(!initial);
  });
});
