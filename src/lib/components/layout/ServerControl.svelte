<script lang="ts">
  import { serverStore } from "$lib/stores/server.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { Power, CircleAlert, CircleCheck } from "lucide-svelte";

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
      await serverStore.startServer(
        configBinaryPath,
        configModelPath,
        settingsStore.settings.serverPort,
      );
    } catch (error) {
      serverStore.error = `Failed to start server: ${error}`;
    } finally {
      isLoading = false;
    }
  }

  async function handleStop() {
    isLoading = true;
    try {
      await serverStore.stopServer();
    } catch (error) {
      serverStore.error = `Failed to stop server: ${error}`;
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="server-control">
  <div class="header">
    <h3>Server Control</h3>
    <div class="status-indicator">
      {#if serverStore.isRunning}
        <CircleCheck size={16} class="text-green-500" />
        <span class="text-green-500">Running</span>
      {:else}
        <CircleAlert size={16} class="text-gray-500" />
        <span class="text-gray-500">Stopped</span>
      {/if}
    </div>
  </div>

  {#if serverStore.error}
    <div class="error-message">
      {serverStore.error}
    </div>
  {/if}

  <div class="controls">
    <button
      onclick={handleStart}
      disabled={serverStore.isRunning || isLoading}
      class="btn btn-primary"
    >
      <Power size={16} />
      Start Server
    </button>
    <button
      onclick={handleStop}
      disabled={!serverStore.isRunning || isLoading}
      class="btn btn-secondary"
    >
      <Power size={16} />
      Stop Server
    </button>
  </div>

  {#if serverStore.isRunning}
    <div class="server-info">
      <p>Port: {settingsStore.settings.serverPort}</p>
      <p>Binary: {settingsStore.settings.llamaDirectory}</p>
      <p>Models: {settingsStore.settings.modelsDirectory}</p>
    </div>
  {/if}
</div>

<style>
  .server-control {
    padding: 1rem;
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    background: var(--bg-secondary);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .error-message {
    padding: 0.75rem;
    margin-bottom: 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgb(239, 68, 68);
    border-radius: 0.375rem;
    color: rgb(239, 68, 68);
    font-size: 0.875rem;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    font-size: 0.875rem;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--primary-color);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--primary-color-dark);
  }

  .btn-secondary {
    background: var(--secondary-color);
    color: white;
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--secondary-color-dark);
  }

  .server-info {
    padding: 0.75rem;
    background: var(--bg-tertiary);
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  .server-info p {
    margin: 0.25rem 0;
    word-break: break-all;
  }
</style>
