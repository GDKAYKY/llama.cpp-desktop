<script lang="ts">
  import { type Snippet } from "svelte";

  let {
    class: className = "",
    children,
  }: {
    class?: string;
    children?: Snippet;
  } = $props();
</script>

<div class={`animate-pulse bg-muted rounded ${className}`}>
  {#if children}
    {@render children()}
  {/if}
</div>
