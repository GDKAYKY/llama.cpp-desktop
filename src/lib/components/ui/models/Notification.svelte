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
    <div
      class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-lg"
    >
      <div class="inline-flex items-center gap-2">
        {#if isLoading}
          <span
            class="size-3 animate-spin rounded-full border-2 border-current border-t-transparent"
            aria-hidden="true"
          ></span>
        {/if}
        <span>{content}</span>
      </div>
    </div>
  </div>
{/if}
