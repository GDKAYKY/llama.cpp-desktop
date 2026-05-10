<script lang="ts">
  let {
    message = "",
    text,
    show = true,
    type = "default",
    loading = false,
  }: {
    message?: string;
    text?: string;
    show?: boolean;
    type?: "default" | "success" | "error" | "info" | "loading";
    loading?: boolean;
  } = $props();

  const content = $derived(message || text || "");
  const isLoading = $derived(loading || type === "loading");
</script>

{#if show && content}
  <div
    class="fixed bottom-8 left-1/2 z-100 -translate-x-1/2 animate-in fade-in slide-in-from-bottom-4 duration-300 pointer-events-none"
  >
    {#if isLoading}
      <div
        class="rounded-full bg-primary w-12 h-12 shadow-lg flex items-center justify-center"
      >
        <span
          class="size-4 animate-spin rounded-full border-2 border-primary-foreground border-t-transparent"
          aria-hidden="true"
        ></span>
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
