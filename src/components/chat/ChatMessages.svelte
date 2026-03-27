<script>
  import ChatMessage from "./ChatMessage.svelte";
  import MessageAvatar from "$components/ui/MessageAvatar.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";

  /** @type {{ messages: Array<{role: string, content: string}>, isLoading: boolean, messagesEnd: HTMLElement, thinkingProcess?: string[], modelThinking?: string, pill?: boolean }} */
  let {
    messages,
    isLoading,
    thinkingProcess = [],
    modelThinking = "",
    messagesEnd = $bindable(),
    pill = true,
  } = $props();
</script>

<div class="flex w-full grow flex-col">
  <div class="flex flex-col pb-4">
    {#each messages as msg, i}
      <ChatMessage
        message={msg}
        index={i}
        isStreaming={isLoading && i === messages.length - 1}
        thinkingProcess={isLoading && i === messages.length - 1
          ? thinkingProcess
          : []}
        modelThinking={isLoading && i === messages.length - 1
          ? modelThinking
          : ""}
        {pill}
      />
    {/each}
    <div bind:this={messagesEnd} class="mt-[-1px] h-px"></div>
  </div>
</div>
