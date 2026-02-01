<script>
  import MessageAvatar from "$components/ui/MessageAvatar.svelte";
  import MarkdownContent from "$components/ui/MarkdownContent.svelte";

  /** @type {{ message: { role: string, content: string } }} */
  let { message } = $props();
</script>

<div class="w-full py-2">
  <div class="mx-auto flex max-w-3xl gap-6 px-6">
    <MessageAvatar role={message.role} />
    <div class="min-w-0 grow text-base leading-relaxed text-foreground">
      {#if message.role === "system"}
        <div
          class="rounded-xl border border-border bg-secondary px-4 py-3 text-sm text-muted-foreground"
        >
          {@html message.content}
        </div>
      {:else}
        <MarkdownContent content={message.content} />
      {/if}
    </div>
  </div>
</div>
