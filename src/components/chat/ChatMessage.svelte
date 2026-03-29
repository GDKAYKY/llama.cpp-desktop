<script>
  import MessageAvatar from "$components/ui/MessageAvatar.svelte";
  import MarkdownContent from "$components/ui/MarkdownContent.svelte";
  import TextShimmer from "$components/ui/TextShimmer.svelte";
  import { cn } from "$shared/cn.js";
  import {
    Copy,
    Pencil,
    ThumbsUp,
    ThumbsDown,
    RotateCcw,
    Share2,
    MoreHorizontal,
    ChevronDown,
    Wrench,
  } from "lucide-svelte";
  import { chatStore } from "$lib/stores/chat.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";
  import { toast } from "svelte-sonner";

  /** @type {{ message: { role: string, content: string }, index: number, isStreaming?: boolean, thinkingProcess?: string[], modelThinking?: string, thinkingLabel?: string, thinkingTags?: string[], toolContext?: any[] }} */
  let {
    message,
    index,
    isStreaming = false,
    thinkingProcess = [],
    modelThinking = "",
    thinkingLabel = "Thinking",
    thinkingTags = [],
    toolContext = [],
  } = $props();

  let isEditing = $state(false);
  let editText = $state("");
  let thinkingOpen = $state(false);
  let thinkingStartedAt = $state(null);
  let thinkingElapsed = $state(0);
  let thinkingForMessage = $state(null);

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

  async function copyToolContext(ctx) {
    try {
      const args = formatValue(ctx?.arguments ?? "");
      const result = formatValue(ctx?.result ?? "");
      const payload = `Arguments:\n${args}\n\nResult:\n${result}`.trim();
      await navigator.clipboard.writeText(payload);
      toast.success("Tool context copied to clipboard");
    } catch (err) {
      toast.error("Failed to copy tool context");
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

  function summarizeThinking(
    entries,
    modelThinkingText,
    toolContextItems,
    labelFromTemplate,
    messageContent,
    tagCandidates,
  ) {
    const ignoredPatterns = [
      "tool loop",
      "no tool calls",
      "streaming final",
      "exceeded max iterations",
      "iteration",
    ];
    const detectedTags = resolveThinkingTags(
      tagCandidates,
      modelThinkingText,
      messageContent,
    );
    let cleaned = extractThinkingSteps(
      entries,
      modelThinkingText,
      messageContent,
      detectedTags,
    )
      .map((entry) => String(entry || "").trim())
      .filter(Boolean)
      .filter((entry) => {
        const text = entry.toLowerCase();
        return !ignoredPatterns.some((p) => text.includes(p));
      });

    let label = labelFromTemplate || "Thinking";
    if (toolContextItems.length > 0) {
      label = "Listed MCP tools";
    } else {
      const tagLabel = detectTagLabel(
        modelThinkingText,
        messageContent,
        detectedTags,
      );
      if (tagLabel) {
        label = tagLabel;
      } else {
        const lowered = cleaned.map((entry) => entry.toLowerCase());
        if (lowered.some((text) => text.includes("listing mcp"))) {
          label = "List MCP tools";
        } else if (lowered.some((text) => text.includes("calling mcp tool"))) {
          label = "Listed MCP tools";
        } else if (lowered.some((text) => text.includes("calling tool"))) {
          label = "Calling tool";
        }
      }
    }

    if (cleaned.length === 0 && toolContextItems.length > 0) {
      cleaned = toolContextItems.map((ctx) => formatToolStep(ctx?.toolName));
    }

    return { label, steps: cleaned };
  }

  function extractThinkingSteps(
    entries,
    modelThinkingText,
    messageContent,
    tags,
  ) {
    const steps = Array.isArray(entries) ? [...entries] : [];
    const raw = String(modelThinkingText || "");
    const contentRaw = modelThinkingText ? "" : String(messageContent || "");

    const quotedThinking = extractQuotedSteps(raw);
    const quotedContent = extractQuotedSteps(contentRaw);
    const taggedThinking = extractTaggedBlocks(raw, tags);
    const taggedContent = extractTaggedBlocks(contentRaw, tags);
    const normalized = replaceThinkingTags(raw, tags)
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean);

    let parsed = [];
    if (quotedThinking.length > 0) {
      parsed = quotedThinking;
    } else if (taggedThinking.length > 0) {
      parsed = taggedThinking;
    } else if (normalized.length > 0) {
      parsed = normalized;
    } else if (taggedContent.length > 0) {
      parsed = taggedContent;
    } else if (quotedContent.length > 0) {
      parsed = quotedContent;
    }

    if (parsed.length === 0) return steps;
    if (raw.trim()) return parsed;
    return mergeUniqueSteps(steps, parsed);
  }

  function extractQuotedSteps(text) {
    const raw = String(text || "");
    if (!raw.trim()) return [];
    return [...raw.matchAll(/^\s*>\s*(.+)$/gm)].map((match) => match[1].trim());
  }

  function resolveThinkingTags(
    tagCandidates,
    modelThinkingText,
    messageContent,
  ) {
    const provided = normalizeTagList(tagCandidates);
    if (provided.length > 0) return provided;
    const detected = detectTagsFromText(
      `${modelThinkingText}\n${messageContent}`,
    );
    if (detected.length > 0) return detected;
    return ["think", "analysis", "reasoning"];
  }

  function normalizeTagList(tags) {
    if (!Array.isArray(tags)) return [];
    return tags
      .map((tag) =>
        String(tag || "")
          .toLowerCase()
          .trim(),
      )
      .filter(Boolean);
  }

  function detectTagsFromText(text) {
    const raw = String(text || "");
    if (!raw.trim()) return [];
    const tags = new Set();
    const regex = /<\/?([a-zA-Z][a-zA-Z0-9_-]{0,32})>/g;
    let match = regex.exec(raw);
    while (match) {
      tags.add(match[1].toLowerCase());
      match = regex.exec(raw);
    }
    return [...tags];
  }

  function replaceThinkingTags(text, tags) {
    let result = String(text || "");
    const list = normalizeTagList(tags);
    for (const tag of list) {
      const safe = tag.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const open = new RegExp(`<${safe}>`, "gi");
      const close = new RegExp(`</${safe}>`, "gi");
      result = result.replace(close, "\n").replace(open, "");
    }
    return result;
  }

  function extractTaggedBlocks(text, tags) {
    const raw = String(text || "");
    if (!raw.trim()) return [];
    const list = normalizeTagList(tags);
    const blocks = [];
    for (const tag of list) {
      const safe = tag.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const regex = new RegExp(`<${safe}>([\\s\\S]*?)</${safe}>`, "gi");
      let match = regex.exec(raw);
      while (match) {
        const chunk = String(match[1] || "")
          .split(/\r?\n/)
          .map((line) => line.trim())
          .filter(Boolean);
        blocks.push(...chunk);
        match = regex.exec(raw);
      }
    }
    return blocks;
  }

  function mergeUniqueSteps(base, extra) {
    const result = [...base];
    for (const item of extra) {
      const trimmed = String(item || "").trim();
      if (!trimmed) continue;
      if (result[result.length - 1] === trimmed) continue;
      result.push(trimmed);
    }
    return result;
  }

  function detectTagLabel(modelThinkingText, messageContent, tags) {
    const raw =
      `${modelThinkingText || ""}\n${messageContent || ""}`.toLowerCase();
    for (const tag of normalizeTagList(tags)) {
      if (raw.includes(`<${tag}>`) || raw.includes(`</${tag}>`)) {
        return formatTagLabel(tag);
      }
    }
    return "";
  }

  function formatTagLabel(tag) {
    const normalized = String(tag || "")
      .replace(/[_-]+/g, " ")
      .trim();
    if (!normalized) return "Thinking";
    return normalized.replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatToolStep(toolName) {
    const raw = String(toolName || "").trim();
    if (!raw) return "Called tool";
    const normalized = raw.replace(/[_-]+/g, " ").trim();
    const lower = normalized.toLowerCase();
    if (lower.startsWith("list ")) return `Listed ${normalized.slice(5)}`;
    if (lower.startsWith("get ")) return `Got ${normalized.slice(4)}`;
    if (lower.startsWith("search ")) return `Searched ${normalized.slice(7)}`;
    if (lower.startsWith("find ")) return `Found ${normalized.slice(5)}`;
    if (lower.startsWith("read ")) return `Read ${normalized.slice(5)}`;
    return `Ran ${normalized}`;
  }

  function groupThinkingSteps(steps) {
    const groups = [];
    let current = null;
    for (const rawStep of steps) {
      const step = String(rawStep || "").trim();
      if (!step) continue;
      const numbered = step.match(/^\d+\.\s+(.*)$/);
      const bullet = step.match(/^(?:\*|-|•)\s+(.*)$/);
      if (numbered) {
        if (current) groups.push(current);
        current = { title: numbered[1].trim(), items: [] };
        continue;
      }
      if (bullet) {
        if (!current) {
          groups.push({ title: bullet[1].trim(), items: [] });
        } else {
          current.items.push(bullet[1].trim());
        }
        continue;
      }
      if (current) {
        current.items.push(step);
      } else {
        groups.push({ title: step, items: [] });
      }
    }
    if (current) groups.push(current);
    return groups;
  }

  function isToolStep(text) {
    const raw = String(text || "")
      .trim()
      .toLowerCase();
    return (
      raw.startsWith("calling mcp tool") || raw.startsWith("called mcp tool")
    );
  }

  function formatValue(value) {
    if (value === null || value === undefined) return "";
    if (typeof value === "string") {
      try {
        const parsed = JSON.parse(value);
        return JSON.stringify(parsed, null, 2);
      } catch {
        return value;
      }
    }
    if (typeof value === "object") {
      try {
        return JSON.stringify(value, null, 2);
      } catch {
        return String(value);
      }
    }
    return String(value);
  }

  $effect(() => {
    if (
      message.role === "assistant" &&
      (thinkingProcess.length > 0 || modelThinking || toolContext.length > 0)
    ) {
      thinkingOpen = true;
    }
  });

  function formatThinkingDuration(totalSeconds) {
    const seconds = Number.isFinite(totalSeconds)
      ? Math.max(0, totalSeconds)
      : 0;
    const minutes = Math.floor(seconds / 60);
    const remaining = seconds % 60;
    if (minutes <= 0) return `${remaining}s`;
    return `${minutes}m ${remaining}s`;
  }

  $effect(() => {
    let intervalId;
    if (isStreaming) {
      if (thinkingForMessage !== message.timestamp) {
        thinkingForMessage = message.timestamp;
        thinkingStartedAt = Date.now();
        thinkingElapsed = 0;
      } else if (!thinkingStartedAt) {
        thinkingStartedAt = Date.now();
      }
      intervalId = setInterval(() => {
        thinkingElapsed = Math.max(
          0,
          Math.floor((Date.now() - (thinkingStartedAt || Date.now())) / 1000),
        );
      }, 1000);
    }
    return () => {
      if (intervalId) clearInterval(intervalId);
    };
  });
</script>

<div class="group w-full py-2">
  <div
    class={cn(
      "mx-auto relative flex w-full max-w-[40rem] px-4 md:px-6 lg:max-w-[48rem] gap-3 md:gap-4",
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
          <div class="w-fit rounded-[20px] bg-[#2f2f2f] px-4 py-2.5 text-white">
            <div
              class="whitespace-pre-wrap break-words text-[15px] leading-6 md:text-base md:leading-relaxed"
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
          {#if message.role === "assistant" && (thinkingProcess.length > 0 || modelThinking || toolContext.length > 0)}
            {@const summary = summarizeThinking(
              thinkingProcess,
              modelThinking,
              toolContext,
              thinkingLabel,
              message.content,
              thinkingTags,
            )}
            <div class="w-full">
              <div class="relative">
                <button
                  class="flex w-full items-center gap-2 text-left text-[12px] text-muted-foreground/70 hover:text-muted-foreground transition-colors"
                  onclick={() => (thinkingOpen = !thinkingOpen)}
                  aria-expanded={thinkingOpen}
                  type="button"
                >
                  <ChevronDown
                    size={14}
                    class={cn(
                      "transition-transform text-muted-foreground/70",
                      thinkingOpen ? "rotate-0" : "-rotate-90",
                    )}
                  />
                  {#if isStreaming}
                    <TextShimmer class="min-w-0 flex-1 truncate" duration={1.5}>
                      Thinking
                    </TextShimmer>
                  {:else}
                    <span class="min-w-0 flex-1 truncate">
                      {`Thinked for ${formatThinkingDuration(thinkingElapsed)}`}
                    </span>
                  {/if}
                </button>

                {#if thinkingOpen}
                  <div class="pl-5">
                    <div
                      class="thinking-scroll max-h-56 overflow-y-auto text-[12px] text-muted-foreground/60 mt-1"
                    >
                      <div class="flex flex-col gap-1">
                        {#if summary.steps.length > 0}
                          {@const groupedSteps = groupThinkingSteps(
                            summary.steps,
                          )}
                          {#each groupedSteps as group, groupIndex}
                            <div class="whitespace-pre-wrap wrap-break-word">
                              {#if isStreaming && groupIndex === groupedSteps.length - 1 && group.items.length === 0}
                                {#if isToolStep(group.title)}
                                  <div class="flex items-center gap-2">
                                    <Wrench
                                      size={12}
                                      class="text-muted-foreground/70"
                                    />
                                    <TextShimmer duration={1.5}
                                      >{group.title}</TextShimmer
                                    >
                                  </div>
                                {:else}
                                  <TextShimmer duration={1.5}
                                    >{group.title}</TextShimmer
                                  >
                                {/if}
                              {:else if isToolStep(group.title)}
                                <div class="flex items-center gap-2">
                                  <Wrench
                                    size={12}
                                    class="text-muted-foreground/70"
                                  />
                                  <span>{group.title}</span>
                                </div>
                              {:else}
                                {group.title}
                              {/if}
                            </div>
                            {#if group.items.length > 0}
                              <ul class="ml-4 list-disc space-y-0.5">
                                {#each group.items as item, itemIndex}
                                  <li class="whitespace-pre-wrap break-words">
                                    {#if isStreaming && groupIndex === groupedSteps.length - 1 && itemIndex === group.items.length - 1}
                                      {#if isToolStep(item)}
                                        <div class="flex items-center gap-2">
                                          <Wrench
                                            size={12}
                                            class="text-muted-foreground/70"
                                          />
                                          <TextShimmer duration={1.5}
                                            >{item}</TextShimmer
                                          >
                                        </div>
                                      {:else}
                                        <TextShimmer duration={1.5}
                                          >{item}</TextShimmer
                                        >
                                      {/if}
                                    {:else if isToolStep(item)}
                                      <div class="flex items-center gap-2">
                                        <Wrench
                                          size={12}
                                          class="text-muted-foreground/70"
                                        />
                                        <span>{item}</span>
                                      </div>
                                    {:else}
                                      {item}
                                    {/if}
                                  </li>
                                {/each}
                              </ul>
                            {/if}
                          {/each}
                        {/if}
                        {#if toolContext.length > 0}
                          <div
                            class="mt-2 text-[11px] text-muted-foreground/60"
                          >
                            <div class="mb-1 uppercase tracking-wider"></div>
                            <div class="flex flex-col gap-2">
                              {#each toolContext as ctx}
                                <details
                                  class="group rounded-md border bg-neutral-900 px-2 py-2 border-0"
                                >
                                  <summary
                                    class="flex cursor-pointer items-center gap-2 text-[12px] text-foreground/80"
                                  >
                                    <Wrench
                                      size={12}
                                      class="text-muted-foreground/70"
                                    />
                                    <span
                                      class="text-[10px] tracking-wider text-muted-foreground/60"
                                    >
                                      {isStreaming
                                        ? "Calling MCP Tool"
                                        : "Called MCP Tool"}
                                    </span>
                                    <span>
                                      {ctx.toolName || "tool"}
                                    </span>

                                    <ChevronDown
                                      size={12}
                                      class="ml-auto text-muted-foreground/70 transition-transform -rotate-90 group-open:rotate-0"
                                    />
                                  </summary>
                                  <div class="relative pt-2">
                                    <button
                                      class="absolute right-0 top-0 p-1 text-muted-foreground/70 hover:text-foreground transition-colors"
                                      onclick={() => copyToolContext(ctx)}
                                      title="Copy tool context"
                                      type="button"
                                    >
                                      <Copy size={12} />
                                    </button>
                                    {#if ctx.toolCallId}
                                      <div
                                        class="text-[10px] text-muted-foreground/60"
                                      >
                                        {ctx.toolCallId}
                                      </div>
                                    {/if}
                                    <div
                                      class="mt-2 text-[11px] text-muted-foreground/60"
                                    >
                                      <details class="group">
                                        <summary
                                          class="flex cursor-pointer items-center gap-2 text-[11px] text-muted-foreground/60"
                                        >
                                          Arguments
                                          <ChevronDown
                                            size={10}
                                            class="ml-auto text-muted-foreground/60 transition-transform -rotate-90 group-open:rotate-0"
                                          />
                                        </summary>
                                        <pre
                                          class="mt-1 whitespace-pre-wrap break-words rounded-md bg-background/40 px-2 py-1 text-[11px] text-muted-foreground/70">
{formatValue(ctx.arguments)}
                                        </pre>
                                      </details>
                                    </div>
                                    <div
                                      class="mt-2 text-[11px] text-muted-foreground/60"
                                    >
                                      <details class="group">
                                        <summary
                                          class="flex cursor-pointer items-center gap-2 text-[11px] text-muted-foreground/60"
                                        >
                                          Result
                                          <ChevronDown
                                            size={10}
                                            class="ml-auto text-muted-foreground/60 transition-transform -rotate-90 group-open:rotate-0"
                                          />
                                        </summary>
                                        <pre
                                          class="mt-1 whitespace-pre-wrap break-words rounded-md bg-background/40 px-2 py-1 text-[11px] text-muted-foreground/70">
{formatValue(ctx.result)}
                                        </pre>
                                      </details>
                                    </div>
                                  </div>
                                </details>
                              {/each}
                            </div>
                          </div>
                        {/if}
                      </div>
                    </div>
                  </div>
                {/if}
              </div>
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

<style>
  .thinking-scroll {
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.18) transparent;
  }

  .thinking-scroll::-webkit-scrollbar {
    width: 8px;
  }

  .thinking-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .thinking-scroll::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.18);
    border-radius: 999px;
    border: 2px solid transparent;
    background-clip: content-box;
  }
</style>
