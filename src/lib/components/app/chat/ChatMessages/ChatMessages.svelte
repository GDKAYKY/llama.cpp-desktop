<script>
  import ChatMessage from "../ChatMessage/ChatMessage.svelte";
  import TypingIndicator from "../TypingIndicator/TypingIndicator.svelte";
  import MessageAvatar from "../MessageAvatar/MessageAvatar.svelte";

  /** @type {{ messages: Array<{role: string, content: string}>, isLoading: boolean, messagesEnd: HTMLElement }} */
  let { messages, isLoading, messagesEnd = $bindable() } = $props();
</script>

<div class="messages-container scrollbar-hide">
  <div class="messages-inner">
    {#each messages as msg}
      <ChatMessage message={msg} />
    {/each}
    {#if isLoading}
      <div class="message-row assistant">
        <div class="message-wrapper">
          <MessageAvatar role="assistant" />
          <div class="message-content">
            <TypingIndicator />
          </div>
        </div>
      </div>
    {/if}
    <div bind:this={messagesEnd} class="scroll-anchor"></div>
  </div>
</div>

<style>
  .messages-container {
    flex-grow: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    width: 100%;
    /* Matching server spacing */
    padding-top: 4rem; /* pt-16 */
  }

  @media (min-width: 768px) {
    .messages-container {
      padding-top: 6rem; /* pt-24 */
    }
  }

  .messages-inner {
    display: flex;
    flex-direction: column;
    gap: 2.5rem; /* space-y-10 */
    padding-bottom: 2rem;
  }

  .message-row {
    width: 100%;
  }

  .message-wrapper {
    max-width: 48rem; /* 768px */
    margin: 0 auto;
    display: flex;
    gap: 1.5rem;
    padding: 0 1.5rem;
  }

  .message-content {
    flex-grow: 1;
    line-height: 1.6;
    font-size: 1rem;
    color: var(--foreground);
    min-width: 0;
  }

  .scroll-anchor {
    height: 1px;
    margin-top: -1px;
  }
</style>
