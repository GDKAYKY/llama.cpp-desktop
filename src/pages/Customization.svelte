<script>
  import {
    Paintbrush,
    SunMoon,
    LayoutGrid,
    MessageSquare,
    Palette,
    Monitor,
    Component,
    Save,
    Check,
  } from "lucide-svelte";

  let theme = $state("dark");
  let density = $state("balanced");
  let radius = $state(14);
  let accent = $state("ember");
</script>

<div
  class="min-h-screen py-6 md:py-10 mx-8 flex text-foreground bg-background font-sans selection:bg-primary/30"
>
  <!-- Main Content -->
  <main class="flex-1 flex flex-col gap-10">
    <!-- Header -->
    <div>
      <h1
        class="text-2xl md:text-3xl font-semibold tracking-tight text-foreground mb-2"
      >
        Customization
      </h1>
      <p class="text-sm text-muted-foreground">
        Personalize the interface appearance and layout.
      </p>
    </div>
    <!-- Theme Section -->
    <section class="flex flex-col gap-5">
      <div>
        <h2 class="text-lg font-medium text-foreground flex items-center gap-2">
          <SunMoon size={18} class="text-muted-foreground" /> Theme
        </h2>
        <p class="text-sm text-muted-foreground mt-1">
          Select the theme for the interface.
        </p>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <!-- Light -->
        <button
          onclick={() => (theme = "light")}
          class="group flex flex-col gap-3 p-4 rounded-xl border transition-all text-left focus:outline-none {theme ===
          'light'
            ? 'bg-zinc-800/80 border-white ring-1 ring-white/10 shadow-lg'
            : 'bg-zinc-900/50 border-zinc-800 hover:border-zinc-700 hover:bg-zinc-800/30'}"
        >
          <div
            class="h-24 rounded-lg bg-[#fafafa] border border-zinc-200 shadow-sm flex flex-col justify-between p-2 overflow-hidden transition-transform group-hover:scale-[1.02]"
          >
            <div class="w-full flex justify-between items-center">
              <div class="h-1.5 w-6 bg-zinc-300 rounded"></div>
              <div class="h-3 w-3 rounded-full bg-zinc-300"></div>
            </div>
            <div
              class="w-full h-10 bg-white border border-zinc-100 rounded shadow-sm"
            ></div>
          </div>
          <span
            class="text-sm font-medium flex items-center justify-between text-zinc-200 group-hover:text-white"
          >
            Light
            {#if theme === "light"}<Check size={16} class="text-white" />{/if}
          </span>
        </button>

        <!-- Dark -->
        <button
          onclick={() => (theme = "dark")}
          class="group flex flex-col gap-3 p-4 rounded-xl border transition-all text-left focus:outline-none {theme ===
          'dark'
            ? 'bg-zinc-800/80 border-white ring-1 ring-white/10 shadow-lg'
            : 'bg-zinc-900/50 border-zinc-800 hover:border-zinc-700 hover:bg-zinc-800/30'}"
        >
          <div
            class="h-24 rounded-lg bg-[#09090b] border border-zinc-800 shadow-sm flex flex-col justify-between p-2 overflow-hidden transition-transform group-hover:scale-[1.02]"
          >
            <div class="w-full flex justify-between items-center">
              <div class="h-1.5 w-6 bg-zinc-800 rounded"></div>
              <div class="h-3 w-3 rounded-full bg-zinc-800"></div>
            </div>
            <div
              class="w-full h-10 bg-zinc-900 border border-zinc-800/80 rounded shadow-sm"
            ></div>
          </div>
          <span
            class="text-sm font-medium flex items-center justify-between text-zinc-200 group-hover:text-white"
          >
            Dark
            {#if theme === "dark"}<Check size={16} class="text-white" />{/if}
          </span>
        </button>

        <!-- System -->
        <button
          onclick={() => (theme = "system")}
          class="group flex flex-col gap-3 p-4 rounded-xl border transition-all text-left focus:outline-none {theme ===
          'system'
            ? 'bg-zinc-800/80 border-white ring-1 ring-white/10 shadow-lg'
            : 'bg-zinc-900/50 border-zinc-800 hover:border-zinc-700 hover:bg-zinc-800/30'}"
        >
          <div
            class="h-24 rounded-lg bg-linear-to-br from-[#fafafa] to-[#09090b] border border-zinc-700/50 shadow-sm flex flex-col justify-between p-2 overflow-hidden transition-transform group-hover:scale-[1.02]"
          >
            <div class="w-full flex justify-between items-center">
              <div
                class="h-1.5 w-6 bg-black/20 rounded backdrop-blur border border-white/20"
              ></div>
              <div
                class="h-3 w-3 rounded-full bg-black/20 backdrop-blur border border-white/20"
              ></div>
            </div>
            <div
              class="w-full h-10 bg-white/10 backdrop-blur border border-white/20 rounded shadow-sm flex overflow-hidden"
            >
              <div class="w-1/2 h-full bg-white"></div>
              <div class="w-1/2 h-full bg-[#09090b]"></div>
            </div>
          </div>
          <span
            class="text-sm font-medium flex items-center justify-between text-zinc-200 group-hover:text-white"
          >
            System
            {#if theme === "system"}<Check size={16} class="text-white" />{/if}
          </span>
        </button>
      </div>
    </section>

    <hr class="border-border" />

    <!-- Details Section -->
    <section class="flex flex-col gap-6">
      <div>
        <h2 class="text-lg font-medium text-foreground flex items-center gap-2">
          <Palette size={18} class="text-muted-foreground" /> Visual Details
        </h2>
        <p class="text-sm text-muted-foreground mt-1">
          Adjust the spacing and rounding of components.
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        <!-- Densidade -->
        <div
          class="flex flex-col gap-3 bg-zinc-900/30 p-5 rounded-2xl border border-zinc-800/60"
        >
          <div class="text-sm font-medium text-zinc-300">Layout Density</div>
          <div
            class="inline-flex bg-zinc-900/80 border border-zinc-800/80 rounded-xl p-1 shadow-inner self-start mt-1"
          >
            <button
              onclick={() => (density = "compact")}
              class="px-4 py-2 rounded-lg text-sm font-medium transition-all {density ===
              'compact'
                ? 'bg-zinc-700/60 text-white shadow-sm border border-zinc-600/50'
                : 'text-zinc-400 hover:text-zinc-200 border border-transparent'}"
            >
              Compact
            </button>
            <button
              onclick={() => (density = "balanced")}
              class="px-4 py-2 rounded-lg text-sm font-medium transition-all {density ===
              'balanced'
                ? 'bg-zinc-700/60 text-white shadow-sm border border-zinc-600/50'
                : 'text-zinc-400 hover:text-zinc-200 border border-transparent'}"
            >
              Balanced
            </button>
            <button
              onclick={() => (density = "airy")}
              class="px-4 py-2 rounded-lg text-sm font-medium transition-all {density ===
              'airy'
                ? 'bg-zinc-700/60 text-white shadow-sm border border-zinc-600/50'
                : 'text-zinc-400 hover:text-zinc-200 border border-transparent'}"
            >
              Airy
            </button>
          </div>
        </div>

        <!-- Arredondamento -->
        <div
          class="flex flex-col gap-3 bg-zinc-900/30 p-5 rounded-2xl border border-zinc-800/60"
        >
          <div class="text-sm font-medium text-zinc-300 flex justify-between">
            <span>Rounding</span>
            <span
              class="text-white bg-white/10 px-2 py-0.5 rounded text-xs border border-white/20"
              >{radius}px</span
            >
          </div>
          <div
            class="flex items-center gap-4 bg-zinc-900/60 border border-zinc-800/50 rounded-xl p-3.5 mt-1 shadow-inner"
          >
            <div
              class="h-5 w-5 border-2 border-zinc-600 rounded-sm shrink-0"
            ></div>
            <input
              type="range"
              min="0"
              max="24"
              step="2"
              bind:value={radius}
              class="w-full accent-white h-1.5 bg-zinc-700 rounded-lg appearance-none cursor-pointer outline-none"
            />
            <div
              class="h-5 w-5 border-2 border-zinc-600 rounded-full shrink-0"
            ></div>
          </div>
        </div>
      </div>
    </section>

    <hr class="border-border" />

    <!-- Footer Actions -->
    <div class="flex justify-end pt-4 mb-10">
      <button
        class="flex items-center gap-2 bg-white hover:bg-zinc-200 text-zinc-950 px-6 py-2.5 rounded-xl font-semibold transition-all shadow-lg active:scale-95"
      >
        <Save size={18} />
        Save Settings
      </button>
    </div>
  </main>
</div>
