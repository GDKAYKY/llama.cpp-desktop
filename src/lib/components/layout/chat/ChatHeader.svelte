<script lang="ts">
  import { cn } from "$shared/cn.js";
  import {
    ChevronDown,
    Check,
    Loader2,
    Square,
    PanelLeft,
    Info,
  } from "lucide-svelte";
  import { serverStore } from "$lib/stores/server.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { chatStore } from "$lib/stores/chat.svelte";
  import ModelLogo from "../../ui/models/ModelLogo.svelte";

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
   *   modelLoaded: boolean,
   *   toggleSessionPanel: () => void
   * }} */
  let {
    isSidebarOpen,
    isSidebarHidden,
    toggleSidebar,
    isLoading,
    toggleDropdown,
    selectedModel,
    isDropdownOpen,
    models,
    selectModel,
    handleClickOutside,
    modelLoaded,
    toggleSessionPanel,
  } = $props();

  function getModelMetadata(version: string) {
    const combined = version.toLowerCase();
    const quantMatch = combined.match(
      /(iq\d+_[a-z0-9_]+)|(q\d+_[a-z0-9_]+)|(q\d+_[a-z0-9])|(q\d+)|(fp16)|(bf16)|(f16)|(f32)/i,
    );
    return quantMatch ? quantMatch[0].toUpperCase() : null;
  }

  // Calculate context usage percentage
  function getContextUsage() {
    const userTokens = chatStore.messages
      .filter((m) => m.role === "user")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    const assistantTokens = chatStore.messages
      .filter((m) => m.role === "assistant")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    const systemTokens = chatStore.messages
      .filter((m) => m.role === "system")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    const toolTokens = chatStore.messages.reduce((sum, m) => {
      if (m.toolContext && m.toolContext.length > 0) {
        const toolText = JSON.stringify(m.toolContext);
        return sum + Math.ceil(toolText.length / 4);
      }
      return sum;
    }, 0);

    const totalTokens =
      userTokens + assistantTokens + systemTokens + toolTokens;
    const contextLimit = 200000;
    const usagePercent =
      totalTokens > 0 ? (totalTokens / contextLimit) * 100 : 0;

    return usagePercent;
  }

  const contextUsage = $derived(getContextUsage());
</script>

<svelte:window on:click={handleClickOutside} />

<header
  class={cn(
    "z-50 flex h-[60px] items-center px-4 p-2",
    isSidebarHidden
      ? "absolute top-0 left-0 right-0 bg-transparent"
      : cn(
          "sticky top-0",
          settingsStore.settings.chatHeaderStyle === "capsule"
            ? "bg-[#171717]"
            : "bg-background",
        ),
  )}
>
  <div class={cn("flex min-w-[80px] shrink-0", isSidebarHidden && "invisible")}></div>

  <div class="relative flex grow justify-center font-inter">
    <div class="relative flex w-full max-w-[400px] justify-center">
      <button
        type="button"
        class={cn(
          "flex max-w-full cursor-pointer items-center gap-2 rounded-lg px-3 py-1.5 text-sm font-medium text-foreground transition-colors hover:bg-white/5",
          isSidebarHidden && "bg-white/5 backdrop-blur-md backdrop-saturate-150 border border-white/10 shadow-sm",
        )}
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
            !isDropdownOpen && "-rotate-90",
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
            {#each models as model (model.full_identifier)}
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

  <div class={cn("flex min-w-[80px] shrink-0 items-center justify-end gap-1.5", isSidebarHidden && "invisible")}>
    {#if isLoading}
      <div
        class="flex items-center justify-center text-muted-foreground"
        aria-live="polite"
      >
        <Loader2 class="h-3.5 w-3.5 animate-spin" />
      </div>
    {/if}

    <!-- Square rounded button -->
    <button
      type="button"
      class="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-white/5"
      aria-label="Actions"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="1" />
        <circle cx="12" cy="5" r="1" />
        <circle cx="12" cy="19" r="1" />
      </svg>
    </button>

    <!-- Circle with context usage -->
    <button
      type="button"
      class="relative flex h-8 w-8 items-center justify-center rounded-lg transition-colors hover:bg-white/5"
      onclick={toggleSessionPanel}
      aria-label="Context usage: {contextUsage.toFixed(1)}%"
      title="Context usage: {contextUsage.toFixed(1)}%"
    >
      <!-- Background circle -->
      <svg class="h-5.5 w-5.5 -rotate-90" viewBox="0 0 24 24">
        <circle
          cx="12"
          cy="12"
          r="10.5"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          class="text-white/10"
        />
        <!-- Progress arc -->
        <circle
          cx="12"
          cy="12"
          r="10.5"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-dasharray="{(contextUsage / 100) * 65.97} 65.97"
          stroke-linecap="round"
          class="text-muted-foreground"
        />
      </svg>
    </button>
  </div>
</header>
