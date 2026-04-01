import { invoke } from "@tauri-apps/api/core";

/**
 * Wrapper for Tauri's invoke function with error logging.
 * @param {string} command - The command name.
 * @param {Record<string, unknown>} [args={}] - The arguments.
 * @returns {Promise<unknown>}
 */
export async function invokeCommand(command: string, args: Record<string, unknown> = {}): Promise<unknown> {
  try {
    if (typeof window !== "undefined") {
      const hasTauri =
        typeof (window as any).__TAURI_INTERNALS__ !== "undefined" ||
        typeof (window as any).__TAURI__ !== "undefined";
      if (!hasTauri) {
        throw new Error(
          "Tauri IPC unavailable. This view is not running inside the Tauri shell."
        );
      }
    }
    return await invoke(command, args);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (message.includes("ipc.localhost") || message.includes("ERR_CONNECTION_REFUSED")) {
      throw new Error(
        "Tauri IPC connection refused. The Rust backend may have crashed or the app is running in a browser instead of Tauri."
      );
    }
    throw error;
  }
}
