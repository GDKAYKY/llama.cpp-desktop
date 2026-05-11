<script lang="ts">
  import { cn } from "$shared/cn.js";
  import type { ComponentType } from "svelte";

  interface Props {
    variant?: "default" | "success" | "warning" | "error";
    icon?: ComponentType;
    class?: string;
    children?: any;
  }

  let {
    variant = "default",
    icon: Icon,
    class: className,
    children,
  }: Props = $props();

  const variantStyles = {
    default: "bg-white/10 text-foreground",
    success: "bg-green-500 text-white",
    warning: "bg-orange-500/20 text-orange-400 border-orange-500/30",
    error: "bg-red-500/20 text-red-400 border-red-500/30",
  };
</script>

<div
  class={cn(
    "inline-flex items-center gap-1.5 rounded-md border border-transparent px-2 py-0.5 text-[10px] font-semibold uppercase tracking-tight transition-colors",
    variantStyles[variant],
    className,
  )}
>
  {#if Icon}
    <Icon size={10} strokeWidth={3} />
  {/if}
  {@render children?.()}
</div>
