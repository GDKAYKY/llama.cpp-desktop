<script lang="ts">
  import { onMount } from "svelte";
  import {
    OrchestratorService,
    useConversation,
  } from "$lib/services/orchestrator";
  import { cn } from "$shared/cn.js";
  import type { Message, GenerationParams } from "$lib/types/backend";
  import { DEFAULT_GENERATION_PARAMS } from "$lib/types/backend";
  import { Eraser, Plus, Send, Info, X } from "lucide-svelte";

  let conversation = useConversation();
  let messages: Message[] = [];
  let input = "";
  let loading = false;
  let error: string | null = null;
  let slotId: string | null = null;

  onMount(async () => {
    try {
      slotId = await conversation.init(10);
      messages = [];
    } catch (e) {
      error = `Failed to initialize conversation: ${e}`;
    }
  });

  async function sendMessage() {
    if (!input.trim() || loading || !slotId) return;

    loading = true;
    error = null;

    try {
      const response = await OrchestratorService.sendMessage(
        slotId,
        input,
        DEFAULT_GENERATION_PARAMS,
      );

      // Refresh messages
      messages = await OrchestratorService.getSlotMessages(slotId);
      input = "";
    } catch (e) {
      error = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function clearChat() {
    if (!slotId) return;
    try {
      await OrchestratorService.clearSlot(slotId);
      messages = [];
      error = null;
    } catch (e) {
      error = `Error clearing chat: ${e}`;
    }
  }

  async function newChat() {
    try {
      if (slotId) {
        await OrchestratorService.deleteSlot(slotId);
      }
      slotId = await OrchestratorService.createSlot(10);
      messages = [];
      error = null;
    } catch (e) {
      error = `Error creating new chat: ${e}`;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }
</script>

<div
  class="flex h-full flex-col overflow-hidden rounded-xl border border-border bg-background shadow-lg"
>
  {#if error}
    <div
      class="flex items-center justify-between border-b border-red-500/30 bg-red-500/10 p-4"
    >
      <p class="text-sm font-medium text-red-500">{error}</p>
      <button
        class="text-red-500 hover:text-red-600 transition-colors"
        onclick={() => (error = null)}
      >
        <X size={18} />
      </button>
    </div>
  {/if}

  <div
    class="flex items-center justify-between border-b border-border bg-secondary/50 p-4"
  >
    <h2 class="text-lg font-bold">Chat Orchestrator</h2>
    <div class="flex gap-2">
      <button
        class="flex cursor-pointer items-center gap-2 rounded-lg border border-border bg-transparent px-3 py-1.5 text-xs font-medium transition-colors hover:bg-white/5 disabled:opacity-50"
        onclick={clearChat}
        disabled={loading || messages.length === 0}
      >
        <Eraser size={14} />
        Clear
      </button>
      <button
        class="flex cursor-pointer items-center gap-2 rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground transition-all hover:bg-primary/90 disabled:opacity-50"
        onclick={newChat}
        disabled={loading}
      >
        <Plus size={14} />
        New Chat
      </button>
    </div>
  </div>

  <div class="flex grow flex-col gap-4 overflow-y-auto p-4">
    {#if messages.length === 0}
      <div
        class="flex h-full items-center justify-center text-sm text-muted-foreground"
      >
        <p>Start a conversation by typing a message below.</p>
      </div>
    {:else}
      {#each messages as msg (msg.content)}
        <div
          class={cn(
            "flex max-w-[85%] flex-col gap-1 rounded-2xl p-4 shadow-xs",
            msg.role === "user"
              ? "self-end border-l-4 border-primary bg-primary/5"
              : msg.role === "system"
                ? "self-center border-l-4 border-orange-500 bg-orange-500/5 max-w-[90%]"
                : msg.role === "tool"
                  ? "self-start border-l-4 border-purple-500 bg-purple-500/5"
                  : "self-start border-l-4 border-muted-foreground bg-secondary",
          )}
        >
          <div
            class="text-[10px] font-bold uppercase tracking-wider text-muted-foreground"
          >
            {msg.role}
          </div>
          <div class="text-sm leading-relaxed break-words whitespace-pre-wrap">
            {msg.content}
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <div class="border-t border-border bg-secondary/30 p-4">
    <div class="flex gap-3">
      <textarea
        bind:value={input}
        placeholder="Type your message..."
        disabled={loading}
        on:keydown={handleKeydown}
        class="max-h-32 grow resize-none rounded-lg border border-border bg-input p-3 text-sm outline-none transition-all focus:border-primary disabled:opacity-50"
        rows="1"
      />
      <button
        class="flex cursor-pointer items-center gap-2 self-end rounded-lg bg-primary px-5 py-2.5 font-semibold text-primary-foreground transition-all hover:scale-105 hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-50 shadow-md shadow-primary/20"
        onclick={sendMessage}
        disabled={loading || !input.trim()}
      >
        <Send size={16} />
        {loading ? "..." : "Send"}
      </button>
    </div>
  </div>

  {#if slotId}
    <div
      class="flex items-center justify-end border-t border-border bg-secondary/50 px-4 py-2 text-[10px] text-muted-foreground"
    >
      <div class="flex items-center gap-1.5">
        <Info size={10} />
        <span>Slot: {slotId.substring(0, 8)}...</span>
        <span class="mx-1 opacity-30">|</span>
        <span>Messages: {messages.length}</span>
      </div>
    </div>
  {/if}
</div>
