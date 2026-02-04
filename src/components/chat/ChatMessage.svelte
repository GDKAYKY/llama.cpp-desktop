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
  import { modelsStore } from "$lib/stores/models.svelte";
  import { toast } from "svelte-sonner";

  /** @type {{ message: { role: string, content: string }, index: number }} */
  let { message, index } = $props();

  let isEditing = $state(false);
  let editText = $state("");

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
      toast.success("Message copied to clipboard");
    } catch (err) {
      toast.error("Failed to copy message");
    }
  }

  function extractTokens(text) {
    const urlRegex = /https?:\/\/[^\s)]+/g;
    const tokens = [];
    let lastIndex = 0;
    let match;
    while ((match = urlRegex.exec(text)) !== null) {
      if (match.index > lastIndex) {
        tokens.push({ type: "text", value: text.slice(lastIndex, match.index) });
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
      toast.success("Curtido");
    } catch (err) {
      toast.error("Falha ao curtir");
    }
  }

  async function handleDislike() {
    try {
      await chatStore.dislikeMessage(index);
      toast.success("Feedback enviado");
    } catch (err) {
      toast.error("Falha ao enviar feedback");
    }
  }

  async function handleShare() {
    try {
      const path = await chatStore.shareMessage(index);
      toast.success(`Salvo em ${path}`);
    } catch (err) {
      toast.error("Falha ao compartilhar");
    }
  }

  async function handleRegenerate() {
    try {
      await chatStore.regenerateMessage(index);
    } catch (err) {
      toast.error("Falha ao regenerar");
    }
  }

  function handleMore() {
    toast.message("Mais ações em breve");
  }
</script>

<div class="group w-full py-4">
  <div
    class={cn(
      "mx-auto flex max-w-3xl px-4 md:px-6 gap-3 md:gap-4",
      message.role === "user" ? "flex-row-reverse" : "flex-row",
    )}
  >
    {#if message.role !== "user"}
      <div class="flex-shrink-0">
        <MessageAvatar role={message.role} modelName={message.model} />
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
        {#if isEditing}
          <div class="w-full flex-1 mb-4 relative flex min-w-0 flex-col">
            <div
              class="bg-secondary/50 border border-border rounded-3xl px-3 py-3"
            >
              <div class="m-2 max-h-[25vh] overflow-auto">
                <div class="grid">
                  <textarea
                    bind:value={editText}
                    class="col-start-1 col-end-2 row-start-1 row-end-2 w-full resize-none overflow-hidden p-0 m-0 border-0 bg-transparent text-foreground outline-none focus:ring-0"
                    placeholder="Edit message..."
                    onkeydown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        submitEdit();
                      }
                      if (e.key === "Escape") {
                        cancelEditing();
                      }
                    }}
                  ></textarea>
                  <span
                    class="invisible col-start-1 col-end-2 row-start-1 row-end-2 p-0 break-all whitespace-pre-wrap"
                    >{editText}
                  </span>
                </div>
              </div>
              <div class="flex justify-end gap-2 mt-2">
                <button
                  class="px-4 py-2 rounded-full text-sm font-medium hover:bg-white/5 transition-colors"
                  onclick={cancelEditing}
                >
                  Cancelar
                </button>
                <button
                  class="px-4 py-2 rounded-full text-sm font-medium bg-foreground text-background hover:opacity-90 transition-opacity"
                  onclick={submitEdit}
                >
                  Enviar
                </button>
              </div>
            </div>
          </div>
        {:else}
          <div
            class="w-fit rounded-[20px] bg-[#2f2f2f] px-4 py-2.5 text-white shadow-sm"
          >
            <div
              class="whitespace-pre-wrap break-words text-base leading-relaxed"
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
              class="mb-1 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60"
            >
              {message.model}
            </div>
          {/if}
          <MarkdownContent content={message.content} />
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
