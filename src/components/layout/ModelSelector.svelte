<script lang="ts">
  import { modelsStore, type Model } from "$lib/stores/models.svelte";
  import { createEventDispatcher } from "svelte";
  import { cn } from "$shared/cn.js";
  import {
    FolderOpen,
    Scan,
    Check,
    AlertTriangle,
    Box,
    Activity,
    Square,
    MoreVertical,
    Copy,
    FileText,
  } from "lucide-svelte";
  import { serverStore } from "$lib/stores/server.svelte";
  import { chatStore } from "$lib/stores/chat.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import ModelUsageGraph from "$components/chat/ModelUsageGraph.svelte";
  import { Play } from "lucide-svelte";

  import ModelCard from "./ModelCard.svelte";

  const dispatch = createEventDispatcher();
  let activeDropdown = $state<string | null>(null);

  function toggleDropdown(id: string, e: MouseEvent) {
    e.stopPropagation();
    activeDropdown = activeDropdown === id ? null : id;
  }

  function handleAction(action: string, model: Model, e: MouseEvent) {
    e.stopPropagation();
    activeDropdown = null;

    if (action === "copy-path") {
      if (model.model_file_path) {
        navigator.clipboard.writeText(model.model_file_path);
        // You might want to add a toast here, but I'll stick to the core request
      }
    } else if (action === "view-manifest") {
      console.log("Viewing manifest for:", model.name, model.manifest);
      // Logic for viewing manifest could be a modal or another page,
      // but for now, we're just adding the dropdown structure.
    }
  }

  async function handleSelectDirectory() {
    await modelsStore.selectDirectory();
  }

  async function handleScanDirectory() {
    await modelsStore.scan();
  }

  function handleSelectModel(model: Model) {
    modelsStore.selectModel(model);
  }

  function handleLoadModel() {
    if (!modelsStore.selectedModel) {
      modelsStore.error = "Please select a model first";
      return;
    }

    // Dispatch event to parent component
    dispatch("modelSelected", {
      model: modelsStore.selectedModel,
    });

    modelsStore.successMessage = `Model "${modelsStore.selectedModel.name}:${modelsStore.selectedModel.version}" is ready to use`;
  }

  async function handleLaunchModel(model: Model, e: MouseEvent) {
    e.stopPropagation();
    if (!model.model_file_path) return;

    await serverStore.startServer(
      settingsStore.settings.llamaDirectory,
      model.model_file_path,
    );
  }
</script>

<svelte:window onclick={() => (activeDropdown = null)} />

<div class="mx-auto max-w-7xl p-6 text-foreground">
  <div
    class="mb-8 flex flex-col md:flex-row md:items-center md:justify-between gap-4"
  >
    <div>
      <h2 class="text-2xl font-semibold">Model Library</h2>
      <p class="text-sm text-muted-foreground mt-1">
        Manage and select models for inference
      </p>
    </div>

    <div class="flex flex-wrap gap-2">
      <button
        onclick={handleSelectDirectory}
        disabled={modelsStore.isLoading}
        class="flex cursor-pointer items-center gap-2 rounded-lg border border-border bg-transparent px-4 py-2 text-sm font-medium transition-colors hover:bg-white/5 disabled:opacity-50"
      >
        <FolderOpen size={18} />
        Select Models Directory
      </button>

      {#if modelsStore.modelsRoot}
        <button
          onclick={handleScanDirectory}
          disabled={modelsStore.isLoading}
          class="flex cursor-pointer items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-all hover:bg-primary/90 disabled:opacity-50"
        >
          <Scan size={18} class={cn(modelsStore.isLoading && "animate-spin")} />
          {modelsStore.isLoading ? "Scanning..." : "Scan for Models"}
        </button>
      {/if}
    </div>
  </div>

  {#if modelsStore.modelsRoot}
    <div
      class="mb-6 rounded-lg border border-border bg-white/[0.02] p-3 font-mono text-sm text-muted-foreground"
    >
      <span class="mr-2 font-bold text-foreground">Path:</span
      >{modelsStore.modelsRoot}
    </div>
  {/if}

  {#if modelsStore.error}
    <div
      class="mb-6 flex items-center gap-3 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-400"
    >
      <AlertTriangle size={18} />
      {modelsStore.error}
    </div>
  {/if}

  {#if modelsStore.successMessage}
    <div
      class="mb-6 flex items-center gap-3 rounded-lg border border-green-500/30 bg-green-500/10 px-4 py-3 text-sm text-green-400"
    >
      <Check size={18} />
      {modelsStore.successMessage}
    </div>
  {/if}

  {#if modelsStore.models.length > 0}
    <div class="space-y-6">
      <div
        class="flex items-center justify-between border-b border-border pb-4"
      >
        <h3 class="flex items-center gap-2 text-lg font-medium">
          <Box size={20} class="text-muted-foreground" />
          Available Models ({modelsStore.models.length})
        </h3>
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {#each modelsStore.models as model}
          <ModelCard
            {model}
            isSelected={modelsStore.selectedModel?.full_identifier ===
              model.full_identifier}
            {activeDropdown}
            onSelect={handleSelectModel}
            onToggleDropdown={toggleDropdown}
            onAction={handleAction}
          />
        {/each}
      </div>
    </div>
  {/if}
</div>
