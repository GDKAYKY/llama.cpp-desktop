import { invoke } from "@tauri-apps/api/core";

/**
 * Wrapper for Tauri's invoke function with error logging.
 * @param {string} command - The command name.
 * @param {Record<string, unknown>} [args={}] - The arguments.
 * @returns {Promise<unknown>}
 */
export async function invokeCommand(command: string, args: Record<string, unknown> = {}): Promise<unknown> {
  try {
    return await invoke(command, args);
  } catch (error) {
    console.error(`IPC Error [${command}]:`, error);
    throw error;
  }
}
