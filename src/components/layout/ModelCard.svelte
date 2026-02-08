<script lang="ts">
  import { type Model } from "$lib/stores/models.svelte";
  import { cn } from "$shared/cn.js";
  import {
    Check,
    TriangleAlert,
    MoreVertical,
    Copy,
    FileText,
    Box,
    Library,
    Layers,
    Fingerprint,
    Play,
    Square,
    Binary,
    Tag,
    Info,
    Activity,
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

  const BORDER_GAP_LEFT = 5;
  const BORDER_GAP_RIGHT = 16;

  let {
    model,
    isSelected,
    activeDropdown,
    onSelect,
    onToggleDropdown,
    onAction,
  }: Props = $props();

  let statusWidth = $state(0);

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

  function getModelMetadata(model: Model) {
    const combined =
      `${model.name} ${model.version} ${model.model_file_path || ""} ${model.library} ${model.full_identifier}`.toLowerCase();

    // Match 8b, 70B, etc. with common separators
    const paramMatch = combined.match(
      /(?:^|[\-\.\s_:])(\d+\.?\d*b)(?:$|[\-\.\s_:])/i,
    );
    // Support Q4_K_M, Q5_0, IQ4_XS, fp16, etc.
    const quantMatch = combined.match(
      /(iq\d+_[a-z0-9_]+)|(q\d+_[a-z0-9_]+)|(q\d+_[a-z0-9])|(q\d+)|(fp16)|(bf16)|(f16)|(f32)/i,
    );
    const formatMatch = combined.match(
      /\b(gguf|safetensors|awq|gptq|exl2|onnx|mlx|ggml)\b/i,
    );
    const typeMatch = combined.match(
      /\b(instruct|chat|coder|vision|rl|base|moe|uncensored|distilled)\b/i,
    );

    let format =
      formatMatch?.[0].toUpperCase() ||
      (combined.includes("gguf") ? "GGUF" : null);

    // Advanced heuristics for models without explicit extensions
    if (!format) {
      const isOllama = model.provider.toLowerCase() === "ollama";
      const hasQuant = !!quantMatch;
      const nameLower = model.name.toLowerCase();

      // If it's Ollama, has quantization patterns, or llama library -> it's GGUF
      if (isOllama || hasQuant || model.library === "llama") {
        format = "GGUF";
      } else if (
        /\b(llama|mistral|qwen|phi|gemma|deepseek|yi|stable|starcoder|command|internlm|grok|smollm|nemotron|granite)\b/i.test(
          combined,
        )
      ) {
        format = "GGUF";
      }
    }

    return {
      params:
        paramMatch?.[1].toUpperCase() ||
        combined.match(/\b\d+\.?\d*b\b/i)?.[0].toUpperCase() ||
        combined.match(/\d+\.?\d*b/i)?.[0].toUpperCase() ||
        null,
      quant: quantMatch ? quantMatch[0].toUpperCase() : null,
      format: format,
      type: typeMatch ? typeMatch[0].toUpperCase() : null,
    };
  }

  const metadata = $derived(getModelMetadata(model));

  function getShortDigest(digest: string) {
    if (!digest) return "";
    const parts = digest.split(":");
    const hash = parts[1] || parts[0];
    return hash.substring(0, 12);
  }

  const currentStatus = $derived.by(() => {
    if (!model.model_file_path) {
      return {
        label: "Model file not found",
        icon: TriangleAlert,
        color: "text-orange-400",
        badgeColor: "bg-orange-500",
      };
    }

    if (isModelRunning(model)) {
      if (!serverStore.isHealthy && serverStore.error) {
        return {
          label: serverStore.error,
          icon: TriangleAlert,
          color: "text-orange-400",
          badgeColor: "bg-orange-500",
        };
      }
      return {
        label: "Model is running",
        icon: Activity,
        color: "text-green-400",
        badgeColor: "bg-green-500",
      };
    }

    return null;
  });

  const borderColors = $derived.by(() => {
    if (currentStatus?.icon === TriangleAlert) {
      if (isSelected) return "border-orange-300 group-hover:border-orange-100";
      return "border-orange-500 group-hover:border-orange-400";
    }
    if (isSelected) return "border-primary";
    if (isModelRunning(model))
      return "border-[#416b418f] group-hover:border-[#347034] group-active:!border-[#8fff94]";
    return "border-border group-hover:border-white/20";
  });
</script>

<div
  class={cn(
    "group relative flex h-[400px] cursor-pointer flex-col gap-2 rounded-xl p-4 transition-all",
    isSelected
      ? "bg-primary/5 active:scale-[0.98] active:duration-0"
      : isModelRunning(model)
        ? "bg-white/2 hover:bg-white/5 active:scale-[0.98] active:duration-0"
        : "bg-white/2 hover:bg-white/5 active:scale-[0.98] active:duration-0",
  )}
  onclick={() => onSelect(model)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && onSelect(model)}
>
  <div
    class={cn(
      "pointer-events-none absolute inset-0 rounded-xl border transition-all",
      borderColors,
    )}
    style={currentStatus && statusWidth > 0
      ? `
      -webkit-mask-image: linear-gradient(to bottom, black calc(100% - 1px), transparent calc(100% - 1px)), linear-gradient(to right, black ${BORDER_GAP_LEFT}px, transparent ${BORDER_GAP_LEFT}px, transparent calc(16px + ${statusWidth}px + 7px), black calc(16px + ${statusWidth}px + 7px));
      -webkit-mask-composite: source-over;
      -webkit-mask-size: 100% 100%, 100% 1px;
      -webkit-mask-position: 0 0, 0 100%;
      -webkit-mask-repeat: no-repeat;
      mask-image: linear-gradient(to bottom, black calc(100% - 1px), transparent calc(100% - 1px)), linear-gradient(to right, black ${BORDER_GAP_LEFT}px, transparent ${BORDER_GAP_LEFT}px, transparent calc(16px + ${statusWidth}px + 7px), black calc(16px + ${statusWidth}px + 7px));
      mask-composite: add;
      mask-size: 100% 100%, 100% 1px;
      mask-position: 0 0, 0 100%;
      mask-repeat: no-repeat;
    `.trim()
      : ""}
  ></div>
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
        <div class="flex items-center gap-2">
          <h4 class="truncate font-semibold text-foreground">
            {model.name}
          </h4>
        </div>
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

          <div class="my-1 border-t border-border"></div>

          {#if isModelRunning(model)}
            <button
              class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs text-red-400 hover:bg-red-500/10"
              onclick={(e) => onAction("stop-model", model, e)}
            >
              <Square size={14} />
              Stop Model
            </button>
          {:else}
            <button
              class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs text-green-400 hover:bg-green-500/10"
              onclick={(e) => onAction("start-model", model, e)}
            >
              <Play size={14} />
              Start Model
            </button>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <div
    class={!isModelRunning(model)
      ? "opacity-50 pointer-events-none select-none transition-all duration-500"
      : "transition-all duration-500"}
  >
    <ModelUsageGraph
      isRunning={isModelRunning(model)}
      isMock={!isModelRunning(model)}
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
      <div class="flex flex-col">
        <span
          class="text-[9px] font-bold tracking-wider text-muted-foreground uppercase"
          >Params</span
        >
        <span class="font-mono text-[10px] font-bold text-foreground/90"
          >{metadata.params || "N/A"}</span
        >
      </div>
      <div class="flex flex-col">
        <span
          class="text-[9px] font-bold tracking-wider text-muted-foreground uppercase"
          >Format</span
        >
        <span class="font-mono text-[10px] font-bold text-foreground/90"
          >{metadata.format || "N/A"}</span
        >
      </div>
      <div class="flex flex-col">
        <span
          class="text-[9px] font-bold tracking-wider text-muted-foreground uppercase"
          >Size</span
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
        <Binary size={10} />
        <span>Quantization</span>
      </div>
      <span class="font-mono font-medium text-foreground/80"
        >{metadata.quant || "None"}</span
      >
    </div>
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
        <span>Architecture</span>
      </div>
      <span class="font-medium text-foreground/80">{model.library}</span>
    </div>
    {#if metadata.type}
      <div class="flex items-center justify-between text-[10px]">
        <div class="flex items-center gap-1.5 text-muted-foreground">
          <Tag size={10} />
          <span>Type</span>
        </div>
        <span class="font-medium text-foreground/80 uppercase"
          >{metadata.type}</span
        >
      </div>
    {/if}
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

  {#if currentStatus}
    <div
      bind:clientWidth={statusWidth}
      class={cn(
        "absolute bottom-0 left-4 z-[100] flex translate-y-1/2 items-center gap-1.5 text-[9px] font-bold uppercase tracking-tight transition-all",
        currentStatus.color,
      )}
    >
      <currentStatus.icon size={10} strokeWidth={3} />
      <span class="truncate">{currentStatus.label}</span>
    </div>
  {/if}

  {#if isSelected}
    <div
      class="absolute -right-1.5 -top-1.5 flex h-7 w-7 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg"
    >
      <Check size={16} strokeWidth={3} />
    </div>
  {:else if currentStatus?.icon === TriangleAlert}
    <div
      class={cn(
        "absolute -right-1.5 -top-1.5 flex h-7 w-7 items-center justify-center rounded-full text-white shadow-lg",
        currentStatus.badgeColor,
      )}
    >
      <TriangleAlert size={16} strokeWidth={3} />
    </div>
  {/if}
</div>
