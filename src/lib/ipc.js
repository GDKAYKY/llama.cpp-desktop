import { invoke } from "@tauri-apps/api/core";

/**
 * Wrapper for Tauri's invoke function with error logging.
 * @param {string} command - The command name.
 * @param {Object} [args={}] - The arguments.
 * @returns {Promise<any>}
 */
export async function invokeCommand(command, args = {}) {
  try {
    return await invoke(command, args);
  } catch (error) {
    console.error(`IPC Error [${command}]:`, error);
    throw error;
  }
}
