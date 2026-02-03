/**
 * Backend Type Definitions
 * Mirrors Rust types for TypeScript frontend
 */

export type Role = 'user' | 'assistant' | 'system' | 'tool';

export interface Message {
    role: Role;
    content: string;
}

export interface GenerationParams {
    temperature: number;
    max_tokens: number;
    top_p: number;
    top_k: number;
}

export interface ToolSpec {
    name: string;
    description: string;
    parameters?: Record<string, unknown>;
}

export interface ToolCall {
    name: string;
    args: Record<string, unknown>;
}

export interface ChatRequest {
    messages: Message[];
    tools?: ToolSpec[];
    params: GenerationParams;
}

export type ModelOutput =
    | { type: 'assistant'; content: string }
    | { type: 'tool_call'; content: ToolCall }
    | { type: 'stream_chunk'; content: string };

// Tauri Command Requests/Responses

export interface CreateSlotRequest {
    max_ctx?: number;
}

export interface ChatMessageRequest {
    slot_id: string;
    message: string;
    params?: GenerationParams;
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
}
