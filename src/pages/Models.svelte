<script>
  import ModelSelector from "$components/layout/ModelSelector.svelte";
  import { X } from "lucide-svelte";

  let currentModel = $state(null);

  function handleModelSelected(event) {
    currentModel = event.detail.model;
    console.log("Model selected:", currentModel);
  }
</script>

<div class="flex h-screen w-full flex-col bg-background text-foreground">
  {#if currentModel}
    <div
      class="flex shrink-0 items-center justify-between bg-linear-to-r from-[#667eea] to-[#764ba2] px-6 py-4 text-white shadow-md"
    >
      <div class="flex items-center gap-3">
        <span class="text-sm font-semibold">Current Model:</span>
        <span class="font-mono text-base font-bold"
          >{currentModel.name}:{currentModel.version}</span
        >
        <span class="rounded-full bg-white/20 px-3 py-1 text-xs font-semibold"
          >✓ Loaded</span
        >
      </div>
      <a
        href="/"
        class="flex h-10 w-10 items-center justify-center rounded-lg border border-white/30 bg-white/10 text-white transition-all hover:border-red-400 hover:bg-white/20 hover:text-red-400"
        title="Exit Models"
      >
        <X size={20} />
      </a>
    </div>
  {:else}
    <div
      class="flex shrink-0 items-center justify-between bg-linear-to-r from-[#667eea] to-[#764ba2] px-6 py-4 text-white shadow-md"
    >
      <h2 class="text-2xl font-semibold">Model Selection</h2>
      <a
        href="/"
        class="flex h-10 w-10 items-center justify-center rounded-lg border border-white/30 bg-white/10 text-white transition-all hover:border-red-400 hover:bg-white/20 hover:text-red-400"
        title="Exit Models"
      >
        <X size={20} />
      </a>
    </div>
  {/if}

  <div class="grow overflow-y-auto">
    <ModelSelector on:modelSelected={handleModelSelected} />
  </div>
</div>
