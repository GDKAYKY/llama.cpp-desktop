/**
 * Chat API Helpers
 * Utilities for constructing llama.cpp-compatible chat requests
 */

import type { ChatRequest, Message, ToolSpec, GenerationParams } from '../types/backend';

/**
 * Default values for chat requests
 */
export const DEFAULT_CHAT_CONFIG = {
    model: 'local-model',
    temperature: 0.7,
    top_p: 0.9,
    top_k: 40,
    max_tokens: 512,
    stream: false,
} as const;

/**
 * Builder for constructing ChatRequest objects
 * 
 * @example
 * ```typescript
 * const request = new ChatRequestBuilder()
 *   .addMessage('user', 'Hello!')
 *   .setTemperature(0.8)
 *   .addTool({
 *     type: 'function',
 *     function: {
 *       name: 'get_weather',
 *       description: 'Get weather for a location',
 *       parameters: {
 *         type: 'object',
 *         properties: {
 *           location: { type: 'string' }
 *         },
 *         required: ['location']
 *       }
 *     }
 *   })
 *   .build();
 * ```
 */
export class ChatRequestBuilder {
    private request: Partial<ChatRequest> = {
        ...DEFAULT_CHAT_CONFIG,
        messages: [],
    };

    constructor(model?: string) {
        if (model) {
            this.request.model = model;
        }
    }

    addMessage(role: string, content: string): this {
        this.request.messages!.push({ role, content });
        return this;
    }

    addSystemMessage(content: string): this {
        return this.addMessage('system', content);
    }

    addUserMessage(content: string): this {
        return this.addMessage('user', content);
    }

    addAssistantMessage(content: string): this {
        return this.addMessage('assistant', content);
    }

    setMessages(messages: Message[]): this {
        this.request.messages = messages;
        return this;
    }

    setTemperature(temperature: number): this {
        this.request.temperature = temperature;
        return this;
    }

    setTopP(top_p: number): this {
        this.request.top_p = top_p;
        return this;
    }

    setTopK(top_k: number): this {
        this.request.top_k = top_k;
        return this;
    }

    setMaxTokens(max_tokens: number): this {
        this.request.max_tokens = max_tokens;
        return this;
    }

    setStream(stream: boolean): this {
        this.request.stream = stream;
        return this;
    }

    addTool(tool: ToolSpec): this {
        if (!this.request.tools) {
            this.request.tools = [];
        }
        this.request.tools.push(tool as any);
        return this;
    }

    setTools(tools: ToolSpec[]): this {
        this.request.tools = tools as any;
        return this;
    }

    setToolChoice(choice: string | Record<string, any>): this {
        this.request.tool_choice = choice;
        return this;
    }

    setSessionId(session_id: string): this {
        this.request.session_id = session_id;
        return this;
    }

    /**
     * Enable reasoning/thinking mode (for models that support it)
     */
    enableReasoning(budget?: number, format?: string): this {
        this.request.reasoning_format = format || 'thinking';
        if (budget) {
            this.request.reasoning_budget = budget;
        }
        return this;
    }

    build(): ChatRequest {
        if (!this.request.messages || this.request.messages.length === 0) {
            throw new Error('ChatRequest must have at least one message');
        }
        return this.request as ChatRequest;
    }
}

/**
 * Create a simple chat request with minimal configuration
 */
export function createSimpleChatRequest(
    userMessage: string,
    systemPrompt?: string,
    params?: Partial<GenerationParams>
): ChatRequest {
    const builder = new ChatRequestBuilder();
    
    if (systemPrompt) {
        builder.addSystemMessage(systemPrompt);
    }
    
    builder.addUserMessage(userMessage);
    
    if (params) {
        if (params.temperature !== undefined) builder.setTemperature(params.temperature);
        if (params.top_p !== undefined) builder.setTopP(params.top_p);
        if (params.top_k !== undefined) builder.setTopK(params.top_k);
        if (params.max_tokens !== undefined) builder.setMaxTokens(params.max_tokens);
    }
    
    return builder.build();
}

/**
 * Create a tool definition following OpenAI format
 */
export function createTool(
    name: string,
    description: string,
    parameters: {
        properties: Record<string, any>;
        required?: string[];
    }
): ToolSpec {
    return {
        type: 'function',
        function: {
            name,
            description,
            parameters: {
                type: 'object',
                ...parameters,
            },
        },
    };
}

/**
 * Example: Weather tool definition
 */
export const EXAMPLE_WEATHER_TOOL = createTool(
    'get_weather',
    'Get the current weather for a location',
    {
        properties: {
            location: {
                type: 'string',
                description: 'City name or coordinates',
            },
            units: {
                type: 'string',
                enum: ['celsius', 'fahrenheit'],
                description: 'Temperature units',
            },
        },
        required: ['location'],
    }
);
