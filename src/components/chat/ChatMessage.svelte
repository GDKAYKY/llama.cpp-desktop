<script>
  import MessageAvatar from "$components/ui/MessageAvatar.svelte";
  import MarkdownContent from "$components/ui/MarkdownContent.svelte";
  import { cn } from "$shared/cn.js";

  /** @type {{ message: { role: string, content: string } }} */
  let { message } = $props();
</script>

<div class="w-full py-4">
  <div
    class={cn(
      "mx-auto flex max-w-3xl px-4 md:px-6 gap-3 md:gap-4",
      message.role === "user" ? "flex-row-reverse" : "flex-row",
    )}
  >
    {#if message.role !== "user"}
      <div class="flex-shrink-0">
        <MessageAvatar role={message.role} />
      </div>
    {/if}

    <div
      class={cn(
        "flex flex-col min-w-0 max-w-[85%] md:max-w-[80%]",
        message.role === "user" ? "items-end ml-auto" : "items-start",
      )}
    >
      {#if message.role === "system"}
        <div
          class="rounded-xl border border-border bg-secondary px-4 py-3 text-sm text-muted-foreground"
        >
          {@html message.content}
        </div>
      {:else if message.role === "user"}
        <div
          class="w-fit rounded-[20px] bg-[#2f2f2f] px-4 py-2.5 text-white shadow-sm"
        >
          <div
            class="whitespace-pre-wrap break-words text-base leading-relaxed"
          >
            {message.content}
          </div>
        </div>
      {:else}
        <div class="w-full min-w-0 text-foreground">
          <MarkdownContent content={message.content} />
        </div>
      {/if}
    </div>

    {#if message.role === "user"}
      <div class="w-0 md:w-10"></div>
    {/if}
  </div>
</div>
