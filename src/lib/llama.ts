import { invokeCommand } from './infrastructure/ipc';

export interface LlamaConfig {
  port: number;
}

/**
 * Sends a message to the llama server via the Tauri backend.
 */
export async function sendMessage(message: string): Promise<string> {
  return (await invokeCommand('send_chat_message', { message })) as string;
}
