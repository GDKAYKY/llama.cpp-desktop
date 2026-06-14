<script lang="ts">
  import {
    ChevronDown,
    ChevronUp,
    X,
    Plus,
    Diff,
    Maximize2,
    Minimize2,
  } from "lucide-svelte";
  import { chatStore, type Message } from "$lib/stores/chat.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";
  import { serverStore } from "$lib/stores/server.svelte";
  import RevisionHistory from "./RevisionHistory.svelte";

  let {
    isOpen = false,
    onClose,
    width = $bindable(400),
    isResizing = $bindable(false),
  } = $props();

  let expandedMessageIndex = $state<number | null>(null);
  let activeTab = $state<"context" | "revision">("context");
  let startX = $state(0);
  let startWidth = $state(400);
  let isExpanded = $state(false);
  let previousWidth = $state(400);

  function toggleExpand() {
    if (isExpanded) {
      width = previousWidth;
      isExpanded = false;
    } else {
      previousWidth = width;
      width = window.innerWidth - 60; // Assuming 60px sidebar
      isExpanded = true;
    }
  }

  function toggleMessage(index: number) {
    expandedMessageIndex = expandedMessageIndex === index ? null : index;
  }

  function handleResizeStart(e: MouseEvent) {
    isResizing = true;
    startX = e.clientX;
    startWidth = width;
    e.preventDefault();
  }

  function handleResizeMove(e: MouseEvent) {
    if (!isResizing) return;

    const delta = startX - e.clientX;
    const newWidth = Math.min(Math.max(startWidth + delta, 300), 700);
    width = newWidth;
  }

  function handleResizeEnd() {
    isResizing = false;
  }

  $effect(() => {
    if (isResizing) {
      document.addEventListener("mousemove", handleResizeMove);
      document.addEventListener("mouseup", handleResizeEnd);
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
    } else {
      document.removeEventListener("mousemove", handleResizeMove);
      document.removeEventListener("mouseup", handleResizeEnd);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }

    return () => {
      document.removeEventListener("mousemove", handleResizeMove);
      document.removeEventListener("mouseup", handleResizeEnd);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
  });

  // Calculate context usage percentage
  function getContextUsage() {
    const userTokens = chatStore.messages
      .filter((m) => m.role === "user")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    const assistantTokens = chatStore.messages
      .filter((m) => m.role === "assistant")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    const systemTokens = chatStore.messages
      .filter((m) => m.role === "system")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    const toolTokens = chatStore.messages.reduce((sum, m) => {
      if (m.toolContext && m.toolContext.length > 0) {
        const toolText = JSON.stringify(m.toolContext);
        return sum + Math.ceil(toolText.length / 4);
      }
      return sum;
    }, 0);

    // Estimate reasoning tokens
    const reasoningTokens = chatStore.messages.reduce((sum, m) => {
      if (m.modelThinking) {
        return sum + Math.ceil(m.modelThinking.length / 4);
      }
      return sum;
    }, 0);

    const totalTokens =
      userTokens +
      assistantTokens +
      systemTokens +
      toolTokens +
      reasoningTokens;
    const contextLimit = serverStore.currentConfig?.ctx_size ?? 8192;
    const usagePercent =
      totalTokens > 0 ? (totalTokens / contextLimit) * 100 : 0;

    return usagePercent;
  }

  function formatTimestamp(timestamp: number) {
    const date = new Date(timestamp);
    return date.toLocaleString("pt-BR", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function getMessageId(index: number) {
    return `msg_${chatStore.sessionId.slice(0, 8)}${index.toString().padStart(3, "0")}`;
  }

  function buildMessagePayload(message: Message, index: number): object {
    return {
      message: {
        id: getMessageId(index),
        sessionID: chatStore.sessionId,
        role: message.role,
        time: {
          created: message.timestamp,
        },
        agent: "build",
        model: {
          modelID: message.model || "unknown",
          providerID: "llama.cpp",
          variant: "high",
        },
        content: message.content,
        ...(message.thinkingProcess && {
          thinkingProcess: message.thinkingProcess,
        }),
        ...(message.modelThinking && { modelThinking: message.modelThinking }),
        ...(message.toolContext && { toolContext: message.toolContext }),
      },
    };
  }

  // Calculate real token statistics
  function getTokenStats() {
    const userTokens = chatStore.messages
      .filter((m) => m.role === "user")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    const assistantTokens = chatStore.messages
      .filter((m) => m.role === "assistant")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    const systemTokens = chatStore.messages
      .filter((m) => m.role === "system")
      .reduce((sum, m) => sum + (m.tokens || 0), 0);

    // Estimate tool context tokens
    const toolTokens = chatStore.messages.reduce((sum, m) => {
      if (m.toolContext && m.toolContext.length > 0) {
        const toolText = JSON.stringify(m.toolContext);
        return sum + Math.ceil(toolText.length / 4);
      }
      return sum;
    }, 0);

    // Estimate reasoning tokens
    const reasoningTokens = chatStore.messages.reduce((sum, m) => {
      if (m.modelThinking) {
        return sum + Math.ceil(m.modelThinking.length / 4);
      }
      return sum;
    }, 0);

    const totalTokens =
      userTokens +
      assistantTokens +
      systemTokens +
      toolTokens +
      reasoningTokens;
    const contextLimit = serverStore.currentConfig?.ctx_size ?? 8192;
    const usagePercent =
      totalTokens > 0 ? (totalTokens / contextLimit) * 100 : 0;

    return {
      userTokens,
      assistantTokens,
      systemTokens,
      toolTokens,
      reasoningTokens,
      totalTokens,
      inputTokens: userTokens + systemTokens,
      outputTokens: assistantTokens,
      contextLimit,
      usagePercent,
    };
  }

  // Calculate context breakdown percentages
  function getContextBreakdown() {
    const stats = getTokenStats();
    const total = stats.totalTokens;

    if (total === 0) {
      return { user: 0, assistant: 0, tools: 0, reasoning: 0, other: 0 };
    }

    const userPercent = (stats.userTokens / total) * 100;
    const assistantPercent = (stats.assistantTokens / total) * 100;
    const toolPercent = (stats.toolTokens / total) * 100;
    const reasoningPercent = (stats.reasoningTokens / total) * 100;
    const otherPercent = (stats.systemTokens / total) * 100;

    return {
      user: userPercent,
      assistant: assistantPercent,
      tools: toolPercent,
      reasoning: reasoningPercent,
      other: otherPercent,
    };
  }

  // Get the most commonly used model in this conversation
  function getMostUsedModel() {
    const modelCounts = new Map<string, number>();

    chatStore.messages.forEach((m) => {
      if (m.model) {
        modelCounts.set(m.model, (modelCounts.get(m.model) || 0) + 1);
      }
    });

    if (modelCounts.size === 0) return "Unknown";

    let mostUsedModel = "";
    let maxCount = 0;

    modelCounts.forEach((count, model) => {
      if (count > maxCount) {
        maxCount = count;
        mostUsedModel = model;
      }
    });

    return mostUsedModel || "Unknown";
  }

  $effect(() => {
    if (!isOpen) {
      expandedMessageIndex = null;
    }
  });

  const tokenStats = $derived(getTokenStats());
  const breakdown = $derived(getContextBreakdown());
  const contextUsage = $derived(getContextUsage());
  const currentConversation = $derived(
    chatStore.history.find((c) => c.id === chatStore.activeConversationId),
  );
  const mostUsedModel = $derived(getMostUsedModel());
  const userMessageCount = $derived(
    chatStore.messages.filter((m) => m.role === "user").length,
  );
  const assistantMessageCount = $derived(
    chatStore.messages.filter((m) => m.role === "assistant").length,
  );

  const sessionInfoItems = $derived([
    { label: "Sessão", value: currentConversation?.title || "New Chat" },
    { label: "Mensagens", value: chatStore.messages.length },
    { label: "Provedor", value: "llama.cpp" },
    { label: "Modelo", value: mostUsedModel },
    {
      label: "Limite de Contexto",
      value: tokenStats.contextLimit.toLocaleString(),
    },
    {
      label: "Total de Tokens",
      value: tokenStats.totalTokens.toLocaleString(),
    },
    { label: "Uso", value: `${tokenStats.usagePercent.toFixed(1)}%` },
    {
      label: "Tokens de Entrada",
      value: tokenStats.inputTokens.toLocaleString(),
    },
    {
      label: "Tokens de Saída",
      value: tokenStats.outputTokens.toLocaleString(),
    },
    {
      label: "Tokens de Raciocínio",
      value: tokenStats.reasoningTokens.toLocaleString(),
    },
    { label: "Tokens de Cache (L/E)", value: "N/A" },
    { label: "Mensagens de Usuário", value: userMessageCount },
    { label: "Mensagens do Assistente", value: assistantMessageCount },
    { label: "Custo Total", value: "Gratuito" },
    {
      label: "Sessão Criada",
      value: currentConversation
        ? formatTimestamp(currentConversation.updatedAt)
        : "N/A",
    },
    {
      label: "Última Atividade",
      value:
        chatStore.messages.length > 0
          ? formatTimestamp(
              chatStore.messages[chatStore.messages.length - 1].timestamp,
            )
          : "N/A",
    },
  ]);

  const tabs = [
    {
      id: "revision" as const,
      label: "Revision",
      iconType: "diff" as const,
    },
    {
      id: "context" as const,
      label: "Contexto",
      showProgress: true,
    },
  ];

  const breakdownItems = $derived([
    { color: "bg-blue-500", label: "Usuário", percent: breakdown.user },
    {
      color: "bg-green-500",
      label: "Assistente",
      percent: breakdown.assistant,
    },
    {
      color: "bg-yellow-500",
      label: "Raciocínio",
      percent: breakdown.reasoning,
    },
    {
      color: "bg-purple-500",
      label: "Chamadas de Ferramentas",
      percent: breakdown.tools,
    },
    { color: "bg-gray-500", label: "Outros", percent: breakdown.other },
  ]);
</script>

{#if isOpen}
  <div
    class="relative flex h-full flex-col shrink-0 border-l border-border bg-background"
    style="width: {width}px"
    role="dialog"
    aria-label="Session Information"
  >
    <!-- Resize handle -->
    <button
      type="button"
      class="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-white/20 active:bg-white/30 border-0 p-0"
      aria-label="Resize panel"
      onmousedown={handleResizeStart}
    ></button>

    <div class="flex h-full flex-col">
      <!-- Tab Header -->
      <div
        class="flex h-15 items-center justify-between border-b border-border/50 px-3 bg-background"
      >
        <!-- Left from right: Active Tabs -->
        <div class="flex items-center gap-1">
          {#each tabs as tab}
            <div
              role="button"
              tabindex="0"
              class="group relative flex cursor-pointer items-center gap-1.5 rounded-lg px-3 py-1.5 text-[12px] font-medium transition-colors {activeTab ===
              tab.id
                ? 'bg-white/5 text-foreground hover:bg-white/10'
                : 'text-muted-foreground hover:bg-white/5'}"
              onclick={() => (activeTab = tab.id)}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  activeTab = tab.id;
                }
              }}
            >
              {#if tab.showProgress}
                <!-- Circle progress indicator -->
                <svg class="h-3.5 w-3.5 -rotate-90" viewBox="0 0 24 24">
                  <circle
                    cx="12"
                    cy="12"
                    r="10.5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    class="text-white/10"
                  />
                  <!-- Progress arc -->
                  <circle
                    cx="12"
                    cy="12"
                    r="10.5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-dasharray="{(contextUsage / 100) * 65.97} 65.97"
                    stroke-linecap="round"
                    class="text-current"
                  />
                </svg>
              {:else if tab.iconType === "diff"}
                <Diff size={12} strokeWidth={2} />
              {/if}
              <span>{tab.label}</span>
              {#if activeTab === tab.id}
                <div
                  role="button"
                  tabindex="0"
                  class="ml-1 rounded p-0.5 opacity-0 transition-opacity hover:bg-white/10 group-hover:opacity-100"
                  onclick={(e) => {
                    e.stopPropagation();
                    onClose();
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      onClose();
                    }
                  }}
                >
                  <X size={12} strokeWidth={2} />
                </div>
              {/if}

              <!-- Active indicator -->
              {#if activeTab === tab.id}
                <div
                  class="absolute -bottom-3.25 left-0 right-0 h-0.5 bg-white"
                ></div>
              {/if}
            </div>
          {/each}
        </div>

        <!-- Right: Actions -->
        <div class="flex items-center gap-1">
          <button
            type="button"
            class="flex h-6 w-6 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-white/5 hover:text-foreground"
            onclick={toggleExpand}
            title={isExpanded ? "Restaurar painel" : "Expandir painel"}
          >
            {#if isExpanded}
              <Minimize2 size={14} strokeWidth={2} />
            {:else}
              <Maximize2 size={14} strokeWidth={2} />
            {/if}
          </button>
          <button
            type="button"
            class="flex h-6 w-6 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-white/5 hover:text-foreground"
            aria-label="Add tab"
          >
            <Plus size={14} strokeWidth={2} />
          </button>
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto px-4 py-4">
        {#if activeTab === "context"}
          <!-- Session Info -->
          <div class="mb-6 space-y-2 text-[13px]">
            <div class="grid grid-cols-2 gap-3 text-muted-foreground">
              {#each sessionInfoItems as item}
                <div>
                  <div class="">{item.label}</div>
                  <div class="text-foreground font-medium">{item.value}</div>
                </div>
              {/each}
            </div>
          </div>

          <!-- Context Breakdown -->
          <div class="mb-6">
            <div class="mb-2 text-sm font-medium text-muted-foreground">
              Detalhamento do Contexto
            </div>
            <div
              class="mb-2 flex h-1.5 overflow-hidden rounded-full bg-white/5"
            >
              {#each breakdownItems as item}
                <div
                  class={item.color}
                  style="width: {item.percent}%"
                  title="{item.label} {item.percent.toFixed(1)}%"
                ></div>
              {/each}
            </div>
            <div class="flex flex-wrap gap-2 text-[13px] text-muted-foreground">
              {#each breakdownItems as item}
                <div class="flex items-center gap-2">
                  <div class="h-2.5 w-2.5 rounded-full {item.color}"></div>
                  <span>{item.label} {item.percent.toFixed(1)}%</span>
                </div>
              {/each}
            </div>
          </div>

          <!-- Raw Messages -->
          <div>
            <div class="mb-2 text-xs font-medium text-foreground">
              Mensagens brutas
            </div>
            <div class="">
              {#each chatStore.messages as message, index (index)}
                <div class="rounded-md border border-border/50 bg-white/2">
                  <button
                    type="button"
                    class="flex w-full items-center justify-between px-3 py-2 text-left text-[11px] transition-colors hover:bg-white/5"
                    onclick={() => toggleMessage(index)}
                  >
                    <div class="flex items-center gap-2">
                      <span class="font-mono text-white">{message.role}</span>
                      <span class="text-muted-foreground/50">·</span>
                      <span class="font-mono text-muted-foreground"
                        >{getMessageId(index)}</span
                      >
                      <span class="text-muted-foreground/50">·</span>
                      <span class="text-muted-foreground"
                        >{formatTimestamp(message.timestamp)}</span
                      >
                    </div>
                    {#if expandedMessageIndex === index}
                      <ChevronUp
                        size={12}
                        class="shrink-0 text-muted-foreground"
                      />
                    {:else}
                      <ChevronDown
                        size={12}
                        class="shrink-0 text-muted-foreground"
                      />
                    {/if}
                  </button>

                  {#if expandedMessageIndex === index}
                    <div class="border-t border-border/50 px-3 py-2">
                      <pre
                        class="overflow-x-auto text-[10px] text-muted-foreground">{JSON.stringify(
                          buildMessagePayload(message, index),
                          null,
                          2,
                        )}</pre>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {:else if activeTab === "revision"}
          <!-- Revision Content -->
          <div class="h-full w-full">
            <RevisionHistory />
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
