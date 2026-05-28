import { invokeCommand } from "$infrastructure/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { settingsStore } from "$lib/stores/settings.svelte";
import { modelsStore } from "$lib/stores/models.svelte";
import { serverStore } from "$lib/stores/server.svelte";
import { generateLocalTitle, runLocalChat, type AiChatMessage } from "$lib/ai/llama";
import {
  saveMessage,
  createConversation,
  getConversationHistory,
  getRecentConversations,
  truncateConversationFromIndex,
  updateConversationTitle,
  updateConversationMessageAtIndex,
  deleteConversation,
  estimateTokens,
  type Conversation,
} from "$lib/services/history";

export interface Message {
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
  tokens?: number;
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

    // Stable local session id for persisted UI metadata and action logs.
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
      tokens: h.tokens,
      model: h.model,
      thinkingProcess: h.thinkingProcess,
      modelThinking: h.modelThinking,
      toolContext: h.toolContext,
    }));

    this.activeConversationId = id;

    // The AI SDK request is built from frontend history on each turn. Rust does
    // not own or mirror conversation state.
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
      tokens: estimateTokens(content),
    };

    this.messages.push(userMessage);
    await saveMessage(conversationId, "user", content);

    this.messages.push({
      role: "assistant",
      content: "",
      timestamp: Date.now(),
      tokens: 0,
    });

    this.isLoading = true;
    this.error = null;
    await this.refreshThinkingLabel();
    this.thinkingProcess = [];
    this.modelThinking = "";
    this.toolContext = [];
    this.thinkingLineBuffer = "";
    this.currentAssistantResponse = "";

    const requestMessages = this.buildAiMessages();

    try {
      await runLocalChat({
        port: this.getRunningPort(),
        messages: requestMessages,
        userInput: content,
        temperature: settingsStore.settings.temperature,
        maxTokens: settingsStore.settings.maxTokens,
        ctxSize: this.getRunningCtxSize(),
        enableThinking: this.hasExplicitChatTemplate(),
        onText: (text) => this.appendChunk(text),
        onThinkingChunk: (text) => this.appendThinkingChunk(text),
        onStatus: (text) => this.appendThinkingStatus(text),
        onToolContext: (context) => {
          this.toolContext = [...this.toolContext, context];
        },
      });

      await this.finishAssistantResponse(conversationId);
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
      content: this.messages[messageIndex]?.content ?? "",
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

    let buffer = "";

    await this.refreshThinkingLabel();
    this.thinkingProcess = [];
    this.modelThinking = "";
    this.toolContext = [];
    this.thinkingLineBuffer = "";

    const historyBeforeTarget = this.messages.slice(0, messageIndex);

    try {
      await runLocalChat({
        port: this.getRunningPort(),
        messages: this.buildAiMessages(historyBeforeTarget),
        userInput: historyBeforeTarget.findLast((message) => message.role === "user")?.content ?? "",
        temperature: settingsStore.settings.temperature,
        maxTokens: settingsStore.settings.maxTokens,
        ctxSize: this.getRunningCtxSize(),
        enableThinking: this.hasExplicitChatTemplate(),
        onText: (text) => {
          buffer += text;
          this.updateAssistantAt(messageIndex, { content: buffer });
        },
        onThinkingChunk: (text) => this.appendThinkingChunk(text),
        onStatus: (text) => this.appendThinkingStatus(text),
        onToolContext: (context) => {
          this.toolContext = [...this.toolContext, context];
        },
      });

      this.flushThinkingBuffer();
      const modelName = this.getRunningModelName();
      const msg = this.messages[messageIndex];
      const finalContent = buffer || (msg?.role === "assistant" ? msg.content : "");
      this.updateAssistantAt(messageIndex, {
        content: finalContent,
        model: modelName,
        tokens: estimateTokens(finalContent),
      });

      await updateConversationMessageAtIndex(conversationId, messageIndex, {
        content: finalContent,
        model: modelName,
        ...this.buildAssistantDebugMeta(),
      });
      await this.loadRecentConversations();
      this.attachDebugToAssistantAt(messageIndex);
      this.resetStreamingState();
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
      const result = await generateLocalTitle(
        this.getRunningPort(),
        userFirstMsg.slice(0, 500),
        assistantFirstMsg.slice(0, 500),
      );

      console.log("generateLocalTitle raw result:", JSON.stringify(result));

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

  private appendThinkingStatus(text: string) {
    if (!text.trim()) return;
    this.thinkingProcess = [...this.thinkingProcess, text.trim()];
  }

  private buildAiMessages(source: Message[] = this.messages): AiChatMessage[] {
    return source
      .filter((message) => message.role !== "assistant" || message.content.trim())
      .map((message) => ({ role: message.role, content: message.content }));
  }

  private async finishAssistantResponse(conversationId: number) {
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

    this.updateAssistantAt(this.messages.length - 1, {
      model: modelName,
      tokens: estimateTokens(assistantContent),
    });

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
    this.attachDebugToAssistantAt(this.messages.length - 1);
    this.resetStreamingState();
  }

  private updateAssistantAt(index: number, patch: Partial<Message>) {
    const msg = this.messages[index];
    if (!msg || msg.role !== "assistant") return;
    this.messages = [
      ...this.messages.slice(0, index),
      { ...msg, ...patch },
      ...this.messages.slice(index + 1),
    ];
  }

  private getRunningPort() {
    const port = serverStore.currentConfig?.port;
    if (!port) throw new Error("No llama.cpp server is running");
    return port;
  }

  private getRunningCtxSize() {
    return serverStore.currentConfig?.ctx_size ?? 4096;
  }

  private hasExplicitChatTemplate() {
    const config = serverStore.currentConfig;
    return Boolean(config?.chat_template || config?.chat_template_file);
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
