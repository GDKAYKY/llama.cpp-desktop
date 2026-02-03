<script lang="ts">
  import { cn } from "$shared/cn.js";
  import { ChevronDown, Check, LoaderCircle, Square } from "lucide-svelte";
  import { serverStore } from "$lib/stores/server.svelte";
  import ModelLogo from "./ModelLogo.svelte";

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
  } = $props();

  function getModelMetadata(version: string) {
    const combined = version.toLowerCase();
    const quantMatch = combined.match(
      /(iq\d+_[a-z0-9_]+)|(q\d+_[a-z0-9_]+)|(q\d+_[a-z0-9])|(q\d+)|(fp16)|(bf16)|(f16)|(f32)/i,
    );
    return quantMatch ? quantMatch[0].toUpperCase() : null;
  }
</script>

<svelte:window on:click={handleClickOutside} />

<header
  class="sticky top-0 z-50 flex h-[60px] items-center bg-background px-4 p-2 shadow-background shadow-2xl"
>
  <div class="relative flex grow justify-center">
    <div class="relative flex w-full max-w-[400px] justify-center">
      <button
        type="button"
        class="flex max-w-full cursor-pointer items-center gap-2 rounded-lg px-3 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-white/5"
        aria-haspopup="listbox"
        aria-expanded={isDropdownOpen ? "true" : "false"}
        onclick={toggleDropdown}
      >
        <div
          class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-white/5 p-1"
        >
          <ModelLogo name={selectedModel?.name || ""} size={16} />
        </div>
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
                <div class="flex min-w-0 items-center gap-3">
                  <div
                    class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-white/5 p-1.5"
                  >
                    <ModelLogo name={model.name} size={18} />
                  </div>
                  <div class="flex min-w-0 flex-col">
                    <div class="flex items-center gap-2">
                      <span
                        class="overflow-hidden text-ellipsis whitespace-nowrap font-medium"
                        >{model.name}</span
                      >
                    </div>
                    <span
                      class="overflow-hidden text-ellipsis whitespace-nowrap text-[0.75rem] text-muted-foreground"
                      >{model.full_identifier}</span
                    >
                  </div>
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
    {#if isLoading}
      <div
        class="flex items-center justify-center text-muted-foreground"
        aria-live="polite"
      >
        <LoaderCircle class="h-4 w-4 animate-spin" />
      </div>
    {/if}
  </div>
</header>
