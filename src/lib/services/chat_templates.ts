import { invokeCommand } from '../infrastructure/ipc';

// ─── Tipos ────────────────────────────────────────────────────────────────────

export interface StartServerOptions {
  binaryPath: string;
  modelPath: string;
  port: number;
  ctxSize: number;
  nGpuLayers: number;
  parallel?: number;
  /** Inline template string (passado diretamente ao --chat-template) */
  chatTemplate?: string;
  /** Caminho para um arquivo .jinja local (passado ao --chat-template-file) */
  chatTemplatePath?: string;
}

// ─── ensure_chat_template ─────────────────────────────────────────────────────

/**
 * Garante que o chat template Jinja do repo HuggingFace esteja em cache local.
 *
 * Fluxo:
 *   1. Se o arquivo já existir em cache → retorna o caminho imediatamente.
 *   2. Caso contrário:
 *      a. Tenta baixar `chat_template.jinja` direto do HuggingFace.
 *      b. Se receber 404, faz fallback para `tokenizer_config.json` e extrai
 *         o campo `chat_template`.
 *      c. Salva o resultado em `<app_data>/chat_templates/<slug>.jinja`.
 *
 * @param hfRepo Identificador do repositório HuggingFace, ex:
 *               "meta-llama/Llama-3.2-1B-Instruct"
 * @returns Caminho absoluto do arquivo `.jinja` em cache.
 */
export async function ensureChatTemplate(hfRepo: string): Promise<string> {
  return (await invokeCommand('ensure_chat_template', {
    hfRepo,
  })) as string;
}

// ─── start_llama_server ───────────────────────────────────────────────────────

/**
 * Inicia o llama-server com os parâmetros fornecidos.
 *
 * Se `chatTemplatePath` for fornecido, os flags `--jinja` e
 * `--chat-template-file` serão adicionados automaticamente pelo backend.
 *
 * @returns PID do processo como string.
 */
export async function startLlamaServer(
  options: StartServerOptions,
): Promise<string> {
  return (await invokeCommand('start_llama_server', {
    binaryPath: options.binaryPath,
    modelPath: options.modelPath,
    port: options.port,
    ctxSize: options.ctxSize,
    nGpuLayers: options.nGpuLayers,
    parallel: options.parallel ?? null,
    chatTemplate: options.chatTemplate ?? null,
    chatTemplateFile: options.chatTemplatePath ?? null,
  })) as string;
}

// ─── Utilitário composto ──────────────────────────────────────────────────────

/**
 * Fluxo completo: garante o template em cache e depois inicia o servidor.
 *
 * Exemplo de uso:
 * ```ts
 * const pid = await ensureTemplateAndStartServer(
 *   'meta-llama/Llama-3.2-1B-Instruct',
 *   {
 *     binaryPath: '/usr/local/bin/llama-server',
 *     modelPath:  '/models/llama-3.2-1b-instruct.gguf',
 *     port:       8080,
 *     ctxSize:    4096,
 *     nGpuLayers: 35,
 *   },
 * );
 * ```
 *
 * @param hfRepo     Repositório HuggingFace do modelo.
 * @param options    Opções do servidor (sem `chatTemplatePath` — preenchido aqui).
 */
export async function ensureTemplateAndStartServer(
  hfRepo: string,
  options: Omit<StartServerOptions, 'chatTemplatePath'>,
): Promise<string> {
  // Passo 1 — garante (ou baixa) o template
  const chatTemplatePath = await ensureChatTemplate(hfRepo);

  // Passo 2 — inicia o servidor com o template em cache
  return startLlamaServer({ ...options, chatTemplatePath });
}
