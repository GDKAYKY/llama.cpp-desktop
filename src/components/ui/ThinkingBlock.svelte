<script>
  import { Brain, ChevronDown } from "lucide-svelte";
  import { cn } from "$shared/cn.js";
  import { slide } from "svelte/transition";

  /** @type {{ thoughts: string[], isStreaming?: boolean }} */
  let { thoughts = [], isStreaming = false } = $props();

  let expanded = $state(true);

  $effect(() => {
    if (!isStreaming && thoughts.length > 0) {
      expanded = false;
    }
  });

  const label = $derived(
    isStreaming
      ? "Thinking\u2026"
      : `Thought for ${thoughts.length} step${thoughts.length !== 1 ? "s" : ""}`
  );
</script>

{#if thoughts.length > 0 || isStreaming}
  <div class="mb-3 w-full">
    <button
      type="button"
      class="group flex items-center gap-2 rounded-lg px-2 py-1 text-xs transition-colors hover:bg-white/5"
      onclick={() => (expanded = !expanded)}
    >
      <Brain
        size={13}
        class={cn(
          "shrink-0 transition-colors",
          isStreaming ? "text-muted-foreground/50" : "text-muted-foreground/40"
        )}
      />

      {#if isStreaming}
        <span class="thinking-gradient font-medium">
          {label}
        </span>
      {:else}
        <span class="text-muted-foreground/60 font-medium">{label}</span>
      {/if}

      <ChevronDown
        size={13}
        class={cn(
          "shrink-0 transition-transform duration-200 text-muted-foreground/40",
          expanded && "rotate-180"
        )}
      />
    </button>

    {#if expanded}
      <div
        transition:slide={{ duration: 180 }}
        class="mt-1 ml-1 border-l-2 border-white/10 pl-3"
      >
        {#each thoughts as thought}
          <p class="whitespace-pre-wrap text-[12px] leading-5 text-muted-foreground/55 [&+p]:mt-2">
            {thought}
          </p>
        {/each}

        {#if isStreaming}
          <p class="thinking-gradient mt-1 text-[12px] leading-5 font-medium">
            Thinking&hellip;
          </p>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .thinking-gradient {
    background: linear-gradient(
      90deg,
      rgba(161, 161, 170, 0.4) 0%,
      rgba(212, 212, 216, 0.9) 35%,
      rgba(161, 161, 170, 0.4) 70%
    );
    background-size: 200% auto;
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    animation: shimmer 2.2s linear infinite;
  }

  @keyframes shimmer {
    0% { background-position: 200% center; }
    100% { background-position: -200% center; }
  }
</style>
