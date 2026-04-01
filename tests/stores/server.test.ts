import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";
import { serverStore } from "$lib/stores/server.svelte";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("serverStore", () => {
  beforeAll(() => {
    vi.stubGlobal("window", { __TAURI_INTERNALS__: true });
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("initializes with default state", () => {
    expect(serverStore.isRunning).toBe(false);
  });

  it("sets running state", () => {
    serverStore.isRunning = true;
    expect(serverStore.isRunning).toBe(true);
  });

  it("sets starting state", () => {
    serverStore.isStarting = true;
    expect(serverStore.isStarting).toBe(true);
  });

  it("sets error message", () => {
    serverStore.error = "Test error";
    expect(serverStore.error).toBe("Test error");
  });

  it("clears error", () => {
    serverStore.error = "Test error";
    serverStore.error = null;
    expect(serverStore.error).toBeNull();
  });
});
