import type { AppConfig } from "./AppConfig";

/**
 * Default configuration values
 */
export const DEFAULT_CONFIG: AppConfig = {
  modelsDirectory: null,
  llamaDirectory: null,
  theme: "dark",
  language: "en",
  maxTokens: 2048,
  contextSize: 4096,
  temperature: 0.7,
  autoSaveChat: true,
  chatHistoryLimit: 50,
  serverPort: 8080,
  webSearchProvider: "tavily",
  webSearchMcpId: null,
};
