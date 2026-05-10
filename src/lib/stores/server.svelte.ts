import { invokeCommand } from '$infrastructure/ipc';
import type { LlamaCppConfig, StartServerOptions } from '$lib/types/backend';

export interface ServerStatus {
    isRunning: boolean;
    isHealthy: boolean;
    error: string | null;
    currentConfig: LlamaCppConfig | null;
}

class ServerStore {
    isRunning = $state(false);
    isHealthy = $state(false);
    error = $state<string | null>(null);
    isChecking = $state(false);
    isStarting = $state(false);
    currentConfig = $state<LlamaCppConfig | null>(null);
    serverMetrics = $state<{ 
        cpu_usage: number; 
        mem_usage: number; 
        gpu_usage?: number; 
        vram_usage?: number; 
    } | null>(null);
    private healthInterval: ReturnType<typeof setInterval> | null = null;

    private sameArgs(left?: string[] | null, right?: string[] | null) {
        const leftArgs = left ?? [];
        const rightArgs = right ?? [];

        return leftArgs.length === rightArgs.length && leftArgs.every((arg, index) => arg === rightArgs[index]);
    }

    private normalizeArgs(args?: string[] | null) {
        return (args ?? []).map((arg) => arg.trim()).filter(Boolean);
    }

    constructor() {
        this.init();
    }

    async init() {
        await this.checkRunning();
        if (this.isRunning) {
            await this.checkHealth();
            this.startHealthMonitoring();
        }
    }

    async startServer(options: StartServerOptions) {
        if (this.isStarting) return;

        if (!options.binaryPath?.trim()) {
            this.error = 'Llama server binary path is required';
            return;
        }

        if (!options.modelPath?.trim()) {
            this.error = 'Model path is required';
            return;
        }

        const extraArgs = this.normalizeArgs(options.extraArgs);

        if (
            this.isRunning &&
            this.currentConfig?.llama_cpp_path === options.binaryPath &&
            this.currentConfig?.model_path === options.modelPath &&
            this.currentConfig?.port === options.port &&
            this.currentConfig?.ctx_size === options.ctxSize &&
            this.currentConfig?.n_gpu_layers === options.nGpuLayers &&
            this.currentConfig?.jinja === options.jinja &&
            this.currentConfig?.parallel === (options.parallel ?? 1) &&
            this.sameArgs(this.currentConfig?.extra_args ?? [], extraArgs) &&
            (this.currentConfig?.chat_template ?? null) === (options.chatTemplate ?? null) &&
            (this.currentConfig?.chat_template_file ?? null) === (options.chatTemplatePath ?? null)
        ) {
            return;
        }
        try {
            this.error = null;
            this.isStarting = true;
            const pid = await invokeCommand('start_llama_server', {
                binaryPath: options.binaryPath,
                modelPath: options.modelPath,
                port: options.port,
                ctxSize: options.ctxSize,
                nGpuLayers: options.nGpuLayers,
                jinja: options.jinja,
                parallel: options.parallel ?? null,
                extraArgs: extraArgs.length > 0 ? extraArgs : null,
                chatTemplate: options.chatTemplate ?? null,
                chatTemplateFile: options.chatTemplatePath ?? null,
            });
            this.isRunning = true;
            this.currentConfig = {
                llama_cpp_path: options.binaryPath,
                model_path: options.modelPath,
                port: options.port,
                ctx_size: options.ctxSize,
                parallel: options.parallel ?? 1,
                n_gpu_layers: options.nGpuLayers,
                jinja: options.jinja,
                extra_args: extraArgs,
                chat_template: options.chatTemplate ?? null,
                chat_template_file: options.chatTemplatePath ?? null
            };
            console.log('Server started with PID:', pid);

            // Start health monitoring
            this.startHealthMonitoring();
        } catch (err) {
            this.error = err instanceof Error ? err.message : String(err);
            this.isRunning = false;
            console.error('Failed to start server:', err);
        } finally {
            this.isStarting = false;
        }
    }

    async stopServer() {
        try {
            this.error = null;
            await invokeCommand('stop_llama_server');
            this.isRunning = false;
            this.isHealthy = false;
            this.isStarting = false;
            this.currentConfig = null;
            this.serverMetrics = null;
            if (this.healthInterval) {
                clearInterval(this.healthInterval);
                this.healthInterval = null;
            }
            console.log('Server stopped');
        } catch (err) {
            this.error = err instanceof Error ? err.message : String(err);
            console.error('Failed to stop server:', err);
        }
    }

    async checkHealth() {
        try {
            if (!this.isRunning) {
                this.isHealthy = false;
                this.error = null;
                return;
            }
            const detail = await invokeCommand('check_server_health_detail');
            const healthy = Boolean((detail as any)?.healthy);
            this.isHealthy = healthy;
            if (!healthy) {
                const error = (detail as any)?.error;
                const url = (detail as any)?.url;
                this.error = error
                    ? `Healthcheck failed (${url ?? 'unknown'}): ${error}`
                    : 'Server health check failed';
            } else {
                this.error = null;
            }
        } catch (err) {
            this.isHealthy = false;
            this.error = err instanceof Error ? err.message : String(err);
        }
    }

    async checkRunning() {
        try {
            const running = await invokeCommand('is_server_running') as boolean;
            this.isRunning = running;
            if (running) {
                const config = await invokeCommand('get_llama_config') as LlamaCppConfig;
                this.currentConfig = config;
            } else {
                this.currentConfig = null;
            }
        } catch (err) {
            this.isRunning = false;
            this.currentConfig = null;
            console.error('Failed to check if server is running:', err);
        }
    }

    async fetchMetrics() {
        try {
            if (!this.isRunning) return;
            const metrics = await invokeCommand('get_server_metrics');
            this.serverMetrics = metrics as { 
                cpu_usage: number; 
                mem_usage: number; 
                gpu_usage?: number; 
                vram_usage?: number; 
            } | null;
        } catch (err) {
            console.error('Failed to fetch server metrics:', err);
        }
    }

    startHealthMonitoring() {
        if (this.healthInterval) {
            clearInterval(this.healthInterval);
            this.healthInterval = null;
        }
        // Check health and metrics every 2 seconds
        this.healthInterval = setInterval(async () => {
            if (!this.isRunning) {
                if (this.healthInterval) {
                    clearInterval(this.healthInterval);
                    this.healthInterval = null;
                }
                this.serverMetrics = null;
                return;
            }
            await this.checkHealth();
            
            // Only fetch metrics if on models page
            if (window.location.pathname === "/models") {
                await this.fetchMetrics();
            }
        }, 2000);
    }

    async getStatus(): Promise<ServerStatus> {
        await this.checkRunning();
        if (this.isRunning) {
            await this.checkHealth();
        }
        return {
            isRunning: this.isRunning,
            isHealthy: this.isHealthy,
            error: this.error,
            currentConfig: this.currentConfig,
        };
    }
}

export const serverStore = new ServerStore();
