<script lang="ts">
  import { cn } from "$shared/cn";
  import { X } from "lucide-svelte";

  interface Props {
    title: string;
    isOpen: boolean;
    onClose: () => void;
    children: any;
  }

  let { title, isOpen, onClose, children }: Props = $props();

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-[100] flex items-center justify-center p-4"
    onkeydown={handleKeyDown}
    role="dialog"
    aria-modal="true"
  >
    <!-- backdrop -->
    <div
      class="absolute inset-0 bg-background/80 backdrop-blur-sm"
      onclick={onClose}
      role="presentation"
    ></div>

    <!-- modal -->
    <div
      class="relative z-10 flex max-h-[90vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-border bg-secondary shadow-2xl animate-in zoom-in-95 duration-200"
    >
      <div class="flex items-center justify-between border-b border-border p-4">
        <h3 class="text-lg font-semibold">{title}</h3>
        <button
          onclick={onClose}
          class="rounded-md p-1.5 text-muted-foreground hover:bg-white/5 hover:text-foreground transition-colors"
        >
          <X size={20} />
        </button>
      </div>

      <div class="overflow-y-auto p-6">
        {@render children()}
      </div>
    </div>
  </div>
{/if}
