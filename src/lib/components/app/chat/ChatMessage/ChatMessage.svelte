<script>
  import MessageAvatar from "../MessageAvatar/MessageAvatar.svelte";
  import MarkdownContent from "$lib/components/app/misc/MarkdownContent.svelte";

  /** @type {{ message: { role: string, content: string } }} */
  let { message } = $props();
</script>

<div class="message-row {message.role}">
  <div class="message-wrapper">
    <MessageAvatar role={message.role} />
    <div class="message-content">
      {#if message.role === "system"}
        <div class="system-message">
          {@html message.content}
        </div>
      {:else}
        <MarkdownContent content={message.content} />
      {/if}
    </div>
  </div>
</div>

<style>
  .message-row {
    width: 100%;
    padding: 0.5rem 0;
  }

  .message-wrapper {
    max-width: 48rem;
    margin: 0 auto;
    display: flex;
    gap: 1.5rem;
    padding: 0 1.5rem;
  }

  .message-content {
    flex-grow: 1;
    min-width: 0;
    font-size: 1rem;
    line-height: 1.6;
    color: var(--foreground);
  }

  /* Server-like system message styling */
  .system-message {
    font-size: 0.875rem;
    color: var(--muted-foreground);
    background-color: var(--secondary);
    padding: 0.75rem 1rem;
    border-radius: 0.75rem;
    border: 1px solid var(--border);
  }
</style>
