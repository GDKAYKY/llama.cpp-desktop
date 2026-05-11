<script lang="ts">
  import Skeleton from "$components/ui/Skeleton.svelte";

  let {
    isRunning = false,
    isStarting = false,
    vramUsage = 0,
    gpuUsage = 0,
  } = $props();

  // Task Manager frequency jitter simulation
  let jitter = $state(0);
  let interval: any;

  $effect(() => {
    // Only add jitter when model is actually running
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
    if (!isRunning) return "#101010";

    const currentVram = vramUsage;
    const currentGpu = gpuUsage + jitter;

    // Normalize index to 0-100 scale
    const normalizedIndex = (index / totalSquares) * 100;

    // GPU activity takes precedence
    if (normalizedIndex < currentGpu) return "#ffffff";
    // VRAM squares
    if (normalizedIndex < currentVram) return "#4b4b4b";
    // Empty
    return "#101010";
  }
</script>

<div class="flex flex-col rounded-xl bg-[#131313] p-4">
  {#if isStarting}
    <div class="pb-3">
      <Skeleton class="h-[62px] w-full" />
    </div>
  {:else}
    <div
      class="grid grid-cols-[repeat(20,10px)] gap-[3px] pb-3 justify-center overflow-hidden"
    >
      {#each Array(totalSquares) as _, i}
        {@const color = getSquareColor(i)}
        <div
          class="h-[10px] w-[10px] rounded-[1.5px] transition-all duration-300"
          style="background-color: {color};"
        ></div>
      {/each}
    </div>
  {/if}

  <div class="flex items-center justify-between">
    <span
      class="text-[8px] font-bold uppercase tracking-widest text-muted-foreground"
      >Model Usage</span
    >
    <div class="flex gap-4 items-center">
      <div class="flex items-center gap-1.5">
        <div
          class="h-1.5 w-1.5 rounded-[1px]"
          style="background-color: #4b4b4b"
        ></div>
        <span class="text-[9px] font-mono text-muted-foreground/80">
          VRAM: {isRunning ? Math.round(vramUsage) + "%" : "--"}
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <div
          class="h-1.5 w-1.5 rounded-[1px]"
          style="background-color: white"
        ></div>
        <span
          class="text-[9px] font-mono {isRunning && gpuUsage > 0
            ? 'text-white font-bold'
            : 'text-muted-foreground/80'}"
        >
          GPU: {isRunning ? Math.round(gpuUsage) + "%" : "--"}
        </span>
      </div>
    </div>
  </div>
</div>

<style>
</style>
