<script>
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    PanelLeftOpen,
    PanelLeftClose,
    Minus,
    Square,
    X,
  } from "lucide-svelte";
  import { cn } from "$shared/cn.js";

  let { isSidebarHidden, hideSidebar, showSidebar } = $props();

  const appWindow = getCurrentWindow();

  async function minimize() {
    await appWindow.minimize();
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
  }

  async function close() {
    await appWindow.close();
  }
</script>

<div
  data-tauri-drag-region
  class="flex h-9 w-full shrink-0 select-none items-center bg-sidebar"
>
  <!-- Hide/show sidebar button (sempre visível na titlebar) -->
  <div class="flex w-[60px] shrink-0 items-center justify-center">
    <button
      class="flex h-7 w-7 items-center justify-center rounded-md text-[#b4b4b4] transition-colors duration-[120ms] hover:bg-[#2f2f2f] hover:text-white"
      onclick={isSidebarHidden ? showSidebar : hideSidebar}
      title={isSidebarHidden ? "Show sidebar" : "Hide sidebar"}
    >
      {#if isSidebarHidden}
        <PanelLeftOpen size={16} strokeWidth={1.5} />
      {:else}
        <PanelLeftClose size={16} strokeWidth={1.5} />
      {/if}
    </button>
  </div>

  <!-- Drag region -->
  <div data-tauri-drag-region class="flex-1"></div>

  <!-- Window controls -->
  <div class="flex items-center">
    <button
      class="flex h-9 w-11 items-center justify-center text-[#b4b4b4] transition-colors duration-[120ms] hover:bg-[#2f2f2f] hover:text-white"
      onclick={minimize}
      title="Minimize"
    >
      <Minus size={14} strokeWidth={1.5} />
    </button>
    <button
      class="flex h-9 w-11 items-center justify-center text-[#b4b4b4] transition-colors duration-[120ms] hover:bg-[#2f2f2f] hover:text-white"
      onclick={toggleMaximize}
      title="Maximize"
    >
      <Square size={12} strokeWidth={1.5} />
    </button>
    <button
      class={cn(
        "flex h-9 w-11 items-center justify-center text-[#b4b4b4] transition-colors duration-[120ms]",
        "hover:bg-red-600 hover:text-white",
      )}
      onclick={close}
      title="Close"
    >
      <X size={14} strokeWidth={1.5} />
    </button>
  </div>
</div>
