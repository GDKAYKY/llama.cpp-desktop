import { invokeCommand } from './ipc.js';

/**
 * @typedef {Object} AppConfig
 * @property {string | null} models_directory
 * @property {string} theme
 * @property {string} language
 * @property {number} max_tokens
 * @property {number} temperature
 * @property {boolean} auto_save_chat
 * @property {number} chat_history_limit
 */

/**
 * Load application configuration
 * @returns {Promise<AppConfig>} Configuration object
 */
export async function loadConfig() {
    return await invokeCommand('load_config');
}

/**
 * Save application configuration
 * @param {AppConfig} config - Configuration object
 * @returns {Promise<void>}
 */
export async function saveConfig(config) {
    return await invokeCommand('save_config', { config });
}

/**
 * Reset configuration to defaults
 * @returns {Promise<AppConfig>} Default configuration object
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
 * @type {AppConfig}
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
