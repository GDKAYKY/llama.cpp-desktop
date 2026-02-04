<script lang="ts">
  import ModelSelector from "$components/layout/ModelSelector.svelte";
  import { X, Square } from "lucide-svelte";
  import { modelsStore } from "$lib/stores/models.svelte";
  import { serverStore } from "$lib/stores/server.svelte";
  import { onMount } from "svelte";

  onMount(async () => {
    // Refresh models if we have a directory configured
    await modelsStore.refresh();
    // Sync server status for Models page indicators
    await serverStore.checkRunning();
    if (serverStore.isRunning) {
      await serverStore.checkHealth();
      serverStore.startHealthMonitoring();
    }
  });

  function handleModelSelected(event) {
    // The ModelSelector already updates the store, but we can log it here if needed
    console.log(
      "Model selection processed in store:",
      modelsStore.selectedModel,
    );
  }
</script>

<div class="flex h-full w-full flex-col bg-background text-foreground">
  <div class="grow overflow-y-auto">
    <ModelSelector on:modelSelected={handleModelSelected} />
  </div>
</div>
