import { invokeCommand } from './ipc.js';

/**
 * Load application configuration
 * @returns {Promise<Object>} Configuration object
 */
export async function loadConfig() {
    return await invokeCommand('load_config');
}

/**
 * Save application configuration
 * @param {Object} config - Configuration object
 * @returns {Promise<void>}
 */
export async function saveConfig(config) {
    return await invokeCommand('save_config', { config });
}

/**
 * Reset configuration to defaults
 * @returns {Promise<Object>} Default configuration object
 */
export async function resetConfig() {
    return await invokeCommand('reset_config');
}

/**
 * Get the path to the configuration file
 * @returns {Promise<string>} Path to config file
 */
export async function getConfigPath() {
    return await invokeCommand('get_config_path_string');
}

/**
 * Default configuration values
 */
export const DEFAULT_CONFIG = {
    models_directory: null,
    theme: 'dark',
    language: 'en',
    max_tokens: 2048,
    temperature: 0.7,
    auto_save_chat: true,
    chat_history_limit: 50
};
