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

  function formatSize(bytes: number) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  function getTotalSize(model: Model) {
    return model.manifest.layers.reduce((acc, layer) => acc + layer.size, 0);
  }

  function getModelMetadata(version: string) {
    // Look for patterns like 7b, 8b, 70b
    const paramMatch = version.match(/\b(\d+\.?\d*b)\b/i);
    // Look for quantization patterns
    const quantMatch = version.match(/(q\d+_\w+)|(q\d+)|(fp16)|(bf16)|(f16)/i);

    return {
      params: paramMatch ? paramMatch[0].toUpperCase() : null,
      quant: quantMatch ? quantMatch[0].toUpperCase() : null,
    };
  }

  function getShortDigest(digest: string) {
    if (!digest) return "";
    const parts = digest.split(":");
    const hash = parts[1] || parts[0];
    return hash.substring(0, 12);
  }

  function isModelRunning(model: Model) {
    return (
      serverStore.isRunning &&
      serverStore.currentConfig?.model_path === model.model_file_path
    );
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
          <div
            class={cn(
              "relative flex cursor-pointer flex-col gap-4 rounded-xl border-1 p-5 transition-all",
              modelsStore.selectedModel?.full_identifier ===
                model.full_identifier
                ? "border-primary bg-primary/5 shadow-md"
                : "border-border bg-white/2 hover:border-white/20 hover:bg-white/5",
            )}
            onclick={() => handleSelectModel(model)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === "Enter" && handleSelectModel(model)}
          >
            <div
              class="flex items-center justify-between gap-2 border-b border-border pb-3"
            >
              <div class="flex min-w-0 flex-col">
                <h4 class="truncate font-semibold text-foreground">
                  {model.name}
                </h4>
                <div class="flex items-center gap-2">
                  <span
                    class="shrink-0 rounded bg-primary/20 px-2 py-0.5 text-[10px] font-bold text-primary uppercase"
                  >
                    {model.version}
                  </span>
                </div>
              </div>

              <div class="relative flex items-center gap-1">
                <button
                  class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-white/10 hover:text-foreground"
                  onclick={(e) => toggleDropdown(model.full_identifier, e)}
                >
                  <MoreVertical size={16} />
                </button>

                {#if activeDropdown === model.full_identifier}
                  <div
                    class="absolute right-0 top-8 z-50 w-48 overflow-hidden rounded-lg border border-border bg-secondary shadow-xl"
                  >
                    <button
                      class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs hover:bg-white/5"
                      onclick={(e) => handleAction("copy-path", model, e)}
                    >
                      <Copy size={14} />
                      Copy File Path
                    </button>
                    <button
                      class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs hover:bg-white/5"
                      onclick={(e) => handleAction("view-manifest", model, e)}
                    >
                      <FileText size={14} />
                      View Manifest
                    </button>
                  </div>
                {/if}
              </div>
            </div>

            {#if isModelRunning(model)}
              <ModelUsageGraph
                vramUsage={serverStore.serverMetrics?.vram_usage || 0}
                gpuUsage={serverStore.serverMetrics?.gpu_usage || 0}
              />
            {/if}

            <div class="grid grid-cols-2 gap-2">
              {#each Object.entries(getModelMetadata(model.version)) as [key, value]}
                {#if value}
                  <div
                    class="flex flex-col rounded-md bg-white/5 p-1.5 border border-white/5"
                  >
                    <span
                      class="text-[9px] uppercase text-muted-foreground font-bold tracking-wider"
                      >{key === "params" ? "Parameters" : "Quant"}</span
                    >
                    <span
                      class="font-mono text-[10px] font-bold text-foreground/90"
                      >{value}</span
                    >
                  </div>
                {/if}
              {/each}
              <div
                class="flex flex-col rounded-md bg-white/5 p-1.5 border border-white/5"
              >
                <span
                  class="text-[9px] uppercase text-muted-foreground font-bold tracking-wider"
                  >Total Size</span
                >
                <span class="font-mono text-[10px] font-bold text-foreground/90"
                  >{formatSize(getTotalSize(model))}</span
                >
              </div>
              <div
                class="flex flex-col rounded-md bg-white/5 p-1.5 border border-white/5"
              >
                <span
                  class="text-[9px] uppercase text-muted-foreground font-bold tracking-wider"
                  >Digest</span
                >
                <span class="font-mono text-[10px] font-bold text-foreground/90"
                  >{getShortDigest(model.manifest.config.digest)}</span
                >
              </div>
            </div>

            <div class="space-y-1 pt-1">
              <div class="flex items-center justify-between text-[10px]">
                <span class="text-muted-foreground">Provider</span>
                <span class="font-medium text-foreground/80"
                  >{model.provider}</span
                >
              </div>
              <div class="flex items-center justify-between text-[10px]">
                <span class="text-muted-foreground">Library</span>
                <span class="font-medium text-foreground/80"
                  >{model.library}</span
                >
              </div>
              <div class="flex items-center justify-between text-[10px]">
                <span class="text-muted-foreground">Layers</span>
                <span class="font-medium text-foreground/80"
                  >{model.manifest.layers.length} files</span
                >
              </div>
            </div>

            {#if !model.model_file_path}
              <div
                class="mt-2 flex items-center gap-2 text-[10px] font-medium text-orange-400"
              >
                <AlertTriangle size={12} />
                Model file not found
              </div>
            {/if}

            {#if modelsStore.selectedModel?.full_identifier === model.full_identifier}
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
</div>
