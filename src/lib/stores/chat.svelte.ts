import { initLlama, shutdownLlama, sendMessage } from '$lib/llama';
import { settingsStore } from './settings.svelte';
import { modelsStore } from './models.svelte';

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

class ChatStore {
  messages = $state<Message[]>([]);
  isLoading = $state(false);
  isModelLoading = $state(false);
  modelLoaded = $state(false);
  error = $state<string | null>(null);

  async loadModel() {
    if (!modelsStore.selectedModel) {
      this.error = 'No model selected';
      return;
    }

    const llamaPath = settingsStore.settings.llamaPath;
    if (!llamaPath) {
      this.error = 'Llama.cpp path not configured';
      return;
    }

    try {
      this.isModelLoading = true;
      this.error = null;
      await initLlama(llamaPath, modelsStore.selectedModel.model_file_path || '');
      this.modelLoaded = true;
    } catch (err) {
      console.error('Failed to load model:', err);
      this.error = 'Failed to load model';
      this.modelLoaded = false;
    } finally {
      this.isModelLoading = false;
    }
  }

  async unloadModel() {
    try {
      await shutdownLlama();
      this.modelLoaded = false;
    } catch (err) {
      console.error('Failed to unload model:', err);
    }
  }

  async send(content: string) {
    if (!this.modelLoaded && !this.isModelLoading) {
      await this.loadModel();
    }

    if (!this.modelLoaded) return;

    const userMessage: Message = {
      role: 'user',
      content,
      timestamp: Date.now()
    };

    this.messages.push(userMessage);
    this.isLoading = true;
    this.error = null;

    try {
      const response = await sendMessage(content);
      const assistantMessage: Message = {
        role: 'assistant',
        content: response,
        timestamp: Date.now()
      };
      this.messages.push(assistantMessage);
    } catch (err) {
      console.error('Failed to send message:', err);
      this.error = 'Failed to get response from model';
    } finally {
      this.isLoading = false;
    }
  }

  clear() {
    this.messages = [];
  }
}

export const chatStore = new ChatStore();
