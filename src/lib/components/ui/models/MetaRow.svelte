<script lang="ts">
  import type { ComponentType } from "svelte";

  interface Props {
    icon: ComponentType;
    label: string;
    value: string | number;
    mono?: boolean;
  }

  let { icon: Icon, label, value, mono = false }: Props = $props();
</script>

<div class="flex items-center justify-between text-[11px]">
  <div class="flex items-center gap-1.5 text-muted-foreground">
    <Icon size={11} />
    <span>{label}</span>
  </div>
  <span
    class={mono
      ? "font-mono font-medium text-foreground/80"
      : "font-medium text-foreground/80"}
  >
    {value}
  </span>
</div>
