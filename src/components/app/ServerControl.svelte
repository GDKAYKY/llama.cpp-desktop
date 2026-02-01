<script lang="ts">
  import { serverStore } from "$lib/stores/server.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { Power, AlertCircle, CheckCircle } from "lucide-svelte";

  let binary_directory = $state("");
  let models_directory = $state("");
  let port = $state(8000);
  let isLoading = $state(false);

  async function handleStart() {
    const configBinaryPath = settingsStore.settings.llamaDirectory;
    const configModelPath = settingsStore.settings.modelsDirectory;

    if (!configBinaryPath) {
      serverStore.error =
        "Please configure llama server binary path in settings";
      return;
    }

    if (!configModelPath) {
      serverStore.error = "Please configure models directory in settings";
      return;
    }

    isLoading = true;
    try {
      await serverStore.startServer(configBinaryPath, configModelPath, port);
    } finally {
      isLoading = false;
    }
  }

  async function handleStop() {
    isLoading = true;
    try {
      await serverStore.stopServer();
    } finally {
      isLoading = false;
    }
  }

  async function handleCheckHealth() {
    await serverStore.checkHealth();
  }
</script>

<div class="server-control">
  <div class="header">
    <h3>Llama Server Control</h3>
    <div
      class="status-indicator"
      class:running={serverStore.isRunning}
      class:healthy={serverStore.isHealthy}
    >
      {#if serverStore.isRunning}
        {#if serverStore.isHealthy}
          <CheckCircle size={20} class="text-green-500" />
          <span>Running & Healthy</span>
        {:else}
          <AlertCircle size={20} class="text-yellow-500" />
          <span>Running (Unhealthy)</span>
        {/if}
      {:else}
        <Power size={20} class="text-gray-500" />
        <span>Stopped</span>
      {/if}
    </div>
  </div>

  <div class="config-section">
    <div class="form-group">
      <label for="port">Port</label>
      <input
        id="port"
        type="number"
        bind:value={port}
        min="1024"
        max="65535"
        disabled={serverStore.isRunning}
      />
    </div>
    <div class="info-text">
      <p>Binary path and models directory are configured in Settings</p>
    </div>
  </div>

  <div class="button-group">
    <button
      onclick={handleStart}
      disabled={serverStore.isRunning || isLoading}
      class="btn-start"
    >
      {isLoading ? "Starting..." : "Start Server"}
    </button>

    <button
      onclick={handleStop}
      disabled={!serverStore.isRunning || isLoading}
      class="btn-stop"
    >
      {isLoading ? "Stopping..." : "Stop Server"}
    </button>

    <button
      onclick={handleCheckHealth}
      disabled={!serverStore.isRunning || isLoading}
      class="btn-health"
    >
      Check Health
    </button>
  </div>

  {#if serverStore.error}
    <div class="error-message">
      {serverStore.error}
    </div>
  {/if}
</div>

<style>
  .server-control {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px;
    background: var(--secondary);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    padding: 4px 12px;
    border-radius: 4px;
    background: var(--input);
  }

  .status-indicator.running.healthy {
    background: rgba(34, 197, 94, 0.1);
    color: rgb(34, 197, 94);
  }

  .config-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .form-group label {
    font-size: 12px;
    font-weight: 500;
    color: var(--muted-foreground);
  }

  .form-group input {
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--input);
    color: var(--foreground);
    font-size: 14px;
  }

  .form-group input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .info-text {
    padding: 8px 12px;
    border-radius: 4px;
    background: rgba(59, 130, 246, 0.1);
    color: rgb(59, 130, 246);
    font-size: 12px;
    border: 1px solid rgba(59, 130, 246, 0.3);
  }

  .info-text p {
    margin: 0;
  }

  .button-group {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }

  button {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--input);
    color: var(--foreground);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  button:hover:not(:disabled) {
    background: var(--primary);
    color: var(--primary-foreground);
    border-color: var(--primary);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-start {
    background: rgba(34, 197, 94, 0.1);
    border-color: rgb(34, 197, 94);
    color: rgb(34, 197, 94);
  }

  .btn-start:hover:not(:disabled) {
    background: rgb(34, 197, 94);
    color: white;
  }

  .btn-stop {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgb(239, 68, 68);
    color: rgb(239, 68, 68);
  }

  .btn-stop:hover:not(:disabled) {
    background: rgb(239, 68, 68);
    color: white;
  }

  .error-message {
    padding: 8px 12px;
    border-radius: 4px;
    background: rgba(239, 68, 68, 0.1);
    color: rgb(239, 68, 68);
    font-size: 12px;
  }
</style>
