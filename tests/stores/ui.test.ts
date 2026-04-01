import { describe, it, expect, vi, beforeEach } from "vitest";
import { uiStore } from "$lib/stores/ui.svelte";

describe("uiStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    uiStore.isSidebarOpen = true;
  });

  it("initializes with sidebar open", () => {
    expect(uiStore.isSidebarOpen).toBe(true);
  });

  it("toggles sidebar from open to closed", () => {
    uiStore.isSidebarOpen = true;
    uiStore.toggleSidebar();
    expect(uiStore.isSidebarOpen).toBe(false);
  });

  it("toggles sidebar from closed to open", () => {
    uiStore.isSidebarOpen = false;
    uiStore.toggleSidebar();
    expect(uiStore.isSidebarOpen).toBe(true);
  });

  it("double toggle returns to original state", () => {
    const initial = uiStore.isSidebarOpen;
    uiStore.toggleSidebar();
    uiStore.toggleSidebar();
    expect(uiStore.isSidebarOpen).toBe(initial);
  });

  it("allows direct state assignment", () => {
    uiStore.isSidebarOpen = false;
    expect(uiStore.isSidebarOpen).toBe(false);

    uiStore.isSidebarOpen = true;
    expect(uiStore.isSidebarOpen).toBe(true);
  });
});
