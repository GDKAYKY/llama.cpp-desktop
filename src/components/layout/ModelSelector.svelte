<script lang="ts">
  import { modelsStore, type Model } from "$lib/stores/models.svelte";
  import { createEventDispatcher } from "svelte";
  import { cn } from "$shared/cn.js";
  import {
    FolderOpen,
    Scan,
    Check,
    TriangleAlert,
    Box,
    Bot,
    Square,
  } from "lucide-svelte";
  import { serverStore } from "$lib/stores/server.svelte";
  import { chatStore } from "$lib/stores/chat.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import ModelUsageGraph from "$components/chat/ModelUsageGraph.svelte";
  import ModelLogo from "./ModelLogo.svelte";
  import { Play, Rocket } from "lucide-svelte";

  import ModelCard from "./ModelCard.svelte";
  import Modal from "$components/ui/Modal.svelte";

  const dispatch = createEventDispatcher();
  let activeDropdown = $state<string | null>(null);
  let viewingManifest = $state<Model | null>(null);
  let showCopySuccess = $state(false);

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
        showCopySuccess = true;
        setTimeout(() => (showCopySuccess = false), 2000);
      }
    } else if (action === "view-manifest") {
      viewingManifest = model;
    } else if (action === "start-model") {
      handleLaunchModel(model, e);
    } else if (action === "stop-model") {
      serverStore.stopServer();
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

    modelsStore.successMessage = `Ready to chat with ${modelsStore.selectedModel.name}`;
  }

  async function handleLaunchModel(model: Model, e: MouseEvent) {
    e.stopPropagation();
    if (!model.model_file_path) return;

    const llamaDirectory = settingsStore.settings.llamaDirectory;
    if (!llamaDirectory) {
      modelsStore.error =
        "Llama Server path not configured. Please go to Settings.";
      return;
    }

    await serverStore.startServer(llamaDirectory, model.model_file_path);
  }
</script>

<svelte:window onclick={() => (activeDropdown = null)} />

<div class="mx-auto max-w-7xl p-6 text-foreground">
  <div
    class="mb-8 flex flex-col md:flex-row md:items-center md:justify-between gap-4"
  >
    <div>
      <div class="flex items-center gap-3">
        <div
          class="flex h-10 w-10 items-center justify-center rounded-xl bg-purple-500/10 text-purple-500"
        >
          <Box size={20} />
        </div>
        <div>
          <h2 class="text-2xl font-semibold">Model Library</h2>
          <p class="text-sm text-muted-foreground mt-1">
            Manage and select models for inference
          </p>
        </div>
      </div>
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
      class="mb-6 rounded-lg border border-border bg-white/0.02 p-3 font-mono text-sm text-muted-foreground"
    >
      <span class="mr-2 font-bold text-foreground">Path:</span
      >{modelsStore.modelsRoot}
    </div>
  {/if}

  {#if modelsStore.successMessage}
    <div
      class="mb-6 flex animate-in fade-in slide-in-from-top-2 duration-300 items-center justify-between gap-3 rounded-xl border border-primary/30 bg-primary/10 px-4 py-3 text-sm text-primary"
    >
      <div class="flex items-center gap-3">
        <div
          class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/20"
        >
          <ModelLogo name={modelsStore.selectedModel?.name || ""} size={18} />
        </div>
        <div>
          <p class="font-semibold text-foreground">Model selected</p>
          <p class="text-xs text-muted-foreground">
            {modelsStore.successMessage}
          </p>
        </div>
      </div>
      <button
        onclick={() => modelsStore.clearMessages()}
        class="text-muted-foreground hover:text-foreground transition-colors"
      >
        <Check size={18} />
      </button>
    </div>
  {/if}

  {#if modelsStore.error}
    <div
      class="mb-6 flex animate-in fade-in slide-in-from-top-2 duration-300 items-center gap-3 rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-400"
    >
      <TriangleAlert size={18} />
      <div class="flex-1">
        <p class="font-semibold text-red-100">Error</p>
        <p class="text-xs opacity-80">{modelsStore.error}</p>
      </div>
      <button
        onclick={() => modelsStore.clearMessages()}
        class="opacity-60 hover:opacity-100 transition-opacity"
      >
        <Square size={14} />
      </button>
    </div>
  {/if}

  {#if modelsStore.models.length > 0}
    <div class="space-y-6">
      <div
        class="flex items-center justify-between border-b border-border pb-4"
      >
        <h3 class="flex items-center gap-2 text-lg font-medium">
          <Bot size={20} class="text-muted-foreground" />
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

  {#if showCopySuccess}
    <div
      class="fixed bottom-8 left-1/2 z-100 -translate-x-1/2 animate-in fade-in slide-in-from-bottom-4 duration-300 pointer-events-none"
    >
      <div
        class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-lg"
      >
        Path copied to clipboard!
      </div>
    </div>
  {/if}

  <Modal
    title={`Manifest: ${viewingManifest?.name}:${viewingManifest?.version}`}
    isOpen={!!viewingManifest}
    onClose={() => (viewingManifest = null)}
  >
    {#if viewingManifest}
      <div class="space-y-4">
        <div class="rounded-lg bg-black/20 p-4">
          <pre
            class="overflow-auto text-[11px] font-mono leading-relaxed text-foreground/80">
            {JSON.stringify(viewingManifest.manifest, null, 2)}
          </pre>
        </div>

        <div class="space-y-2">
          <h4
            class="text-sm font-semibold text-muted-foreground uppercase tracking-wider"
          >
            Storage Info
          </h4>
          <div class="grid grid-cols-2 gap-4 text-xs">
            <div class="rounded-md bg-white/5 p-2">
              <span class="block text-muted-foreground">Provider</span>
              <span class="font-mono">{viewingManifest.provider}</span>
            </div>
            <div class="rounded-md bg-white/5 p-2">
              <span class="block text-muted-foreground">Full ID</span>
              <span class="font-mono">{viewingManifest.full_identifier}</span>
            </div>
          </div>
        </div>

        {#if viewingManifest.model_file_path}
          <div class="space-y-2">
            <h4
              class="text-sm font-semibold text-muted-foreground uppercase tracking-wider"
            >
              Physical Path
            </h4>
            <div
              class="rounded-md bg-white/5 p-3 font-mono text-[10px] break-all"
            >
              {viewingManifest.model_file_path}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </Modal>
</div>
