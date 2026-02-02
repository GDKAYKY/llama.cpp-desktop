<script lang="ts">
  import { cn } from "$shared/cn.js";
  import ModelLogo from "$components/layout/ModelLogo.svelte";

  interface Props {
    role: "user" | "assistant" | "system";
    modelName?: string;
    class?: string;
  }

  let { role, modelName = "", class: className = "" }: Props = $props();
</script>

<div
  class={cn(
    "flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-lg text-[12px] font-bold text-white",
    role === "user" && "bg-[#5436da]",
    role === "system" && "bg-blue-500",
    role === "assistant" && "bg-primary/10 overflow-hidden", // Primary tint for logo background
    className,
  )}
>
  {#if role === "user"}
    U
  {:else if role === "system"}
    ⚙
  {:else}
    <ModelLogo name={modelName} size={20} />
  {/if}
</div>
