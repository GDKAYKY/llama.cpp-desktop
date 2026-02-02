<script lang="ts">
  import { type Model } from "$lib/stores/models.svelte";
  import { cn } from "$shared/cn.js";
  import {
    Check,
    AlertTriangle,
    MoreVertical,
    Copy,
    FileText,
    Box,
    Library,
    Layers,
    Fingerprint,
  } from "lucide-svelte";
  import { serverStore } from "$lib/stores/server.svelte";
  import ModelUsageGraph from "$components/chat/ModelUsageGraph.svelte";
  import ModelLogo from "./ModelLogo.svelte";

  interface Props {
    model: Model;
    isSelected: boolean;
    activeDropdown: string | null;
    onSelect: (model: Model) => void;
    onToggleDropdown: (id: string, e: MouseEvent) => void;
    onAction: (action: string, model: Model, e: MouseEvent) => void;
  }

  let {
    model,
    isSelected,
    activeDropdown,
    onSelect,
    onToggleDropdown,
    onAction,
  }: Props = $props();

  function isModelRunning(model: Model) {
    return (
      serverStore.isRunning &&
      serverStore.currentConfig?.model_path === model.model_file_path
    );
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
    const paramMatch = version.match(/\b(\d+\.?\d*b)\b/i);
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
</script>

<div
  class={cn(
    "relative flex h-[400px] cursor-pointer flex-col gap-2 rounded-xl border p-4 transition-all",
    isSelected
      ? "border-primary bg-primary/5 active:scale-[0.98] active:duration-0"
      : isModelRunning(model)
        ? "border-[#416b418f] bg-white/2 hover:border-[#347034] hover:bg-white/5 active:!border-[#8fff94] active:scale-[0.98] active:duration-0"
        : "border-border bg-white/2 hover:border-white/20 hover:bg-white/5 active:scale-[0.98] active:duration-0",
  )}
  onclick={() => onSelect(model)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && onSelect(model)}
>
  <div
    class="flex items-center justify-between gap-2 border-b border-border pb-2"
  >
    <div class="flex min-w-0 items-center gap-2">
      <div
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-white/5 p-1.5 text-muted-foreground/50"
      >
        <ModelLogo name={model.name} size={18} />
      </div>
      <div class="flex min-w-0 flex-col">
        <h4 class="truncate font-semibold text-foreground">
          {model.name}
        </h4>
        <span class="text-[10px] text-muted-foreground">
          {model.version}
        </span>
      </div>
    </div>

    <div class="relative flex shrink-0 items-center gap-1">
      <button
        class="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-white/10 hover:text-foreground"
        onclick={(e) => onToggleDropdown(model.full_identifier, e)}
      >
        <MoreVertical size={16} />
      </button>

      {#if activeDropdown === model.full_identifier}
        <div
          class="absolute right-0 top-8 z-50 w-48 overflow-hidden rounded-lg border border-border bg-secondary shadow-xl"
        >
          <button
            class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs hover:bg-white/5"
            onclick={(e) => onAction("copy-path", model, e)}
          >
            <Copy size={14} />
            Copy File Path
          </button>
          <button
            class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs hover:bg-white/5"
            onclick={(e) => onAction("view-manifest", model, e)}
          >
            <FileText size={14} />
            View Manifest
          </button>
        </div>
      {/if}
    </div>
  </div>

  <div
    class={!isModelRunning(model)
      ? "invisible pointer-events-none select-none"
      : ""}
  >
    <ModelUsageGraph
      isRunning={isModelRunning(model)}
      vramUsage={isModelRunning(model)
        ? serverStore.serverMetrics?.vram_usage || 0
        : 0}
      gpuUsage={isModelRunning(model)
        ? serverStore.serverMetrics?.gpu_usage || 0
        : 0}
    />
  </div>

  <div class="rounded-lg bg-white/5 p-2.5">
    <div class="grid grid-cols-3 gap-2">
      {#each Object.entries(getModelMetadata(model.version)) as [key, value]}
        {#if value}
          <div class="flex flex-col">
            <span
              class="text-[9px] font-bold tracking-wider text-muted-foreground uppercase"
              >{key === "params" ? "Parameters" : "Quant"}</span
            >
            <span class="font-mono text-[10px] font-bold text-foreground/90"
              >{value}</span
            >
          </div>
        {/if}
      {/each}
      <div class="flex flex-col">
        <span
          class="text-[9px] font-bold tracking-wider text-muted-foreground uppercase"
          >Total Size</span
        >
        <span class="font-mono text-[10px] font-bold text-foreground/90"
          >{formatSize(getTotalSize(model))}</span
        >
      </div>
    </div>
  </div>

  <div class="mt-auto space-y-1 pt-2">
    <div class="flex items-center justify-between text-[10px]">
      <div class="flex items-center gap-1.5 text-muted-foreground">
        <Box size={10} />
        <span>Provider</span>
      </div>
      <span class="font-medium text-foreground/80">{model.provider}</span>
    </div>
    <div class="flex items-center justify-between text-[10px]">
      <div class="flex items-center gap-1.5 text-muted-foreground">
        <Library size={10} />
        <span>Library</span>
      </div>
      <span class="font-medium text-foreground/80">{model.library}</span>
    </div>
    <div class="flex items-center justify-between text-[10px]">
      <div class="flex items-center gap-1.5 text-muted-foreground">
        <Layers size={10} />
        <span>Layers</span>
      </div>
      <span class="font-medium text-foreground/80"
        >{model.manifest.layers.length} files</span
      >
    </div>
    <div class="flex items-center justify-between text-[10px]">
      <div class="flex items-center gap-1.5 text-muted-foreground">
        <Fingerprint size={10} />
        <span>Digest</span>
      </div>
      <span class="font-mono font-medium text-foreground/80"
        >{getShortDigest(model.manifest.config.digest)}</span
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

  {#if isSelected}
    <div
      class="absolute -right-1.5 -top-1.5 flex h-7 w-7 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg"
    >
      <Check size={16} strokeWidth={3} />
    </div>
  {/if}
</div>
