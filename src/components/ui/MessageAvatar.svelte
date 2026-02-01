<script>
  import { cn } from "$shared/cn.js";
  /** @type {{ role: 'user' | 'assistant' | 'system', class?: string }} */
  let { role, class: className } = $props();
</script>

<div
  class={cn(
    "flex h-[30px] w-[30px] shrink-0 items-center justify-center rounded-[2px] text-[12px] font-bold text-white",
    role === "user" && "bg-[#5436da]",
    role === "system" && "bg-blue-500",
    role === "assistant" && "bg-primary",
    className,
  )}
>
  {role === "user" ? "U" : role === "system" ? "⚙" : "AI"}
</div>
