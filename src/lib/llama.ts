import { invokeCommand } from './ipc';

export interface LlamaConfig {
  llama_path: string;
  model_path: string;
}

export async function initLlama(llamaPath: string, modelPath: string): Promise<string> {
  return (await invokeCommand('init_llama', { llamaPath, modelPath })) as string;
}

export async function shutdownLlama(): Promise<string> {
  return (await invokeCommand('shutdown_llama')) as string;
}

export async function sendMessage(message: string): Promise<string> {
  return (await invokeCommand('send_message', { message })) as string;
}
