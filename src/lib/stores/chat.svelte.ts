import { invokeCommand } from "$infrastructure/ipc";
import { Channel } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { settingsStore } from "$lib/stores/settings.svelte";
import { modelsStore } from "$lib/stores/models.svelte";
import { serverStore } from "$lib/stores/server.svelte";
import {
  saveMessage,
  createConversation,
  getConversationHistory,
  getRecentConversations,
  truncateConversationFromIndex,
  updateConversationTitle,
  updateConversationMessageAtIndex,
  deleteConversation,
  type Conversation,
} from "$lib/services/history";

export interface Message {
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
  model?: string;
  thinkingProcess?: string[];
  modelThinking?: string;
  toolContext?: ToolContext[];
}

export interface ToolContext {
  serverId?: string;
  toolName?: string;
  arguments?: unknown;
  result?: unknown;
  toolCallId?: string;
}

class ChatStore {
  messages = $state<Message[]>([]);
  thinkingProcess = $state<string[]>([]);
  modelThinking = $state("");
  thinkingLabel = $state("Thinking");
  thinkingTags = $state<string[]>([]);
  toolContext = $state<ToolContext[]>([]);
  isLoading = $state(false);
  error = $state<string | null>(null);
  modelLoaded = $state(true);

  // DB IDs
  activeConversationId = $state<number | null>(null);
  history = $state<Conversation[]>([]);

  sessionId = $state<string>("");

  unlisten: UnlistenFn | null = null;

  // Streaming buffers stay non-reactive on purpose to avoid UI churn.
  currentAssistantResponse = "";
  private lastTemplateKey: string | null = null;
  private thinkingLineBuffer = "";

  async initialize() {
    this.error = null;

    // Backend session: stable while the app is open.
    const savedSession = localStorage.getItem("llama_chat_session_id");
    if (savedSession) {
      this.sessionId = savedSession;
    } else {
      this.sessionId = crypto.randomUUID();
      localStorage.setItem("llama_chat_session_id", this.sessionId);
    }

    try {
      await this.loadRecentConversations();

      if (this.history.length > 0) {
        const lastActive = this.history[0];
        if (lastActive?.id) {
          this.activeConversationId = lastActive.id;
          await this.loadConversation(lastActive.id);
        }
      } else {
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

    this.messages = history.map((h) => ({
      role: h.role,
      content: h.content,
      timestamp: h.timestamp,
      model: h.model,
      thinkingProcess: h.thinkingProcess,
      modelThinking: h.modelThinking,
      toolContext: h.toolContext,
    }));

    this.activeConversationId = id;

    // IMPORTANT:
    // Do NOT regenerate sessionId here.
    // The backend session should stay stable; otherwise context hydration and
    // title generation can drift into different sessions like an amateur magician.

    try {
      // Business rule: backend context is rebuilt from the canonical transcript
      // only. Persisted thinking/debug metadata may exist for UI rendering after
      // reload, but it must never be re-injected into model context.
      const contextPayload = history.map((h) => ({
        role: h.role,
        content: h.content,
      }));

      await invokeCommand("load_history_context", {
        sessionId: this.sessionId,
        messages: contextPayload,
      });

      // console.log('Backend context hydrated for session:', this.sessionId);
    } catch (err) {
      console.warn("Failed to hydrate backend context:", err);
    }
  }

  appendChunk(chunk: string) {
    if (this.messages.length === 0) return;

    const lastIndex = this.messages.length - 1;
    const lastMsg = this.messages[lastIndex];

    if (lastMsg.role === "assistant") {
      const updated = { ...lastMsg, content: lastMsg.content + chunk };
      this.messages = [...this.messages.slice(0, lastIndex), updated];
      this.currentAssistantResponse += chunk;
    } else {
      console.warn("Assistant chunk received without a placeholder message");
    }
  }

  async send(content: string) {
    if (!this.activeConversationId) {
      this.activeConversationId = await createConversation(
        content.slice(0, 30),
      );
    }

    if (!this.sessionId) {
      this.sessionId = crypto.randomUUID();
      localStorage.setItem("llama_chat_session_id", this.sessionId);
    }

    const conversationId = this.activeConversationId;
    if (!conversationId) {
      throw new Error("Failed to create an active conversation before sending");
    }

    const userMessage: Message = {
      role: "user",
      content,
      timestamp: Date.now(),
    };

    this.messages.push(userMessage);
    await saveMessage(conversationId, "user", content);

    this.messages.push({
      role: "assistant",
      content: "",
      timestamp: Date.now(),
    });

    this.isLoading = true;
    this.error = null;
    await this.refreshThinkingLabel();
    this.thinkingProcess = [];
    this.modelThinking = "";
    this.toolContext = [];
    this.thinkingLineBuffer = "";
    this.currentAssistantResponse = "";

    const onEvent = new Channel<any>();
    let eventQueue = Promise.resolve();
    let streamError: unknown = null;
    onEvent.onmessage = (payload) => {
      eventQueue = eventQueue
        .then(async () => {
          if (payload.thinking) {
            this.thinkingProcess = [
              ...this.thinkingProcess,
              String(payload.thinking),
            ];
          }

          if (payload.thinking_chunk) {
            this.appendThinkingChunk(String(payload.thinking_chunk));
          }

          if (payload.tool_context) {
            const ctx = payload.tool_context;
            this.toolContext = [
              ...this.toolContext,
              {
                serverId: ctx.server_id ? String(ctx.server_id) : undefined,
                toolName: ctx.tool_name ? String(ctx.tool_name) : undefined,
                arguments: ctx.arguments,
                result: ctx.result,
                toolCallId: ctx.tool_call_id ? String(ctx.tool_call_id) : undefined,
              },
            ];
          }

          if (payload.chunk) {
            this.appendChunk(String(payload.chunk));
          }

          if (payload.status === "done") {
            console.log("Stream finished");

            try {
              this.flushThinkingBuffer();

              const lastMsg = this.messages[this.messages.length - 1];
              const assistantContent =
                this.currentAssistantResponse ||
                (lastMsg?.role === "assistant" ? lastMsg.content : "");
              const trimmedContent = assistantContent.trim();

              if (!trimmedContent) {
                console.warn("Assistant stream finished without content to persist");
                return;
              }

              this.currentAssistantResponse = assistantContent;

              const modelName = this.getRunningModelName();

              await saveMessage(
                conversationId,
                "assistant",
                assistantContent,
                modelName,
                this.buildAssistantDebugMeta(),
              );

              const lastAssistantIndex = this.messages.length - 1;
              const lastAssistant = this.messages[lastAssistantIndex];
              if (lastAssistant?.role === "assistant") {
                this.messages = [
                  ...this.messages.slice(0, lastAssistantIndex),
                  { ...lastAssistant, model: modelName },
                  ...this.messages.slice(lastAssistantIndex + 1),
                ];
              }

              const userMessages = this.messages.filter((m) => m.role === "user");
              const assistantMessages = this.messages.filter(
                (m) => m.role === "assistant" && m.content.trim(),
              );
              if (
                userMessages.length === 1 &&
                assistantMessages.length === 1 &&
                this.currentAssistantResponse
              ) {
                this.generateTitle(
                  conversationId,
                  userMessages[0].content,
                  this.currentAssistantResponse,
                ).catch((err) => {
                  console.warn("Background title generation error:", err);
                });
              }

              await this.loadRecentConversations();
            } finally {
              this.attachDebugToAssistantAt(this.messages.length - 1);
              this.resetStreamingState();
            }
          }
        })
        .catch((err) => {
          streamError ??= err;
          this.error = err instanceof Error ? err.message : String(err);
          console.error("Chat stream event failed:", err);
        });
    };

    try {
      await invokeCommand("send_message", {
        message: content,
        sessionId: this.sessionId,
        temperature: settingsStore.settings.temperature,
        maxTokens: settingsStore.settings.maxTokens,
        onEvent,
      });
      await eventQueue;
      if (streamError) {
        throw streamError;
      }
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
      console.error("ERRO NO CHAT:", err);
    } finally {
      this.isLoading = false;
    }
  }

  async likeMessage(messageIndex: number) {
    if (this.isLoading) return;

    await invokeCommand("chat_action_like", {
      sessionId: this.sessionId,
      messageIndex,
    });
  }

  async dislikeMessage(messageIndex: number) {
    if (this.isLoading) return;

    await invokeCommand("chat_action_dislike", {
      sessionId: this.sessionId,
      messageIndex,
    });
  }

  async copyMessage(messageIndex: number) {
    if (this.isLoading) return;

    await invokeCommand("chat_action_copy", {
      sessionId: this.sessionId,
      messageIndex,
    });
  }

  async shareMessage(messageIndex: number): Promise<string> {
    if (this.isLoading) return "";

    const result = await invokeCommand("chat_action_share", {
      sessionId: this.sessionId,
      messageIndex,
    });
    return String(result);
  }

  async regenerateMessage(messageIndex: number) {
    if (this.isLoading) return;

    const target = this.messages[messageIndex];
    if (!target || target.role !== "assistant") {
      throw new Error("Target message is not an assistant response");
    }

    const conversationId = this.activeConversationId;
    if (!conversationId) {
      throw new Error("No active conversation available for regeneration");
    }

    this.isLoading = true;
    this.error = null;

    const onEvent = new Channel<any>();
    let buffer = "";
    let eventQueue = Promise.resolve();
    let streamError: unknown = null;

    await this.refreshThinkingLabel();
    this.thinkingProcess = [];
    this.modelThinking = "";
    this.toolContext = [];
    this.thinkingLineBuffer = "";

    onEvent.onmessage = (payload) => {
      eventQueue = eventQueue
        .then(async () => {
          if (payload.thinking) {
            this.thinkingProcess = [
              ...this.thinkingProcess,
              String(payload.thinking),
            ];
          }

          if (payload.thinking_chunk) {
            this.appendThinkingChunk(String(payload.thinking_chunk));
          }

          if (payload.tool_context) {
            const ctx = payload.tool_context;
            this.toolContext = [
              ...this.toolContext,
              {
                serverId: ctx.server_id ? String(ctx.server_id) : undefined,
                toolName: ctx.tool_name ? String(ctx.tool_name) : undefined,
                arguments: ctx.arguments,
                result: ctx.result,
                toolCallId: ctx.tool_call_id ? String(ctx.tool_call_id) : undefined,
              },
            ];
          }

          if (payload.chunk) {
            buffer += String(payload.chunk);
            const msg = this.messages[messageIndex];
            if (msg && msg.role === "assistant") {
              const updated = { ...msg, content: buffer };
              this.messages = [
                ...this.messages.slice(0, messageIndex),
                updated,
                ...this.messages.slice(messageIndex + 1),
              ];
            }
          }

          if (payload.status === "done") {
            try {
              this.flushThinkingBuffer();

              const modelName = this.getRunningModelName();
              const msg = this.messages[messageIndex];
              const finalContent =
                buffer || (msg?.role === "assistant" ? msg.content : "");
              if (msg && msg.role === "assistant") {
                this.messages = [
                  ...this.messages.slice(0, messageIndex),
                  { ...msg, content: finalContent, model: modelName },
                  ...this.messages.slice(messageIndex + 1),
                ];
              }

              await updateConversationMessageAtIndex(conversationId, messageIndex, {
                content: finalContent,
                model: modelName,
                ...this.buildAssistantDebugMeta(),
              });
              await this.loadRecentConversations();
            } finally {
              this.attachDebugToAssistantAt(messageIndex);
              this.resetStreamingState();
            }
          }
        })
        .catch((err) => {
          streamError ??= err;
          this.error = err instanceof Error ? err.message : String(err);
          console.error("Regenerate stream event failed:", err);
        });
    };

    try {
      await invokeCommand("chat_action_regenerate", {
        sessionId: this.sessionId,
        messageIndex,
        temperature: settingsStore.settings.temperature,
        maxTokens: settingsStore.settings.maxTokens,
        onEvent,
      });
      await eventQueue;
      if (streamError) {
        throw streamError;
      }
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
      throw err;
    } finally {
      this.isLoading = false;
    }
  }

  async editMessage(index: number, content: string) {
    if (this.isLoading) return;

    const conversationId = this.activeConversationId;
    if (!conversationId) {
      throw new Error("No active conversation available for editing");
    }

    const truncatedMessages = this.messages.slice(0, index);
    await truncateConversationFromIndex(conversationId, index);
    this.messages = truncatedMessages;
    await this.replaceBackendHistory(truncatedMessages);
    await this.send(content);
  }

  private attachDebugToAssistantAt(index: number) {
    const target = this.messages[index];
    if (!target || target.role !== "assistant") return;

    const updated: Message = {
      ...target,
      thinkingProcess: this.thinkingProcess.length
        ? [...this.thinkingProcess]
        : undefined,
      modelThinking: this.modelThinking ? this.modelThinking : undefined,
      toolContext: this.toolContext.length ? [...this.toolContext] : undefined,
    };

    this.messages = [
      ...this.messages.slice(0, index),
      updated,
      ...this.messages.slice(index + 1),
    ];
  }

  async clear() {
    try {
      await invokeCommand("clear_chat", { sessionId: this.sessionId });
    } catch (err) {
      console.warn("Failed to clear backend chat session:", err);
    }

    this.messages = [];
    this.error = null;

    this.sessionId = crypto.randomUUID();
    localStorage.setItem("llama_chat_session_id", this.sessionId);

    this.activeConversationId = await createConversation("New Chat");
    await this.loadRecentConversations();
  }

  async deleteChat(id: number) {
    await deleteConversation(id);
    await this.loadRecentConversations();

    if (this.activeConversationId === id) {
      if (this.history.length > 0) {
        const nextChat = this.history[0];
        if (nextChat?.id) {
          await this.loadConversation(nextChat.id);
        }
      } else {
        await this.clear();
      }
    }
  }
  async generateTitle(
    conversationId: number,
    userFirstMsg: string,
    assistantFirstMsg: string,
  ) {
    console.log("=== Generating title for conversation", conversationId);

    try {
      const result = await invokeCommand("generate_chat_title", {
        firstUserMessage: userFirstMsg.slice(0, 500),
        firstAssistantMessage: assistantFirstMsg.slice(0, 500),
      });

      console.log("generate_chat_title raw result:", JSON.stringify(result));

      const rawTitle = typeof result === "string" ? result : "";

      console.log("rawTitle:", JSON.stringify(rawTitle));

      const finalTitle = rawTitle
        .trim()
        .replace(/^["']|["']$/g, "")
        .replace(/^Title:\s*/i, "")
        .split("\n")[0]
        .slice(0, 50);

      console.log("finalTitle after cleanup:", JSON.stringify(finalTitle));

      if (finalTitle) {
        console.log("Generated title:", finalTitle);
        await updateConversationTitle(conversationId, finalTitle);
        await this.loadRecentConversations();
      } else {
        console.warn("finalTitle was empty — skipping update");
      }
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
    this.activeConversationId = null;
    this.history = [];
    this.sessionId = "";
    localStorage.removeItem("llama_chat_session_id");
  }

  private buildAssistantDebugMeta() {
    return {
      thinkingProcess: this.thinkingProcess.length
        ? [...this.thinkingProcess]
        : undefined,
      modelThinking: this.modelThinking || undefined,
      toolContext: this.toolContext.length ? [...this.toolContext] : undefined,
    };
  }

  private getRunningModelName() {
    const runningModelPath = serverStore.currentConfig?.model_path;
    const modelInLibrary = modelsStore.models.find(
      (m) => m.model_file_path === runningModelPath,
    );
    return modelInLibrary?.name || "Unknown Model";
  }

  private async replaceBackendHistory(messages: Message[]) {
    await invokeCommand("clear_chat", { sessionId: this.sessionId });
    await invokeCommand("load_history_context", {
      sessionId: this.sessionId,
      messages: messages.map((message) => ({
        role: message.role,
        content: message.content,
      })),
    });
  }

  private resetStreamingState() {
    this.thinkingProcess = [];
    this.modelThinking = "";
    this.toolContext = [];
    this.thinkingLineBuffer = "";
  }

  private async refreshThinkingLabel() {
    const config = serverStore.currentConfig;
    const inlineTemplate = config?.chat_template ?? null;
    const runningModelPath = config?.model_path;
    const modelInLibrary = runningModelPath
      ? modelsStore.models.find((m) => m.model_file_path === runningModelPath)
      : null;
    const metadataTemplate =
      typeof modelInLibrary?.tokenizer_metadata?.["tokenizer.chat_template"] ===
      "string"
        ? modelInLibrary.tokenizer_metadata["tokenizer.chat_template"]
        : null;

    const templateKey = inlineTemplate
      ? `inline:${inlineTemplate.length}`
      : metadataTemplate
        ? `metadata:${metadataTemplate.length}`
        : null;

    if (!templateKey) {
      this.thinkingLabel = "Thinking";
      this.thinkingTags = [];
      this.lastTemplateKey = null;
      return;
    }

    if (this.lastTemplateKey === templateKey) {
      return;
    }

    const templateText: string | null = inlineTemplate ?? metadataTemplate;

    this.thinkingLabel = deriveThinkingLabelFromTemplate(templateText);
    this.thinkingTags = deriveThinkingTagsFromTemplate(templateText);
    this.lastTemplateKey = templateKey;
  }

  private appendThinkingChunk(chunk: string) {
    if (!chunk) return;

    this.modelThinking += chunk;

    const combined = `${this.thinkingLineBuffer}${chunk}`;
    const lines = combined.split(/\r?\n/);
    this.thinkingLineBuffer = lines.pop() ?? "";

    const cleaned = lines.map((line) => line.trim()).filter(Boolean);
    if (cleaned.length > 0) {
      this.thinkingProcess = [...this.thinkingProcess, ...cleaned];
    }
  }

  private flushThinkingBuffer() {
    const remaining = this.thinkingLineBuffer.trim();
    if (remaining) {
      this.thinkingProcess = [...this.thinkingProcess, remaining];
    }
    this.thinkingLineBuffer = "";
  }
}

export const chatStore = new ChatStore();

function deriveThinkingLabelFromTemplate(template: string | null): string {
  if (!template) return "Thinking";

  const lower = template.toLowerCase();
  if (lower.includes("<analysis>") || lower.includes("</analysis>")) {
    return "Analysis";
  }
  if (lower.includes("<reasoning>") || lower.includes("</reasoning>")) {
    return "Reasoning";
  }
  if (lower.includes("<think>") || lower.includes("</think>")) {
    return "Thinking";
  }

  return "Thinking";
}

function deriveThinkingTagsFromTemplate(template: string | null): string[] {
  if (!template) return [];
  const tags = new Set<string>();
  const tagRegex = /<([a-zA-Z][a-zA-Z0-9_-]{0,32})>/g;
  const blocked = new Set(["assistant", "user", "system", "tool"]);
  let match = tagRegex.exec(template);
  while (match) {
    const tag = match[1];
    if (!blocked.has(tag.toLowerCase())) {
      const closingTag = new RegExp(`</${escapeRegExp(tag)}>`, "i");
      if (closingTag.test(template)) {
        tags.add(tag);
      }
    }
    match = tagRegex.exec(template);
  }
  return [...tags];
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
