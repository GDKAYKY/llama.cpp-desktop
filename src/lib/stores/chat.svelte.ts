import { invokeCommand } from '$lib/ipc';
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

  sessionId = crypto.randomUUID();
  unlisten: UnlistenFn | null = null;

  async initialize() {
    this.error = null;
    
    // Persistence: only create session if doesn't exist
    if (!this.sessionId) {
      this.sessionId = crypto.randomUUID();
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

  async clear() {
    this.messages = [];
    this.error = null;
    this.sessionId = crypto.randomUUID();
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
