<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  let { isRunning = false, vramUsage = 0, gpuUsage = 0 } = $props();

  // Task Manager frequency jitter simulation
  let jitter = $state(0);
  let interval: any;

  $effect(() => {
    if (isRunning) {
      interval = setInterval(() => {
        jitter = (Math.random() - 0.5) * 4; // subtle noise
      }, 400); // 2.5Hz (similar to high-perf task manager)
    } else {
      jitter = 0;
      if (interval) clearInterval(interval);
    }
    return () => clearInterval(interval);
  });

  const totalSquares = 100; // 20x5 grid

  function getSquareColor(index: number) {
    const currentGpu = gpuUsage + (gpuUsage > 0 ? jitter : 0);
    // Normalize index to 0-100 scale
    const normalizedIndex = (index / totalSquares) * 100;

    // GPU activity takes precedence (Vibrant Indigo-White)
    if (normalizedIndex < currentGpu) return "#ffffff";
    // VRAM squares (Medium Gray)
    if (normalizedIndex < vramUsage) return "#4b4b4b";
    // Empty (Dark Gray)
    return "#161b22";
  }
</script>

<div class="flex flex-col gap-3 rounded-xl bg-[#171717] p-3">
  <div
    class="grid grid-cols-[repeat(20,10px)] gap-[3px] justify-start overflow-hidden"
  >
    {#each Array(totalSquares) as _, i}
      {@const color = getSquareColor(i)}
      <div
        class="h-[10px] w-[10px] rounded-[1.5px] transition-all duration-300"
        style="
          background-color: {color};
          {color === '#ffffff'
          ? 'border: 1px solid rgba(255,255,255,0.1);'
          : ''}
        "
      ></div>
    {/each}
  </div>

  <div class="flex items-center justify-between border-t border-white/5 pt-2">
    <span
      class="text-[8px] font-bold uppercase tracking-widest text-muted-foreground/40"
      >Model Usage</span
    >
    <div class="flex gap-4 items-center">
      <div class="flex items-center gap-1.5">
        <div class="h-1.5 w-1.5 bg-[#4b4b4b] rounded-[1px]"></div>
        <span class="text-[9px] font-mono text-muted-foreground/80"
          >VRAM: {Math.round(vramUsage)}%</span
        >
      </div>
      <div class="flex items-center gap-1.5">
        <div class="h-1.5 w-1.5 bg-white rounded-[1px]"></div>
        <span
          class="text-[9px] font-mono {gpuUsage > 0
            ? 'text-white font-bold'
            : 'text-muted-foreground/80'}">GPU: {Math.round(gpuUsage)}%</span
        >
      </div>
    </div>
  </div>
</div>

<style>
</style>
