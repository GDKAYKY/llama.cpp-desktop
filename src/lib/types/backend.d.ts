/**
 * Backend Type Definitions
 * Mirrors Rust types for TypeScript frontend
 */

export type Role = "user" | "assistant" | "system" | "tool";

/**
 * Chat message matching Rust ChatMessage struct
 * Compatible with OpenAI message format
 */
export interface Message {
  role: string;
  content: string;
  name?: string;
  tool_call_id?: string;
  tool_calls?: Array<Record<string, any>>;
}

export interface GenerationParams {
  temperature: number;
  max_tokens: number;
  top_p: number;
  top_k: number;
}

/**
 * OpenAI-compatible tool definition
 * Use this format when defining tools for the model
 */
export interface ToolSpec {
  type: "function";
  function: {
    name: string;
    description: string;
    parameters?: {
      type: "object";
      properties: Record<string, any>;
      required?: string[];
    };
  };
}

/**
 * Legacy tool spec format (deprecated, use ToolSpec instead)
 */
export interface LegacyToolSpec {
  name: string;
  description: string;
  parameters?: Record<string, unknown>;
}

export interface ToolCall {
  name: string;
  args: Record<string, unknown>;
}

/**
 * OpenAI-compatible chat request for llama.cpp server
 * Matches the Rust ChatRequest struct and llama.cpp's /v1/chat/completions endpoint
 */
export interface ChatRequest {
  model: string;
  session_id?: string;
  messages: Message[];
  temperature: number;
  top_p: number;
  top_k: number;
  max_tokens: number;
  reasoning_format?: string;
  reasoning_budget?: number;
  reasoning_budget_message?: string;
  thinking_forced_open?: boolean;
  chat_template_kwargs?: Record<string, unknown>;
  tools?: Array<Record<string, any>>;
  tool_choice?: Record<string, any> | string;
  stream: boolean;
}

/**
 * Chat response choice matching Rust ChatChoice struct
 */
export interface ChatChoice {
  message: Message;
  finish_reason: string;
}

/**
 * Chat response matching Rust ChatResponse struct
 */
export interface ChatResponse {
  choices: ChatChoice[];
  usage: Record<string, any>;
}

/**
 * Intent classification result from LLM
 */
export interface IntentClassification {
  needs_external: boolean;
  query: string;
  suggested_tool?: string;
  suggested_server?: string;
  arguments?: Record<string, any>;
  needs_multi_step?: boolean;
  multi_step_reasoning?: string;
}

export type ModelOutput =
  | { type: "assistant"; content: string }
  | { type: "tool_call"; content: ToolCall }
  | { type: "stream_chunk"; content: string };

// Tauri Command Requests/Responses

export interface CreateSlotRequest {
  max_ctx?: number;
}

export interface ChatMessageRequest {
  slot_id: string;
  message: string;
  temperature?: number;
  top_p?: number;
  top_k?: number;
  max_tokens?: number;
}

export interface SlotInfo {
  id: string;
  message_count: number;
}

// Default generation parameters
export const DEFAULT_GENERATION_PARAMS: GenerationParams = {
  temperature: 0.7,
  max_tokens: 256,
  top_p: 0.9,
  top_k: 40,
};

export interface LlamaCppConfig {
  llama_cpp_path: String;
  model_path: String;
  port: number;
  ctx_size: number;
  parallel: number;
  n_gpu_layers: number;
  jinja: boolean;
  extra_args?: string[];
  chat_template?: string | null;
  chat_template_file?: string | null;
}

export type GpuVendor = "Nvidia" | "Amd" | "Intel" | "Unknown";

export type GpuUtilization =
  | { Precise: { sm: number; mem: number } }
  | { Engine: { gpu: number } }
  | "Unavailable";

export interface ProcessGpuMetrics {
  gpu_index: number;
  gpu_name: string;
  vram_used_mb: number;
  vram_total_mb: number;
  temperature?: number | null;
  utilization: GpuUtilization;
}

export interface ServerMetrics {
  cpu_usage: number;
  mem_usage: number;
  gpu_usage?: number | null;
  vram_usage?: number | null;
  gpu_instances: ProcessGpuMetrics[];
}

export interface RunningServerInfo {
  model_id: string;
  pid: number;
  config: LlamaCppConfig;
  metrics: ServerMetrics | null;
}

export interface GpuInfo {
  index: number;
  name: string;
  vram_total_mb: number;
  vendor: GpuVendor;
}

export interface StartServerBaseOptions {
  binaryPath: string;
  modelPath: string;
  port: number;
  ctxSize: number;
  nGpuLayers: number;
  jinja: boolean;
  parallel?: number;
  extraArgs?: string[];
}

export type StartServerOptions =
  | (StartServerBaseOptions & {
      chatTemplate?: string;
      chatTemplatePath?: never;
    })
  | (StartServerBaseOptions & {
      chatTemplate?: never;
      chatTemplatePath?: string;
    });

export type McpTransport = "stdio" | "http_sse";

export interface McpServerConfig {
  id: string;
  name: string;
  enabled: boolean;
  transport: McpTransport;
  command?: string | null;
  args?: string[] | null;
  cwd?: string | null;
  env?: Record<string, string> | null;
  url?: string | null;
  headers?: Record<string, string> | null;
  tool_allowlist?: string[] | null;
  resource_allowlist?: string[] | null;
}

export interface McpConfig {
  servers: McpServerConfig[];
}

export interface McpServerStatus {
  id: string;
  connected: boolean;
  last_error?: string | null;
  tools_cached: number;
  resources_cached: number;
  capabilities?: McpCapabilities | null;
}

export interface McpInferredTool {
  name: string;
  method: string;
}

export interface McpCapabilities {
  has_tools_list: boolean;
  has_resources_list: boolean;
  supports_tools_call: boolean;
  supports_resources_read: boolean;
  inferred_tools: McpInferredTool[];
  last_error?: string | null;
}

export type ToolDefinition = Record<string, any>;
export type ResourceDefinition = Record<string, any>;
