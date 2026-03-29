<script>
  import { ChevronDown, Wrench } from "lucide-svelte";
  import TextShimmer from "$components/ui/TextShimmer.svelte";
  import { cn } from "$shared/cn.js";
  import ToolContextItem from "./ToolContextItem.svelte";
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
    toolContext = [],
    messageContent = "",
    isStreaming = false,
    messageTimestamp = null,
  } = $props();

  let thinkingOpen = $state(false);
  let thinkingStartedAt = $state(null);
  let thinkingElapsed = $state(0);
  let thinkingForMessage = $state(null);

  let summary = $derived(
    summarizeThinking(
      thinkingProcess,
      modelThinking,
      toolContext,
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
    if (minutes <= 0) return `${remaining}s`;
    return `${minutes}m ${remaining}s`;
  }

  $effect(() => {
    if (thinkingProcess.length > 0 || modelThinking || toolContext.length > 0) {
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
              {#each groupedSteps as group, groupIndex}
                <div class="whitespace-pre-wrap wrap-break-word">
                  {#if isStreaming && groupIndex === groupedSteps.length - 1 && group.items.length === 0}
                    {#if isToolStep(group.title)}
                      <div class="flex items-center gap-2">
                        <Wrench size={12} class="text-muted-foreground/70" />
                        <TextShimmer duration={1.5}>{group.title}</TextShimmer>
                      </div>
                    {:else}
                      <TextShimmer duration={1.5}>{group.title}</TextShimmer>
                    {/if}
                  {:else if isToolStep(group.title)}
                    <div class="flex items-center gap-2">
                      <Wrench size={12} class="text-muted-foreground/70" />
                      <span>{group.title}</span>
                    </div>
                  {:else}
                    {group.title}
                  {/if}
                </div>
                {#if group.items.length > 0}
                  <ul class="ml-4 list-disc space-y-0.5">
                    {#each group.items as item, itemIndex}
                      <li class="whitespace-pre-wrap wrap-break-word">
                        {#if isStreaming && groupIndex === groupedSteps.length - 1 && itemIndex === group.items.length - 1}
                          {#if isToolStep(item)}
                            <div class="flex items-center gap-2">
                              <Wrench
                                size={12}
                                class="text-muted-foreground/70"
                              />
                              <TextShimmer duration={1.5}>{item}</TextShimmer>
                            </div>
                          {:else}
                            <TextShimmer duration={1.5}>{item}</TextShimmer>
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
              <div class="mt-2 text-[11px] text-muted-foreground/60">
                <div class="mb-1 uppercase tracking-wider"></div>
                <div class="flex flex-col gap-2">
                  {#each toolContext as ctx}
                    <ToolContextItem {ctx} {isStreaming} />
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
