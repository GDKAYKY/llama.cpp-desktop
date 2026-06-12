<script lang="ts">
  let {
    message = "",
    text,
    show = true,
    type = "default",
    loading = false,
    progress = null,
  }: {
    message?: string;
    text?: string;
    show?: boolean;
    type?: "default" | "success" | "error" | "info" | "loading";
    loading?: boolean;
    progress?: number | null;
  } = $props();

  const content = $derived(message || text || "");
  const isLoading = $derived(loading || type === "loading");
  const progressLabel = $derived(
    typeof progress === "number" && Number.isFinite(progress)
      ? `${Math.max(0, Math.min(100, Math.round(progress)))}%`
      : null,
  );
</script>

{#if show && content}
  <div
    class="fixed bottom-8 left-1/2 z-100 -translate-x-1/2 animate-in fade-in slide-in-from-bottom-4 duration-300 pointer-events-none"
  >
    {#if isLoading}
      <div
        class="inline-flex items-center gap-3 rounded-full bg-primary px-4 py-2 shadow-lg"
      >
        <div
          class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-primary-foreground/15 text-[11px] font-semibold text-primary-foreground"
        >
          {#if progressLabel}
            {progressLabel}
          {:else}
            <span
              class="size-4 animate-spin rounded-full border-2 border-primary-foreground border-t-transparent"
              aria-hidden="true"
            ></span>
          {/if}
        </div>
        <span class="text-sm font-medium text-primary-foreground">{content}</span>
      </div>
    {:else}
      <div
        class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-lg"
      >
        <div class="inline-flex items-center gap-2">
          <span>{content}</span>
        </div>
      </div>
    {/if}
  </div>
{/if}
