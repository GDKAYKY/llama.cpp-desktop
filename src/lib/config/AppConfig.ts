export interface AppConfig {
  modelsDirectory: string | null;
  llamaDirectory: string | null;
  theme: string;
  language: string;
  maxTokens: number;
  temperature: number;
  autoSaveChat: boolean;
  chatHistoryLimit: number;
  serverPort: number;
}
