<script>
  import MessageAvatar from "$components/ui/MessageAvatar.svelte";
  import MarkdownContent from "$components/ui/MarkdownContent.svelte";
  import { cn } from "$shared/cn.js";
  import {
    Copy,
    Pencil,
    ThumbsUp,
    ThumbsDown,
    RotateCcw,
    Share2,
    MoreHorizontal,
  } from "lucide-svelte";
  import { chatStore } from "$lib/stores/chat.svelte";
  import { notifications } from "$lib/shared/notifications";
  import EditableMessage from "./EditableMessage.svelte";
  import ToolContextItem from "$components/ui/tools/ToolContextItem.svelte";
  import ToolPermissionPrompt from "./ToolPermissionPrompt.svelte";
  import ThinkingPanel from "./ThinkingPanel.svelte";

  /** @type {{ message: { role: string, content: string, model?: string, timestamp?: number }, index: number, isLast?: boolean, isStreaming?: boolean, thinkingProcess?: string[], modelThinking?: string, thinkingLabel?: string, thinkingTags?: string[], toolContext?: any[] }} */
  let {
    message,
    index,
    isLast = false,
    isStreaming = false,
    thinkingProcess = [],
    modelThinking = "",
    thinkingLabel = "Thinking",
    thinkingTags = [],
    toolContext = [],
    thinkingTime = 0,
  } = $props();

  let isEditing = $state(false);
  let editText = $state("");

  let messageParts = $derived.by(() => {
    const result = [];
    let lastIndex = 0;

    if (thinkingProcess.length > 0 || modelThinking) {
      result.push({ type: 'thinking' });
    }
    
    const sortedTools = [...toolContext].sort((a, b) => (a.textIndex || 0) - (b.textIndex || 0));
    const content = message.content || "";
    
    for (const ctx of sortedTools) {
      const idx = ctx.textIndex;
      if (idx !== undefined && idx !== null && idx >= lastIndex && idx <= content.length) {
        if (idx > lastIndex) {
          const textChunk = content.slice(lastIndex, idx);
          if (textChunk.trim()) {
            result.push({ type: 'text', content: textChunk });
          }
        }
        result.push({ type: 'tool', ctx });
        lastIndex = idx;
      }
    }
    
    if (lastIndex < content.length) {
      const textChunk = content.slice(lastIndex);
      if (textChunk.trim()) {
        result.push({ type: 'text', content: textChunk });
      }
    }
    
    for (const ctx of sortedTools) {
      const idx = ctx.textIndex;
      if (idx === undefined || idx === null || idx < 0 || idx > content.length) {
         result.push({ type: 'tool', ctx });
      }
    }
    
    return result;
  });

  function startEditing() {
    editText = message.content;
    isEditing = true;
  }

  function cancelEditing() {
    isEditing = false;
  }

  async function submitEdit() {
    if (!editText.trim() || editText === message.content) {
      cancelEditing();
      return;
    }
    isEditing = false;
    await chatStore.editMessage(index, editText);
  }

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(message.content);
      await chatStore.copyMessage(index);
      notifications.success("Message copied to clipboard");
    } catch (err) {
      notifications.error("Failed to copy message");
    }
  }

  function extractTokens(text) {
    const urlRegex = /https?:\/\/[^\s)]+/g;
    const tokens = [];
    let lastIndex = 0;
    let match;
    while ((match = urlRegex.exec(text)) !== null) {
      if (match.index > lastIndex) {
        tokens.push({
          type: "text",
          value: text.slice(lastIndex, match.index),
        });
      }
      tokens.push({ type: "link", value: match[0] });
      lastIndex = match.index + match[0].length;
    }
    if (lastIndex < text.length) {
      tokens.push({ type: "text", value: text.slice(lastIndex) });
    }
    return tokens;
  }

  function formatLinkLabel(url) {
    try {
      const parsed = new URL(url);
      return parsed.hostname;
    } catch {
      return url;
    }
  }

  async function handleLike() {
    try {
      await chatStore.likeMessage(index);
      notifications.success("Curtido");
    } catch (err) {
      notifications.error("Falha ao curtir");
    }
  }

  async function handleDislike() {
    try {
      await chatStore.dislikeMessage(index);
      notifications.success("Feedback enviado");
    } catch (err) {
      notifications.error("Falha ao enviar feedback");
    }
  }

  async function handleShare() {
    try {
      const path = await chatStore.shareMessage(index);
      notifications.success(`Salvo em ${path}`);
    } catch (err) {
      notifications.error("Falha ao compartilhar");
    }
  }

  async function handleRegenerate() {
    try {
      await chatStore.regenerateMessage(index);
    } catch (err) {
      notifications.error("Falha ao regenerar");
    }
  }

  function handleMore() {
    notifications.message("Mais ações em breve");
  }
</script>

<div class="group w-full py-2">
  <div
    class={cn(
      "mx-auto relative flex w-full max-w-160 px-4 md:px-6 lg:max-w-3xl gap-3 md:gap-4",
      message.role === "user" ? "flex-row-reverse" : "flex-row",
    )}
  >
    {#if message.role !== "user"}
      <div class="absolute -left-4 top-0 text-[initial]">
        <MessageAvatar role={message.role} modelName={message.model} />
      </div>
    {/if}

    <div
      class={cn(
        "flex min-w-0 flex-1 flex-col",
        message.role === "user"
          ? "items-end ml-auto max-w-[70%]"
          : "items-start w-full",
      )}
    >
      {#if message.role === "system"}
        <div
          class="rounded-xl border border-border bg-secondary px-4 py-3 text-[13px] leading-5 text-muted-foreground md:text-sm md:leading-relaxed"
        >
          {@html message.content}
        </div>
      {:else if message.role === "user"}
        {#if isEditing}
          <EditableMessage
            bind:value={editText}
            onCancel={cancelEditing}
            onSubmit={submitEdit}
          />
        {:else}
          <div class="w-fit rounded-[20px] bg-[#2f2f2f] px-4 py-2.5 text-white">
            <div
              class="whitespace-pre-wrap wrap-break-word text-[15px] leading-6 md:text-base md:leading-relaxed"
            >
              {#each extractTokens(message.content) as token}
                {#if token.type === "link"}
                  <a
                    href={token.value}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-sm text-white/90 hover:bg-white/10"
                  >
                    {formatLinkLabel(token.value)}
                  </a>
                {:else}
                  {token.value}
                {/if}
              {/each}
            </div>
          </div>

          <div
            class="mt-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
          >
            <button
              class="p-1.5 rounded-lg text-muted-foreground hover:bg-white/5 hover:text-foreground transition-all"
              onclick={copyToClipboard}
              title="Copy message"
            >
              <Copy size={16} />
            </button>
            <button
              class="p-1.5 rounded-lg text-muted-foreground hover:bg-white/5 hover:text-foreground transition-all"
              onclick={startEditing}
              title="Edit message"
            >
              <Pencil size={16} />
            </button>
          </div>
        {/if}
      {:else}
        <div class="w-full min-w-0 text-foreground">
          {#if message.role === "assistant" && message.model}
            <div
              class="text-[12px] font-inter tracking-wider text-muted-foreground/60 leading-none"
            >
              {message.model}
            </div>
          {/if}
          
          <div class="flex flex-col mt-2 gap-2">
            {#each messageParts as part}
              {#if part.type === 'text'}
                <MarkdownContent content={part.content} />
              {:else if part.type === 'tool'}
                <ToolContextItem ctx={part.ctx} {isStreaming} />
              {:else if part.type === 'thinking'}
                <ThinkingPanel
                  {thinkingProcess}
                  {modelThinking}
                  {thinkingLabel}
                  {thinkingTags}
                  {isStreaming}
                  messageContent={message.content}
                  messageTimestamp={message.timestamp ?? null}
                  {thinkingTime}
                />
              {/if}
            {/each}

            {#if chatStore.pendingPermission && isLast}
              <ToolPermissionPrompt permission={chatStore.pendingPermission} />
            {/if}
          </div>
          <div
            class="mt-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
          >
            <button
              class="p-1.5 rounded-lg text-muted-foreground hover:bg-white/5 hover:text-foreground transition-all"
              onclick={copyToClipboard}
              title="Copy message"
            >
              <Copy size={16} />
            </button>
            <button
              class="p-1.5 rounded-lg text-muted-foreground hover:bg-white/5 hover:text-foreground transition-all"
              onclick={handleLike}
              title="Like"
            >
              <ThumbsUp size={16} />
            </button>
            <button
              class="p-1.5 rounded-lg text-muted-foreground hover:bg-white/5 hover:text-foreground transition-all"
              onclick={handleDislike}
              title="Dislike"
            >
              <ThumbsDown size={16} />
            </button>
            <button
              class="p-1.5 rounded-lg text-muted-foreground hover:bg-white/5 hover:text-foreground transition-all"
              onclick={handleShare}
              title="Share"
            >
              <Share2 size={16} />
            </button>
            <button
              class="p-1.5 rounded-lg text-muted-foreground hover:bg-white/5 hover:text-foreground transition-all"
              onclick={handleRegenerate}
              title="Regenerate"
            >
              <RotateCcw size={16} />
            </button>
            <button
              class="p-1.5 rounded-lg text-muted-foreground hover:bg-white/5 hover:text-foreground transition-all"
              onclick={handleMore}
              title="More"
            >
              <MoreHorizontal size={16} />
            </button>
          </div>
        </div>
      {/if}
    </div>

    {#if message.role === "user"}
      <div class="w-0 md:w-10"></div>
    {/if}
  </div>
</div>
