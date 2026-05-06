<script>
  import { cn } from "$shared/cn.js";

  /**
   * @type {{
   *   pill?: boolean,
   *   fullHeight?: boolean,
   *   header: import('svelte').Snippet,
   *   body: import('svelte').Snippet,
   *   class?: string
   * }}
   */
  let { pill = false, fullHeight = false, header, body, class: className } = $props();
</script>

<div class={cn("isolate", fullHeight && "flex flex-col h-full", className)} style="isolation: isolate;">
  <div
    class={cn(
      "px-3 py-1.5 text-[11px] font-bold uppercase tracking-wider text-muted-foreground/60",
      pill && "rounded-t-lg bg-secondary/60 border border-b-0 border-border",
    )}
  >
    {@render header()}
  </div>

  <div
    class={cn(
      "overflow-hidden",
      pill && "border border-t-0 border-border rounded-b-lg",
      fullHeight && "flex-1 min-h-0",
    )}
  >
    <div class={cn("overflow-y-auto", fullHeight ? "h-full" : "max-h-48")}>
      {@render body()}
    </div>
  </div>
</div>
