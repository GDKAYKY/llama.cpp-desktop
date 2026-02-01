import { invokeCommand } from "../infrastructure/ipc";
import type { AppConfig } from "./AppConfig";
import { DEFAULT_CONFIG } from "./defaultConfig";

export type { AppConfig };


/**
 * Load application configuration
 * @returns {Promise<AppConfig>} Configuration object
 */
export async function loadConfig(): Promise<AppConfig> {
  return await invokeCommand("load_config") as AppConfig;
}

/**
 * Save application configuration
 * @param {AppConfig} config - Configuration object
 * @returns {Promise<void>}
 */
export async function saveConfig(config: AppConfig): Promise<void> {
  await invokeCommand("save_config", { config });
}

/**
 * Reset configuration to defaults
 * @returns {Promise<AppConfig>} Default configuration object
 */
export async function resetConfig(): Promise<AppConfig> {
  return await invokeCommand("reset_config") as AppConfig;
}

/**
 * Get the path to the configuration file
 * @returns {Promise<string>} Path to config file
 */
export async function getConfigPath(): Promise<string> {
  return await invokeCommand("get_config_path_string") as string;
}

export { DEFAULT_CONFIG };
