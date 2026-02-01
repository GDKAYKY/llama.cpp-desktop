import { invokeCommand } from '$infrastructure/ipc';
import { Channel } from '@tauri-apps/api/core';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { settingsStore } from '$lib/stores/settings.svelte';
import { 
  db, 
  saveMessage, 
  createConversation, 
  getConversationHistory, 
  findRelevantContext,
  getRecentConversations,
  updateConversationTitle,
  type Conversation
} from '$lib/services/history';

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

  // DB IDs
  activeConversationId = $state<number | null>(null);
  history = $state<Conversation[]>([]); 

  sessionId = $state<string>('');

  unlisten: UnlistenFn | null = null;
  
  // To accumulate assistant response before saving
  currentAssistantResponse = '';

  async initialize() {
    this.error = null;
    
    // 1. Create a session ID for the backend (ChatOrchestrator) if not exists
    const savedSession = localStorage.getItem('llama_chat_session_id');
    if (savedSession) {
      this.sessionId = savedSession;
    } else {
      this.sessionId = crypto.randomUUID();
      localStorage.setItem('llama_chat_session_id', this.sessionId);
    }

    // 2. Load last conversation from DB or create new
    try {
        await this.loadRecentConversations();
        
        // If we have history, load the most recent one (first after reverse sort)
        if (this.history.length > 0) {
             // Instead of just taking the last ID, let's take the first from our sorted list
             const lastActive = this.history[0];
             if (lastActive && lastActive.id) {
                 this.activeConversationId = lastActive.id;
                 await this.loadConversation(lastActive.id);
             }
        } else {
            // No conversations yet, start fresh
            this.activeConversationId = await createConversation("New Chat");
            await this.loadRecentConversations();
        }
    } catch (err) {
        console.error("Failed to load history:", err);
    }
  }

  async loadRecentConversations() {
      this.history = await getRecentConversations();
  }

  async loadConversation(id: number) {
      const history = await getConversationHistory(id);
      this.messages = history.map(h => ({
          role: h.role,
          content: h.content,
          timestamp: h.timestamp
      }));
      this.activeConversationId = id;
      
      // Update session ID if needed? 
      // Actually backend session ID is just for the current "run", 
      // but conceptually we might want to reset it on fresh load to avoid state pollution in backend orchestrator.
      this.sessionId = crypto.randomUUID(); 
      localStorage.setItem('llama_chat_session_id', this.sessionId);
  }

  appendChunk(chunk: string) {
    if (this.messages.length === 0) return;
    const lastMsg = this.messages[this.messages.length - 1];
    if (lastMsg.role === 'assistant') {
      lastMsg.content += chunk;
      this.currentAssistantResponse += chunk;
    } else {
      // First chunk of assistant response
      this.messages.push({
        role: 'assistant',
        content: chunk,
        timestamp: Date.now()
      });
      this.currentAssistantResponse = chunk;
    }
  }

  async send(content: string) {
    if (!this.activeConversationId) {
        this.activeConversationId = await createConversation(content.slice(0, 30));
    }
    
    // Ensure sessionId exists
    if (!this.sessionId) {
      this.sessionId = crypto.randomUUID();
      localStorage.setItem('llama_chat_session_id', this.sessionId);
    }

    const userMessage: Message = {
      role: 'user',
      content,
      timestamp: Date.now()
    };
    
    // 1. Update UI immediately
    this.messages.push(userMessage);
    
    // 2. Save User Message to DB
    await saveMessage(this.activeConversationId, 'user', content);
    
    // 3. Find Context (Memory)
    let contextPrompt = "";
    try {
        const context = await findRelevantContext(content, this.activeConversationId);
        if (context) {
            console.log("Injecting Context:", context);
            contextPrompt = `\n\n[System Note: The following is relevant context from the user's past conversations. Use it to answer if applicable.]\n${context}\n\n`;
        }
    } catch (e) {
        console.warn("Context retrieval failed:", e);
    }

    // 4. Prepare backend payload (Inject context)
    // We append the context to the message essentially hiding it from the UI but sending it to the LLM.
    // Ideally, we'd use a System message, but send_message might treat everything as user text.
    // Let's prepend it clearly.
    const finalPayload = contextPrompt ? `${contextPrompt}User: ${content}` : content;

    // Add placeholder for assistant
    this.messages.push({
        role: 'assistant',
        content: '',
        timestamp: Date.now()
    });

    this.isLoading = true;
    this.error = null;
    this.currentAssistantResponse = '';

    const onEvent = new Channel<any>();
    onEvent.onmessage = async (payload) => {
      if (payload.chunk) {
        this.appendChunk(payload.chunk);
      }
      if (payload.status === 'done') {
        console.log('Stream finished');
        // 5. Save Assistant Message to DB
        if (this.activeConversationId && this.currentAssistantResponse) {
            await saveMessage(this.activeConversationId, 'assistant', this.currentAssistantResponse);
            
            // 6. Auto-generate Title if this is the first exchange (2 messages: User + Assistant)
            // And title is "New Chat" (or roughly check if we haven't generated one yet)
            // We verify by checking if the active conversation title is "New Chat" (we might need to fetch it or check store state if we tracked it closer)
            // For simplicity: If messages.length === 2.
            if (this.messages.length === 2 && this.activeConversationId) {
                 // Run in background
                 this.generateTitle(this.activeConversationId, this.messages[0].content, this.currentAssistantResponse);
            }
            
            await this.loadRecentConversations(); // Refresh sidebar order
        }
      }
    };

    try {
      await invokeCommand('send_message', {
        message: finalPayload, // Send context + content
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
    // Basic implementation: Just truncate and resend.
    // DB implication: We effectively branch or just ignore the old tail.
    // For now: Truncate store, resend. DB will just append new messages.
    // Ideally, we should delete from DB, but "History" implies potentially keeping everything.
    // Let's just keep appending for now to avoid data loss.
    
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
    
    // Create NEW conversation in DB
    this.sessionId = crypto.randomUUID();
    localStorage.setItem('llama_chat_session_id', this.sessionId);
    this.activeConversationId = await createConversation('New Chat');
    await this.loadRecentConversations();
  }

  async generateTitle(conversationId: number, userFirstMsg: string, aiFirstMsg: string) {
      console.log("Generating title...");
      const summarySessionId = `summary-${crypto.randomUUID()}`;
      const prompt = `Generate a short, concise title (max 5 words) for this chat conversation. Do not use quotes.
      
      User: ${userFirstMsg.slice(0, 200)}
      AI: ${aiFirstMsg.slice(0, 200)}
      
      Title:`;
      
      let titleBuffer = "";
      
      const onEvent = new Channel<any>();
      onEvent.onmessage = (payload) => {
          if (payload.chunk) {
              titleBuffer += payload.chunk;
          }
      };
      
      try {
          // Use a clean session for summary
          await invokeCommand('send_message', {
              message: prompt,
              sessionId: summarySessionId,
              temperature: 0.7,
              maxTokens: 50,
              onEvent
          });
          
          let finalTitle = titleBuffer.trim().replace(/^["']|["']$/g, ''); // Remove quotes if any
          if (finalTitle) {
              console.log("Generated Title:", finalTitle);
              await updateConversationTitle(conversationId, finalTitle);
              await this.loadRecentConversations(); // Refresh UI
          }
          
          // Cleanup summary session
          await invokeCommand('clear_chat', { sessionId: summarySessionId });
          
      } catch (err) {
          console.warn("Title generation failed:", err);
      }
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
