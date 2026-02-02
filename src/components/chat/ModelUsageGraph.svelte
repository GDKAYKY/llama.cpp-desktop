<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  let {
    isRunning = false,
    vramUsage = 0,
    gpuUsage = 0,
    isMock = false,
  } = $props();

  // Task Manager frequency jitter simulation
  let jitter = $state(0);
  let interval: any;

  $effect(() => {
    // Both running models and mock graphs get some jitter to feel "alive"
    if (isRunning || isMock) {
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
    // If not running and not mock, everything is empty
    if (!isRunning && !isMock) return "#121212";

    const currentVram = isMock ? 0 : vramUsage;
    const currentGpu = (isMock ? 0 : gpuUsage) + jitter;

    // Normalize index to 0-100 scale
    const normalizedIndex = (index / totalSquares) * 100;

    // GPU activity takes precedence
    if (normalizedIndex < currentGpu)
      return isMock ? "rgba(255, 255, 255, 0.08)" : "#ffffff";
    // VRAM squares
    if (normalizedIndex < currentVram)
      return isMock ? "rgba(255, 255, 255, 0.04)" : "#4b4b4b";
    // Empty
    return "#121212";
  }
</script>

<div class="flex flex-col rounded-xl bg-[#171717] p-3">
  <div
    class="grid grid-cols-[repeat(20,10px)] gap-[3px] pb-3 justify-center overflow-hidden"
  >
    {#each Array(totalSquares) as _, i}
      {@const color = getSquareColor(i)}
      <div
        class="h-[10px] w-[10px] rounded-[1.5px] transition-all duration-300"
        style="
          background-color: {color};
          {color === '#ffffff'}
        "
      ></div>
    {/each}
  </div>

  <div class="flex items-center justify-between border-t border-white/5 pt-2">
    <span
      class="text-[8px] font-bold uppercase tracking-widest text-muted-foreground/60"
      >Model Usage</span
    >
    <div class="flex gap-4 items-center">
      <div class="flex items-center gap-1.5">
        <div
          class="h-1.5 w-1.5 rounded-[1px]"
          style="background-color: {isMock ? '#4b4b4b44' : '#4b4b4b'}"
        ></div>
        <span
          class="text-[9px] font-mono {isMock
            ? 'text-muted-foreground/50'
            : 'text-muted-foreground/80'}"
        >
          VRAM: {isMock ? "--" : Math.round(vramUsage) + "%"}
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <div
          class="h-1.5 w-1.5 rounded-[1px]"
          style="background-color: {isMock ? '#4b4b4b22' : 'white'}"
        ></div>
        <span
          class="text-[9px] font-mono {isRunning && gpuUsage > 0
            ? 'text-white font-bold'
            : isMock
              ? 'text-muted-foreground/50'
              : 'text-muted-foreground/80'}"
        >
          GPU: {isMock ? "--" : Math.round(gpuUsage) + "%"}
        </span>
      </div>
    </div>
  </div>
</div>

<style>
</style>
