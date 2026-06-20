import { createOpenAICompatible } from "@ai-sdk/openai-compatible";
import { Channel } from "@tauri-apps/api/core";
import {
  dynamicTool,
  generateText,
  jsonSchema,
  stepCountIs,
  streamText,
  type ModelMessage,
  type ToolSet,
} from "ai";
import { invokeCommand } from "$infrastructure/ipc";
import type { ToolContext } from "$lib/stores/chat.svelte";
import type { McpServerConfig, ToolDefinition } from "$lib/types/backend";
import { ThinkingStreamParser } from "./thinking";

const PROVIDER_NAME = "llamacpp";
const MODEL_ID = "local-model";
const DEFAULT_TOP_P = 0.95;
const DEFAULT_TOP_K = 40;
const MAX_TOOL_STEPS = 3;

/**
 * Final sanitization pass to remove any leaked thinking tags from assistant content.
 * This catches orphaned closing tags and malformed thinking markers that may have
 * slipped through streaming or tool integration.
 */
function sanitizeAssistantContent(content: string): string {
  if (!content) return content;

  let sanitized = content;

  // Remove orphaned closing tags
  sanitized = sanitized
    .replace(/<\/think>/g, "")
    .replace(/<\/analysis>/g, "")
    .replace(/<\/reasoning>/g, "");

  // Remove empty thinking blocks
  sanitized = sanitized
    .replace(/<think>\s*<\/think>/g, "")
    .replace(/<analysis>\s*<\/analysis>/g, "")
    .replace(/<reasoning>\s*<\/reasoning>/g, "");

  // Remove any stray opening tags without closing tags (fallback)
  // This handles cases where a thinking block was started but never closed properly
  sanitized = sanitized
    .replace(/<think>(?![^]*<\/think>)/g, "")
    .replace(/<analysis>(?![^]*<\/analysis>)/g, "")
    .replace(/<reasoning>(?![^]*<\/reasoning>)/g, "");

  return sanitized;
}

export interface AiChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

export interface RunLocalChatOptions {
  port: number;
  messages: AiChatMessage[];
  userInput: string;
  temperature: number;
  maxTokens: number;
  ctxSize: number;
  enableThinking: boolean;
  startInThinking?: boolean;
  onText: (text: string) => void;
  onThinkingChunk: (text: string) => void;
  onStatus: (text: string) => void;
  onToolContext: (context: ToolContext) => void;
  onRequestPermission?: (
    serverId: string,
    toolName: string,
    args: Record<string, unknown>,
  ) => Promise<boolean>;
}

export async function runLocalChat(
  options: RunLocalChatOptions,
): Promise<string> {
  const model = createLlamaProvider(options.port)(MODEL_ID);
  const { cleanedInput } = extractMcpIds(options.userInput);
  const messages = replaceLastUserInput(
    options.messages,
    options.userInput,
    cleanedInput,
  );
  const tools = await buildMcpTools({
    userInput: options.userInput,
    onStatus: options.onStatus,
    onToolContext: options.onToolContext,
    onRequestPermission: options.onRequestPermission,
  });
  const parser = new ThinkingStreamParser({
    startInThinking: options.startInThinking,
  });
  let fullText = "";

  const result = streamText({
    model,
    messages: trimMessagesToBudget(
      messages,
      options.ctxSize,
      options.maxTokens,
    ),
    maxOutputTokens: options.maxTokens,
    temperature: options.temperature,
    topP: DEFAULT_TOP_P,
    topK: DEFAULT_TOP_K,
    tools,
    stopWhen: stepCountIs(MAX_TOOL_STEPS),
    providerOptions: buildProviderOptions(options.enableThinking),
  });

  for await (const part of result.fullStream) {
    if (part.type === "text-delta") {
      for (const parsed of parser.push(part.text)) {
        if (parsed.type === "thinking") {
          options.onThinkingChunk(parsed.text);
        } else {
          fullText += parsed.text;
          options.onText(parsed.text);
        }
      }
    } else if (part.type === "reasoning-delta") {
      options.onThinkingChunk(part.text);
    } else if (part.type === "tool-input-start") {
      options.onStatus(`Preparing MCP tool ${part.toolName}`);
    } else if (part.type === "tool-call") {
      options.onStatus(`Calling MCP tool ${part.toolName}`);
    } else if (part.type === "tool-error") {
      throw new Error(String(part.error));
    } else if (part.type === "error") {
      throw part.error instanceof Error
        ? part.error
        : new Error(String(part.error));
    }
  }

  for (const parsed of parser.flush()) {
    if (parsed.type === "thinking") {
      options.onThinkingChunk(parsed.text);
    } else {
      fullText += parsed.text;
      options.onText(parsed.text);
    }
  }

  // Final sanitization pass to remove any leaked thinking tags
  const sanitized = sanitizeAssistantContent(fullText);
  return sanitized;
}

export async function generateLocalTitle(
  port: number,
  firstUserMessage: string,
  firstAssistantMessage: string,
): Promise<string> {
  const model = createLlamaProvider(port)(MODEL_ID);
  const result = await generateText({
    model,
    messages: [
      {
        role: "system",
        content:
          "You are a title generation assistant. Generate a concise chat title (max 8 words). Return ONLY the title. Do not use punctuation like ':' or '-' or quotation marks. Do not provide explanations or thinking.",
      },
      {
        role: "user",
        content: `User: ${firstUserMessage}\nAssistant: ${firstAssistantMessage}`,
      },
    ],
    maxOutputTokens: 64,
    temperature: 0.3,
    topP: 0.9,
    providerOptions: {
      [PROVIDER_NAME]: {
        top_k: DEFAULT_TOP_K,
        chat_template_kwargs: { enable_thinking: false },
      },
    },
  });

  return result.text;
}

function createLlamaProvider(port: number) {
  return createOpenAICompatible({
    name: PROVIDER_NAME,
    baseURL: `http://localhost:${port}/v1`,
    fetch: tauriLlamaFetch,
    transformRequestBody(body) {
      return removeUndefined(body);
    },
  });
}

function buildProviderOptions(enableThinking: boolean) {
  return {
    [PROVIDER_NAME]: {
      top_k: DEFAULT_TOP_K,
      ...(enableThinking
        ? {
            chat_template_kwargs: {
              enable_thinking: true,
              add_generation_prompt: true,
            },
          }
        : {}),
    },
  };
}

async function tauriLlamaFetch(
  _input: RequestInfo | URL,
  init?: RequestInit,
): Promise<Response> {
  const body = await bodyToText(init?.body ?? null);
  const encoder = new TextEncoder();

  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      const onEvent = new Channel<{
        chunk?: string;
        done?: boolean;
        error?: string;
      }>();
      onEvent.onmessage = (payload) => {
        if (payload.chunk) controller.enqueue(encoder.encode(payload.chunk));
        if (payload.error) controller.error(new Error(payload.error));
        if (payload.done) controller.close();
      };

      invokeCommand("proxy_llama_chat_completion", { body, onEvent }).catch(
        (error) => {
          controller.error(
            error instanceof Error ? error : new Error(String(error)),
          );
        },
      );
    },
  });

  return new Response(stream, {
    status: 200,
    headers: { "content-type": "text/event-stream" },
  });
}

async function bodyToText(body: BodyInit | null): Promise<string> {
  if (!body) return "";
  if (typeof body === "string") return body;
  if (body instanceof Uint8Array) return new TextDecoder().decode(body);
  return new Response(body).text();
}

async function buildMcpTools(options: {
  userInput: string;
  onStatus: (text: string) => void;
  onToolContext: (context: ToolContext) => void;
  onRequestPermission?: (
    serverId: string,
    toolName: string,
    args: Record<string, unknown>,
  ) => Promise<boolean>;
}): Promise<ToolSet> {
  const { mentionedIds, cleanedInput } = extractMcpIds(options.userInput);
  const servers = (await invokeCommand(
    "mcp_list_servers",
    {},
  )) as McpServerConfig[];
  const allowedServers = servers.filter((server) => {
    if (!server.enabled) return false;
    return mentionedIds.length === 0 || mentionedIds.includes(server.id);
  });

  const tools: ToolSet = {};
  for (const server of allowedServers) {
    const connected = await ensureMcpConnected(server.id);
    if (!connected) continue;

    const toolAllowlist = server.tool_allowlist ?? {};
    const definitions = await safeListTools(server.id);
    for (const definition of definitions) {
      const toolName = getToolName(definition);
      if (!toolName) continue;
      const toolId = encodeToolId(server.id, toolName);
      const description = getToolDescription(definition, server.id);
      const schema = getToolInputSchema(definition);

      tools[toolId] = dynamicTool({
        description,
        inputSchema: jsonSchema(schema as any),
        execute: async (input, execution) => {
          const args = isRecord(input) ? input : {};

          const permission = toolAllowlist[toolName] || "ask";
          if (permission === "deny") {
            throw new Error(
              `Tool ${toolName} execution denied by configuration.`,
            );
          }
          if (permission === "ask" && options.onRequestPermission) {
            options.onStatus(`Waiting for permission to run ${toolName}...`);
            const allowed = await options.onRequestPermission(
              server.id,
              toolName,
              args,
            );
            if (!allowed) {
              throw new Error(
                `User denied permission to execute tool ${toolName}.`,
              );
            }
          }

          options.onStatus(`Calling MCP tool ${server.id}::${toolName}`);
          const result = await invokeCommand("mcp_tools_call", {
            id: server.id,
            toolName,
            arguments: args,
          });
          options.onToolContext({
            serverId: server.id,
            toolName,
            arguments: args,
            result,
            toolCallId: execution.toolCallId,
          });
          return result;
        },
      });
    }
  }

  if (Object.keys(tools).length > 0) {
    options.onStatus(
      `MCP tools available for this turn: ${Object.keys(tools).length}${
        cleanedInput !== options.userInput ? " (filtered by mention)" : ""
      }`,
    );
  }

  return tools;
}

async function ensureMcpConnected(id: string): Promise<boolean> {
  try {
    await invokeCommand("mcp_connect", { id });
    return true;
  } catch (error) {
    console.warn(`Failed to connect MCP server ${id}:`, error);
    return false;
  }
}

async function safeListTools(id: string): Promise<ToolDefinition[]> {
  try {
    return (await invokeCommand("mcp_tools_list", { id })) as ToolDefinition[];
  } catch (error) {
    console.warn(`Failed to list MCP tools for ${id}:`, error);
    return [];
  }
}

function extractMcpIds(input: string) {
  const mentionedIds: string[] = [];
  const cleanedTokens: string[] = [];

  for (const token of input.split(/\s+/)) {
    const id = parseMcpToken(token);
    if (id) {
      if (!mentionedIds.includes(id)) mentionedIds.push(id);
    } else if (token) {
      cleanedTokens.push(token);
    }
  }

  return { mentionedIds, cleanedInput: cleanedTokens.join(" ").trim() };
}

function replaceLastUserInput(
  messages: AiChatMessage[],
  originalInput: string,
  cleanedInput: string,
): AiChatMessage[] {
  if (cleanedInput === originalInput) return messages;
  const index = messages.findLastIndex(
    (message) => message.role === "user" && message.content === originalInput,
  );
  if (index < 0) return messages;
  return [
    ...messages.slice(0, index),
    { ...messages[index], content: cleanedInput || originalInput },
    ...messages.slice(index + 1),
  ];
}

function parseMcpToken(token: string): string | null {
  for (const prefix of ["@mcp:", "/mcp:"]) {
    if (token.startsWith(prefix)) {
      const id = token
        .slice(prefix.length)
        .replace(/^[^a-zA-Z0-9_-]+|[^a-zA-Z0-9_-]+$/g, "")
        .trim();
      return id || null;
    }
  }
  return null;
}

function encodeToolId(serverId: string, toolName: string): string {
  const sanitize = (value: string) => value.replace(/[^a-zA-Z0-9_-]/g, "_");
  return `mcp__${sanitize(serverId)}__${sanitize(toolName)}`;
}

function getToolName(definition: ToolDefinition): string | null {
  const name = definition.name;
  return typeof name === "string" && name.trim() ? name : null;
}

function getToolDescription(
  definition: ToolDefinition,
  serverId: string,
): string {
  const description =
    typeof definition.description === "string"
      ? definition.description
      : "(no description)";
  return `[${serverId}] ${description}`;
}

function getToolInputSchema(
  definition: ToolDefinition,
): Record<string, unknown> {
  const schema = definition.inputSchema;
  if (isRecord(schema)) return schema;
  return { type: "object", properties: {} };
}

function trimMessagesToBudget(
  messages: AiChatMessage[],
  ctxSize: number,
  maxOutputTokens: number,
): ModelMessage[] {
  const promptBudget = Math.max(512, ctxSize - maxOutputTokens - 256);
  const systemMessages = messages.filter(
    (message) => message.role === "system",
  );
  const chatMessages = messages.filter((message) => message.role !== "system");
  const selected: AiChatMessage[] = [];
  let tokens = estimateMessageTokens(systemMessages);

  for (let i = chatMessages.length - 1; i >= 0; i -= 1) {
    const message = chatMessages[i];
    const messageTokens = estimateTextTokens(message.content) + 8;
    if (selected.length > 0 && tokens + messageTokens > promptBudget) break;
    selected.unshift(message);
    tokens += messageTokens;
  }

  return [...systemMessages, ...selected]
    .filter((message) => message.content.trim())
    .map((message) => ({ role: message.role, content: message.content }));
}

function estimateMessageTokens(messages: AiChatMessage[]): number {
  return messages.reduce(
    (total, message) => total + estimateTextTokens(message.content) + 8,
    0,
  );
}

function estimateTextTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function removeUndefined<T>(value: T): T {
  if (Array.isArray(value)) return value.map(removeUndefined) as T;
  if (!isRecord(value)) return value;

  return Object.fromEntries(
    Object.entries(value)
      .filter(([, entryValue]) => entryValue !== undefined)
      .map(([key, entryValue]) => [key, removeUndefined(entryValue)]),
  ) as T;
}
