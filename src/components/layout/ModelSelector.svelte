<script lang="ts">
  import {
    selectModelsDirectory,
    scanModelsDirectory,
    saveModelLibrary,
    loadModelLibrary,
  } from "$lib/services/models.js";
  import { createEventDispatcher } from "svelte";
  import { cn } from "$shared/cn.js";
  import {
    FolderOpen,
    Scan,
    Check,
    AlertTriangle,
    Box,
    Activity,
  } from "lucide-svelte";
  import { serverStore } from "$lib/stores/server.svelte";

  type Model = {
    name: string;
    version: string;
    provider: string;
    library: string;
    full_identifier: string;
    manifest: { layers: Array<{ size: number }> };
    model_file_path?: string;
  };

  const dispatch = createEventDispatcher();

  let modelsRoot = $state("");
  let models = $state<Model[]>([]);
  let selectedModel = $state<Model | null>(null);
  let loading = $state(false);
  let error = $state("");
  let libraryPath = $state("");
  let successMessage = $state("");

  async function handleSelectDirectory() {
    try {
      error = "";
      successMessage = "";
      const selected = await selectModelsDirectory();
      if (selected) {
        modelsRoot = selected;
        libraryPath = `${selected}/modelLibrary.json`;
        await loadExistingLibrary();
      }
    } catch (err) {
      error = `Failed to select directory: ${err instanceof Error ? err.message : String(err)}`;
      console.error(err);
    }
  }

  async function loadExistingLibrary() {
    try {
      loading = true;
      const existingModels = await loadModelLibrary(libraryPath);
      if (existingModels.length > 0) {
        models = existingModels;
        successMessage = `Loaded ${existingModels.length} model(s) from library`;
      }
    } catch (err) {
      console.log("No existing library found, will scan directory");
    } finally {
      loading = false;
    }
  }

  async function handleScanDirectory() {
    if (!modelsRoot) {
      error = "Please select a models directory first";
      return;
    }

    try {
      loading = true;
      error = "";
      successMessage = "";
      models = await scanModelsDirectory(modelsRoot);

      if (models.length > 0) {
        await saveModelLibrary(libraryPath, models);
        successMessage = `Found and saved ${models.length} model(s)`;
      } else {
        error = "No models found in the selected directory";
      }
    } catch (err) {
      error = `Failed to scan directory: ${err instanceof Error ? err.message : String(err)}`;
      console.error(err);
    } finally {
      loading = false;
    }
  }

  function handleSelectModel(model: Model) {
    selectedModel = model;
    successMessage = "";
  }

  function handleLoadModel() {
    if (!selectedModel) {
      error = "Please select a model first";
      return;
    }

    // Dispatch event to parent component
    dispatch("modelSelected", {
      model: selectedModel,
    });

    successMessage = `Model "${selectedModel.name}:${selectedModel.version}" is ready to use`;
  }

  function formatSize(bytes: number) {
    const gb = bytes / 1024 ** 3;
    return `${gb.toFixed(2)} GB`;
  }
</script>

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
        disabled={loading}
        class="flex cursor-pointer items-center gap-2 rounded-lg border border-border bg-transparent px-4 py-2 text-sm font-medium transition-colors hover:bg-white/5 disabled:opacity-50"
      >
        <FolderOpen size={18} />
        Select Models Directory
      </button>

      {#if modelsRoot}
        <button
          onclick={handleScanDirectory}
          disabled={loading}
          class="flex cursor-pointer items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-all hover:bg-primary/90 disabled:opacity-50"
        >
          <Scan size={18} class={cn(loading && "animate-spin")} />
          {loading ? "Scanning..." : "Scan for Models"}
        </button>
      {/if}
    </div>
  </div>

  {#if modelsRoot}
    <div
      class="mb-6 rounded-lg border border-border bg-white/[0.02] p-3 font-mono text-sm text-muted-foreground"
    >
      <span class="mr-2 font-bold text-foreground">Path:</span>{modelsRoot}
    </div>
  {/if}

  {#if error}
    <div
      class="mb-6 flex items-center gap-3 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-400"
    >
      <AlertTriangle size={18} />
      {error}
    </div>
  {/if}

  {#if successMessage}
    <div
      class="mb-6 flex items-center gap-3 rounded-lg border border-green-500/30 bg-green-500/10 px-4 py-3 text-sm text-green-400"
    >
      <Check size={18} />
      {successMessage}
    </div>
  {/if}

  {#if models.length > 0}
    <div class="space-y-6">
      <div
        class="flex items-center justify-between border-b border-border pb-4"
      >
        <h3 class="flex items-center gap-2 text-lg font-medium">
          <Box size={20} class="text-muted-foreground" />
          Available Models ({models.length})
        </h3>
      </div>

      <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {#each models as model}
          <div
            class={cn(
              "relative flex cursor-pointer flex-col gap-4 rounded-xl border-2 p-5 transition-all",
              selectedModel?.full_identifier === model.full_identifier
                ? "border-primary bg-primary/5 shadow-md"
                : "border-border bg-white/[0.02] hover:border-white/20 hover:bg-white/[0.05]",
            )}
            onclick={() => handleSelectModel(model)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === "Enter" && handleSelectModel(model)}
          >
            <div
              class="flex items-center justify-between gap-2 border-b border-border pb-3"
            >
              <h4 class="truncate font-semibold text-foreground">
                {model.name}
              </h4>
              <span
                class="shrink-0 rounded bg-primary/20 px-2 py-0.5 text-[10px] font-bold text-primary uppercase"
              >
                {model.version}
              </span>
            </div>

            {#if serverStore.isRunning && serverStore.currentConfig?.model_path === model.model_file_path}
              <div
                class="flex items-center gap-1.5 text-[10px] font-bold text-green-400 uppercase tracking-wider"
              >
                <Activity size={12} class="animate-pulse" />
                Running
              </div>
            {/if}

            <div class="space-y-1 text-xs">
              <div class="flex justify-between">
                <span class="text-muted-foreground">Provider:</span>
                <span class="font-medium">{model.provider}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Library:</span>
                <span class="font-medium">{model.library}</span>
              </div>
              {#if model.manifest.layers[0]}
                <div class="flex justify-between">
                  <span class="text-muted-foreground">Size:</span>
                  <span class="font-medium"
                    >{formatSize(model.manifest.layers[0].size)}</span
                  >
                </div>
              {/if}
            </div>

            {#if !model.model_file_path}
              <div
                class="mt-2 flex items-center gap-2 text-[10px] font-medium text-orange-400"
              >
                <AlertTriangle size={12} />
                Model file not found
              </div>
            {/if}

            {#if selectedModel?.full_identifier === model.full_identifier}
              <div
                class="absolute -right-1.5 -top-1.5 flex h-7 w-7 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg"
              >
                <Check size={16} strokeWidth={3} />
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if selectedModel}
    <div
      class="mt-12 rounded-2xl border border-border bg-secondary p-8 shadow-xl"
    >
      <div
        class="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-border pb-6"
      >
        <div>
          <h3 class="text-xl font-bold">Selected Model</h3>
          <p class="text-sm text-muted-foreground">
            Ready to initialize and run
          </p>
        </div>
        <button
          class="cursor-pointer rounded-xl bg-primary px-8 py-3 font-semibold text-primary-foreground transition-all hover:scale-105 hover:bg-primary/90 shadow-lg shadow-primary/20"
          onclick={handleLoadModel}
        >
          Load This Model
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-x-12 gap-y-4 text-sm">
        <div class="flex justify-between border-b border-border/50 py-2">
          <span class="text-muted-foreground">Name</span>
          <span class="font-medium">{selectedModel.name}</span>
        </div>
        <div class="flex justify-between border-b border-border/50 py-2">
          <span class="text-muted-foreground">Version</span>
          <span class="font-medium">{selectedModel.version}</span>
        </div>
        <div class="flex justify-between border-b border-border/50 py-2">
          <span class="text-muted-foreground">Provider</span>
          <span class="font-medium">{selectedModel.provider}</span>
        </div>
        <div class="flex justify-between border-b border-border/50 py-2">
          <span class="text-muted-foreground">Identifier</span>
          <span class="font-mono text-xs">{selectedModel.full_identifier}</span>
        </div>
        {#if selectedModel.model_file_path}
          <div class="col-span-1 md:col-span-2 flex flex-col gap-2 pt-2">
            <span class="text-muted-foreground">File Path</span>
            <code
              class="block break-all rounded bg-input p-3 text-xs leading-relaxed"
              >{selectedModel.model_file_path}</code
            >
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
