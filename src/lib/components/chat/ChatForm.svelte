<script lang="ts">
  import { cn } from "$lib/utils/cn.js";
  import {
    Paperclip,
    PlusCircle,
    FileCode,
    Globe,
    ArrowUp,
    Mic,
  } from "lucide-svelte";

  /** @type {{
   *   userInput: string,
   *   modelLoaded: boolean,
   *   isLoading: boolean,
   *   onKeydown: (e: KeyboardEvent) => void,
   *   onInput: () => void,
   *   onSend: () => void,
   *   textarea: HTMLTextAreaElement,
   *   selectedModel?: any
   * }} */
  let {
    userInput = $bindable(),
    modelLoaded,
    isLoading,
    onKeydown,
    onInput,
    onSend,
    textarea = $bindable(),
    selectedModel,
  } = $props();

  let isDropdownOpen = $state(false);

  function toggleDropdown(e: MouseEvent) {
    if (e) e.stopPropagation();
    isDropdownOpen = !isDropdownOpen;
  }

  function handleClickOutside() {
    isDropdownOpen = false;
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="mx-auto w-full max-w-3xl px-4 pb-6">
  <form
    class="rounded-3xl border border-border bg-secondary p-3 px-4 shadow-sm transition-shadow focus-within:shadow-md"
    onsubmit={(e: SubmitEvent) => {
      e.preventDefault();
      onSend();
    }}
  >
    <div class="flex flex-col gap-2">
      <div class="w-full">
        <textarea
          bind:this={textarea}
          bind:value={userInput}
          oninput={onInput}
          onkeydown={onKeydown}
          placeholder="Message llama-desktop..."
          rows="1"
          class="min-h-[24px] max-h-[200px] w-full resize-none border-none bg-transparent py-1 text-base leading-relaxed text-foreground outline-none placeholder:text-muted-foreground placeholder:opacity-60"
        ></textarea>
      </div>

      <div class="relative mt-1 flex items-center gap-3">
        <div class="mr-auto flex items-center">
          <div class="relative">
            <button
              type="button"
              class="flex cursor-pointer items-center justify-center rounded-full border-none bg-transparent p-2 text-muted-foreground transition-all hover:bg-white/5 hover:text-foreground"
              onclick={toggleDropdown}
              title="Attach files"
            >
              <Paperclip size={20} />
            </button>

            {#if isDropdownOpen}
              <div
                class="absolute bottom-[calc(100%+12px)] left-0 z-100 w-[200px] overflow-hidden rounded-xl border border-border bg-secondary p-1.5 shadow-lg"
                role="menu"
                tabindex="-1"
                onclick={(e: MouseEvent) => e.stopPropagation()}
                onkeydown={() => {}}
              >
                <button
                  type="button"
                  class="flex w-full cursor-pointer items-center gap-3 rounded-lg border-none bg-transparent px-3.5 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-white/8"
                >
                  <PlusCircle size={18} />
                  <span>Upload from computer</span>
                </button>
                <button
                  type="button"
                  class="flex w-full cursor-pointer items-center gap-3 rounded-lg border-none bg-transparent px-3.5 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-white/8"
                >
                  <FileCode size={18} />
                  <span>Search my files</span>
                </button>
                <button
                  type="button"
                  class="flex w-full cursor-pointer items-center gap-3 rounded-lg border-none bg-transparent px-3.5 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-white/8"
                >
                  <Globe size={18} />
                  <span>Search the web</span>
                </button>
              </div>
            {/if}
          </div>
        </div>
        <div class="flex items-center gap-2">
          <button
            type="button"
            class="flex cursor-pointer items-center justify-center rounded-full border-none bg-transparent p-2 text-muted-foreground transition-all hover:bg-white/5 hover:text-foreground"
            title="Voice input"
          >
            <Mic size={20} />
          </button>

          <button
            type="submit"
            disabled={!userInput.trim() || isLoading || !modelLoaded}
            class="flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-full border-none bg-muted-foreground text-background transition-all hover:enabled:scale-105 hover:enabled:bg-foreground disabled:cursor-not-allowed disabled:opacity-30"
            title="Send message"
          >
            <ArrowUp size={20} strokeWidth={2.5} />
          </button>
        </div>
      </div>
    </div>
  </form>

  <p class="mt-3 text-center text-[0.75rem] text-muted-foreground opacity-80">
    Llama-desktop can make mistakes. Check important info.
  </p>
</div>
