<script>
  import { ChevronDown, Wrench } from "lucide-svelte";
  import TextShimmer from "$components/ui/TextShimmer.svelte";
  import { cn } from "$shared/cn.js";
  import ToolContextItem from "$components/ui/tools/ToolContextItem.svelte";
  import MarkdownContent from "$components/ui/MarkdownContent.svelte";
  import {
    groupThinkingSteps,
    isToolStep,
    summarizeThinking,
  } from "$lib/utils/thinking-parser.js";

  let {
    thinkingProcess = [],
    modelThinking = "",
    thinkingLabel = "Thinking",
    thinkingTags = [],
    messageContent = "",
    isStreaming = false,
    messageTimestamp = null,
    thinkingTime = 0,
  } = $props();

  let thinkingOpen = $state(false);
  let thinkingStartedAt = $state(null);
  let thinkingElapsed = $state(0);
  let thinkingForMessage = $state(null);

  let summary = $derived(
    summarizeThinking(
      thinkingProcess,
      modelThinking,
      [],
      thinkingLabel,
      messageContent,
      thinkingTags,
    ),
  );

  let groupedSteps = $derived(groupThinkingSteps(summary.steps));

  function formatThinkingDuration(totalSeconds) {
    const seconds = Number.isFinite(totalSeconds)
      ? Math.max(0, totalSeconds)
      : 0;
    const minutes = Math.floor(seconds / 60);
    const remaining = seconds % 60;
    if (minutes <= 0) {
      if (seconds > 0) return `${Math.max(1, remaining)}s`;
      return "0s";
    }
    return `${minutes}m ${remaining}s`;
  }

  function normalizeThinkingText(text) {
    return String(text || "")
      .replace(/\\n/g, "\n")
      .trim();
  }

  $effect(() => {
    if (thinkingProcess.length > 0 || modelThinking) {
      thinkingOpen = true;
    }
  });

  $effect(() => {
    let intervalId;
    if (isStreaming) {
      if (thinkingForMessage !== messageTimestamp) {
        thinkingForMessage = messageTimestamp;
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
          {`Thinking... (${formatThinkingDuration(thinkingElapsed)})`}
        </TextShimmer>
      {:else}
        <span class="min-w-0 flex-1 truncate">
          {`Thinked for ${formatThinkingDuration(
            thinkingTime > 0 ? thinkingTime : thinkingElapsed,
          )}`}
        </span>
      {/if}
    </button>

    {#if thinkingOpen}
      <div>
        <div
          class="thinking-scroll max-h-56 overflow-y-auto mt-1 text-[15px] leading-6 md:text-base md:leading-relaxed text-foreground/90 break-words"
        >
          <div class="flex flex-col">
            {#if summary.steps.length > 0}
              {#each groupedSteps as group, groupIndex}
                <div>
                  {#if isStreaming && groupIndex === groupedSteps.length - 1 && group.items.length === 0}
                    {#if isToolStep(group.title)}
                      <div class="flex items-center gap-2">
                        <Wrench size={12} class="text-muted-foreground/70" />
                        <TextShimmer duration={1.5}>
                          {normalizeThinkingText(group.title)}
                        </TextShimmer>
                      </div>
                    {:else}
                      <TextShimmer duration={1.5}>
                        {normalizeThinkingText(group.title)}
                      </TextShimmer>
                    {/if}
                  {:else if isToolStep(group.title)}
                    <div class="flex items-center gap-2">
                      <Wrench size={12} class="text-muted-foreground/70 shrink-0" />
                      <span class="whitespace-pre-wrap text-inherit leading-inherit">
                        {normalizeThinkingText(group.title)}
                      </span>
                    </div>
                  {:else}
                    <MarkdownContent
                      content={normalizeThinkingText(group.title)}
                      class="text-inherit leading-inherit [&_p]:mb-0 [&_p]:text-inherit [&_p]:leading-inherit [&_pre]:my-2"
                    />
                  {/if}
                </div>
                {#if group.items.length > 0}
                  <ul class="ml-4 list-disc space-y-0.5">
                    {#each group.items as item, itemIndex}
                      <li>
                        {#if isStreaming && groupIndex === groupedSteps.length - 1 && itemIndex === group.items.length - 1}
                          {#if isToolStep(item)}
                          <div class="flex items-center gap-2">
                              <Wrench
                                size={12}
                                class="text-muted-foreground/70 shrink-0"
                              />
                              <TextShimmer duration={1.5}>
                                {normalizeThinkingText(item)}
                              </TextShimmer>
                            </div>
                          {:else}
                            <TextShimmer duration={1.5}>
                              {normalizeThinkingText(item)}
                            </TextShimmer>
                          {/if}
                        {:else if isToolStep(item)}
                          <div class="flex items-center gap-2">
                            <Wrench
                              size={12}
                              class="text-muted-foreground/70 shrink-0"
                            />
                            <span class="whitespace-pre-wrap text-inherit leading-inherit">
                              {normalizeThinkingText(item)}
                            </span>
                          </div>
                        {:else}
                          <MarkdownContent
                            content={normalizeThinkingText(item)}
                            class="text-inherit leading-inherit [&_p]:mb-0 [&_p]:text-inherit [&_p]:leading-inherit [&_pre]:my-2"
                          />
                        {/if}
                      </li>
                    {/each}
                  </ul>
                {/if}
              {/each}
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
