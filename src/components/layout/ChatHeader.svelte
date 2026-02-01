<script lang="ts">
  import { cn } from "$shared/cn.js";
  import { ChevronDown, Check, Loader2, Square } from "lucide-svelte";
  import { serverStore } from "$lib/stores/server.svelte";

  /** @type {{
   *   isSidebarOpen: boolean,
   *   toggleSidebar: () => void,
   *   isLoading: boolean,
   *   toggleDropdown: (e: MouseEvent) => void,
   *   selectedModel: any,
   *   isDropdownOpen: boolean,
   *   models: any[],
   *   selectModel: (model: any) => void,
   *   handleClickOutside: (e: MouseEvent) => void,
   *   modelLoaded: boolean
   * }} */
  let {
    isLoading,
    toggleDropdown,
    selectedModel,
    isDropdownOpen,
    models,
    selectModel,
    handleClickOutside,
    modelLoaded,
  } = $props();
</script>

<svelte:window on:click={handleClickOutside} />

<header
  class="sticky top-0 z-50 flex h-[54px] items-center border-b border-border bg-background px-4"
>
  <div class="flex min-w-[80px] shrink-0 items-center gap-2"></div>

  <div class="relative flex grow justify-center">
    <div class="relative flex w-full max-w-[400px] justify-center">
      <button
        type="button"
        class="flex max-w-full cursor-pointer items-center gap-2 rounded-lg border border-border bg-secondary px-3 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-white/5"
        aria-haspopup="listbox"
        aria-expanded={isDropdownOpen ? "true" : "false"}
        onclick={toggleDropdown}
      >
        <span class="overflow-hidden text-ellipsis whitespace-nowrap">
          {selectedModel ? selectedModel.name : "Select a model"}
        </span>

        <ChevronDown
          size={16}
          strokeWidth={2}
          class={cn(
            "shrink-0 transition-transform duration-200",
            isDropdownOpen && "rotate-180",
          )}
        />
      </button>

      {#if isDropdownOpen}
        <div
          class="absolute top-[calc(100%+8px)] left-1/2 z-100 w-320px -translate-x-1/2 overflow-hidden rounded-xl border border-border bg-secondary shadow-[0_10px_25px_-5px_rgba(0,0,0,0.3)]"
          role="listbox"
        >
          {#if models.length === 0}
            <div
              class="flex items-center justify-center p-4 italic text-muted-foreground pointer-events-none"
            >
              No models found
            </div>
          {:else}
            {#each models as model}
              <button
                type="button"
                role="option"
                aria-selected={selectedModel?.full_identifier ===
                model.full_identifier
                  ? "true"
                  : "false"}
                class={cn(
                  "flex w-full cursor-pointer items-center justify-between px-4 py-2.5 text-left text-foreground transition-colors hover:bg-white/5",
                  selectedModel?.full_identifier === model.full_identifier
                    ? "bg-white/10"
                    : "",
                )}
                onclick={() => selectModel(model)}
              >
                <div class="flex min-w-0 flex-col">
                  <span
                    class="overflow-hidden text-ellipsis whitespace-nowrap font-medium"
                    >{model.name}</span
                  >
                  <span
                    class="overflow-hidden text-ellipsis whitespace-nowrap text-[0.75rem] text-muted-foreground"
                    >{model.full_identifier}</span
                  >
                </div>

                {#if selectedModel?.full_identifier === model.full_identifier}
                  <Check size={16} strokeWidth={2} class="shrink-0" />
                {/if}
              </button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <div class="flex min-w-[80px] shrink-0 items-center justify-end gap-3">
    {#if serverStore.isRunning}
      <button
        onclick={() => serverStore.stopServer()}
        class="group flex h-8 items-center gap-2 rounded-lg border border-red-500/30 bg-red-500/10 pl-2 pr-3 text-red-400 transition-all hover:bg-red-500/20 hover:text-red-300 shadow-sm"
        title="Stop Server"
      >
        <Square
          size={12}
          fill="currentColor"
          class="transition-transform group-hover:scale-110"
        />
        <span class="text-[10px] font-bold uppercase tracking-wider">Stop</span>
      </button>
    {/if}

    {#if isLoading}
      <div
        class="flex items-center justify-center text-muted-foreground"
        aria-live="polite"
      >
        <Loader2 class="h-4 w-4 animate-spin" />
      </div>
    {/if}
  </div>
</header>
