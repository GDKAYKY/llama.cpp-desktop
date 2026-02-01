import { invokeCommand } from '$infrastructure/ipc';
import { Channel } from '@tauri-apps/api/core';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { settingsStore } from '$lib/stores/settings.svelte';

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
}

class ChatStore {
  messages = $state<Message[]>([]);
  isLoading = $state(false);
  error = $state<string | null>(null);
  modelLoaded = $state(true);

  sessionId = $state<string>('');
  unlisten: UnlistenFn | null = null;

  async initialize() {
    this.error = null;
    
    // Load from localStorage or create new
    const savedSession = localStorage.getItem('llama_chat_session_id');
    if (savedSession) {
      this.sessionId = savedSession;
    } else {
      this.sessionId = crypto.randomUUID();
      localStorage.setItem('llama_chat_session_id', this.sessionId);
    }
  }

  appendChunk(chunk: string) {
    if (this.messages.length === 0) return;
    const lastMsg = this.messages[this.messages.length - 1];
    if (lastMsg.role === 'assistant') {
      lastMsg.content += chunk;
    } else {
      this.messages.push({
        role: 'assistant',
        content: chunk,
        timestamp: Date.now()
      });
    }
  }

  async send(content: string) {
    // Ensure sessionId exists and is stored
    if (!this.sessionId) {
      this.sessionId = crypto.randomUUID();
    }
    localStorage.setItem('llama_chat_session_id', this.sessionId);

    const userMessage: Message = {
      role: 'user',
      content,
      timestamp: Date.now()
    };

    this.messages.push(userMessage);
    
    // Add placeholder for assistant
    this.messages.push({
        role: 'assistant',
        content: '',
        timestamp: Date.now()
    });

    this.isLoading = true;
    this.error = null;

    const onEvent = new Channel<any>();
    onEvent.onmessage = (payload) => {
      if (payload.chunk) {
        this.appendChunk(payload.chunk);
      }
      if (payload.status === 'done') {
        console.log('Stream finished');
      }
    };

    try {
      await invokeCommand('send_message', {
        message: content,
        sessionId: this.sessionId,
        temperature: settingsStore.settings.temperature,
        maxTokens: settingsStore.settings.maxTokens,
        onEvent
      });
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
      console.error("ERRO NO CHAT:", err);
    } finally {
      this.isLoading = false;
    }
  }

  async editMessage(index: number, content: string) {
    if (this.isLoading) return;
    
    // Remove all messages from this index onwards
    this.messages = this.messages.slice(0, index);
    
    // Send the new version
    await this.send(content);
  }

  async clear() {
    try {
      await invokeCommand('clear_chat', { sessionId: this.sessionId });
    } catch (err) {
      console.warn("Failed to clear backend chat session:", err);
    }
    this.messages = [];
    this.error = null;
    this.sessionId = crypto.randomUUID();
    localStorage.setItem('llama_chat_session_id', this.sessionId);
  }

  async destroy() {
    if (this.unlisten) {
      this.unlisten();
      this.unlisten = null;
    }
    this.messages = [];
    this.error = null;
  }
}

export const chatStore = new ChatStore();
