<script>
  import ChatMessage from "./ChatMessage.svelte";
  import TypingIndicator from "$components/ui/TypingIndicator.svelte";
  import MessageAvatar from "$components/ui/MessageAvatar.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";

  /** @type {{ messages: Array<{role: string, content: string}>, isLoading: boolean, messagesEnd: HTMLElement, thinkingProcess?: string[] }} */
  let { messages, isLoading, thinkingProcess = [], messagesEnd = $bindable() } =
    $props();
</script>

<div class="flex w-full grow flex-col">
  <div class="flex flex-col pb-8">
    {#each messages as msg, i}
      <ChatMessage message={msg} index={i} />
    {/each}
    {#if isLoading}
      <div class="w-full">
        <div class="mx-auto flex max-w-3xl gap-6 px-6">
          <MessageAvatar
            role="assistant"
            modelName={modelsStore.selectedModel?.name}
          />
          <div class="grow text-base text-foreground min-w-0">
            {#if thinkingProcess.length > 0}
              <div
                class="mb-3 rounded-md border border-white/10 bg-white/5 px-3 py-2 text-xs text-muted-foreground font-mono"
              >
                {#each thinkingProcess as step}
                  <div>{step}</div>
                {/each}
              </div>
            {/if}
            <TypingIndicator />
          </div>
        </div>
      </div>
    {/if}
    <div bind:this={messagesEnd} class="mt-[-1px] h-px"></div>
  </div>
</div>
