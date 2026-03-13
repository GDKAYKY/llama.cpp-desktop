<script lang="ts">
  import { type Snippet } from "svelte";

  let {
    as = "p",
    className = "",
    duration = 2,
    children,
  }: {
    as?: string;
    className?: string;
    duration?: number;
    spread?: number; // Kept for backwards compatibility, but we now use purely CSS-driven spread!
    children: Snippet;
  } = $props();
</script>

<svelte:element
  this={as}
  class="shimmer-text {className}"
  style="--duration: {duration}s;"
>
  {@render children()}
</svelte:element>

<style>
  .shimmer-text {
    position: relative;
    display: inline-block;
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    color: transparent;

    --base-color: #a1a1aa;
    --base-gradient-color: #000;
    --bg: linear-gradient(
      90deg,
      transparent 35%,
      var(--base-gradient-color) 50%,
      transparent 65%
    );
    background-size:
      250% 100%,
      auto;
    background-repeat: no-repeat, padding-box;
    background-image: var(--bg),
      linear-gradient(var(--base-color), var(--base-color));
    animation: shimmer var(--duration) linear infinite;
  }

  :global(.dark) .shimmer-text {
    --base-color: #71717a;
    --base-gradient-color: #ffffff;
  }

  @keyframes shimmer {
    from {
      background-position: 100% center;
    }
    to {
      background-position: 0% center;
    }
  }
</style>
