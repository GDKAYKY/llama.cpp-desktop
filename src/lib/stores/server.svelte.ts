import { invokeCommand } from '$lib/ipc';

export interface ServerStatus {
    isRunning: boolean;
    isHealthy: boolean;
    error: string | null;
}

class ServerStore {
    isRunning = $state(false);
    isHealthy = $state(false);
    error = $state<string | null>(null);
    isChecking = $state(false);

    async startServer(binaryPath: string, modelPath: string, port: number = 8000, ctxSize: number = 4096, nGpuLayers: number = 33) {
        try {
            this.error = null;
            const pid = await invokeCommand('start_llama_server', {
                binaryPath: binaryPath,
                modelPath: modelPath,
                port,
                ctxSize,
                nGpuLayers,
            });
            this.isRunning = true;
            console.log('Server started with PID:', pid);

            // Start health monitoring
            this.startHealthMonitoring();
        } catch (err) {
            this.error = err instanceof Error ? err.message : String(err);
            this.isRunning = false;
            console.error('Failed to start server:', err);
        }
    }

    async stopServer() {
        try {
            this.error = null;
            await invokeCommand('stop_llama_server');
            this.isRunning = false;
            this.isHealthy = false;
            console.log('Server stopped');
        } catch (err) {
            this.error = err instanceof Error ? err.message : String(err);
            console.error('Failed to stop server:', err);
        }
    }

    async checkHealth() {
        try {
            const healthy = await invokeCommand('check_server_health');
            this.isHealthy = healthy as boolean;
            if (!healthy) {
                this.error = 'Server health check failed';
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
            const running = await invokeCommand('is_server_running');
            this.isRunning = running as boolean;
        } catch (err) {
            this.isRunning = false;
            console.error('Failed to check if server is running:', err);
        }
    }

    startHealthMonitoring() {
        // Check health every 5 seconds
        const interval = setInterval(async () => {
            if (!this.isRunning) {
                clearInterval(interval);
                return;
            }
            await this.checkHealth();
        }, 5000);
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
        };
    }
}

export const serverStore = new ServerStore();
