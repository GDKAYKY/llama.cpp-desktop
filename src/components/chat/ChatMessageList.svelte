<script>
  import ChatMessage from "./ChatMessage.svelte";
  import MessageAvatar from "$components/ui/MessageAvatar.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";

  /** @type {{ messages: Array<{role: string, content: string, thinkingProcess?: string[], modelThinking?: string, toolContext?: any[]}>, isLoading: boolean, messagesEnd: HTMLElement, thinkingProcess?: string[], modelThinking?: string, thinkingLabel?: string, thinkingTags?: string[], toolContext?: any[], pill?: boolean }} */
  let {
    messages,
    isLoading,
    thinkingProcess = [],
    modelThinking = "",
    thinkingLabel = "Thinking",
    thinkingTags = [],
    toolContext = [],
    messagesEnd = $bindable(),
    pill = true,
  } = $props();
</script>

<div class="flex w-full grow flex-col">
  <div class="flex flex-col pb-4">
    {#each messages as msg, i}
      {@const isLast = i === messages.length - 1}
      {@const useLiveThinking =
        isLast &&
        (isLoading ||
          (msg.thinkingProcess == null &&
            msg.modelThinking == null &&
            msg.toolContext == null))}
      <ChatMessage
        message={msg}
        index={i}
        isStreaming={isLoading && isLast}
        thinkingProcess={useLiveThinking
          ? thinkingProcess
          : (msg.thinkingProcess ?? [])}
        modelThinking={useLiveThinking
          ? modelThinking
          : (msg.modelThinking ?? "")}
        {thinkingLabel}
        {thinkingTags}
        toolContext={useLiveThinking
          ? toolContext
          : (msg.toolContext ?? [])}
        {pill}
      />
    {/each}
    <div bind:this={messagesEnd} class="mt-[-1px] h-px"></div>
  </div>
</div>
