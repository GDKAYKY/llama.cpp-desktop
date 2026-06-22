export interface AppConfig {
  modelsDirectory: string | null;
  llamaDirectory: string | null;
  theme: string;
  language: string;
  maxTokens: number;
  contextSize: number;
  temperature: number;
  autoSaveChat: boolean;
  chatHistoryLimit: number;
  serverPort: number;
  Parallel: number;
  GpuLayers: number;
  GpuDevice: string;
  Jinja: boolean;
  llamaServerExtraArgs: string[];
  webSearchProvider: "tavily" | "custom";
  webSearchMcpId: string | null;
  chatHeaderStyle: "default" | "capsule";
}
//! USE PascalCase

/**
 * Default configuration values
 */

export const DEFAULT_CONFIG: AppConfig = {
  modelsDirectory: null,
  llamaDirectory: null,
  theme: "dark",
  language: "en",
  maxTokens: 2048,
  contextSize: 8192,
  temperature: 0.7,
  autoSaveChat: true,
  chatHistoryLimit: 50,
  serverPort: 8080,
  Parallel: 1,
  GpuLayers: 33,
  GpuDevice: "0",
  Jinja: true,
  llamaServerExtraArgs: [],
  webSearchProvider: "tavily",
  webSearchMcpId: null,
  chatHeaderStyle: "default",
};
