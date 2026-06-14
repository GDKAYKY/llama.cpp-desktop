<script>
  import { ChevronDown, CheckCircle2, Copy } from "lucide-svelte";
  import TextShimmer from "$components/ui/TextShimmer.svelte";
  import { notifications } from "$lib/shared/notifications";

  let { ctx, isStreaming = false } = $props();

  function formatValue(value) {
    if (value === null || value === undefined) return "";
    if (typeof value === "string") {
      try {
        const parsed = JSON.parse(value);
        return JSON.stringify(parsed, null, 2);
      } catch {
        return value;
      }
    }
    if (typeof value === "object") {
      try {
        return JSON.stringify(value, null, 2);
      } catch {
        return String(value);
      }
    }
    return String(value);
  }

  async function copyToolContext() {
    try {
      const args = formatValue(ctx?.arguments ?? "");
      const result = formatValue(ctx?.result ?? "");
      const payload = `Arguments:\n${args}\n\nResult:\n${result}`.trim();
      await navigator.clipboard.writeText(payload);
      notifications.success("Tool context copied to clipboard");
    } catch (err) {
      notifications.error("Failed to copy tool context");
    }
  }

  // Get first letter of tool name for the icon
  let toolIcon = $derived((ctx?.toolName || "F").charAt(0).toUpperCase());
</script>

<div class="my-3 text-[13px] text-muted-foreground/90 font-sans">
  <details class="group" open>
    <summary
      class="inline-flex cursor-pointer items-center gap-1.5 font-medium hover:text-foreground transition-colors list-none [&::-webkit-details-marker]:hidden"
    >
      <span>{ctx?.toolName || "tool"}</span>
      <ChevronDown
        size={14}
        class="text-muted-foreground/70 transition-transform -rotate-90 group-open:rotate-0"
      />
    </summary>

    <div class="mt-3 relative">
      <button
        class="absolute right-0 top-0 p-1 text-muted-foreground/50 hover:text-foreground transition-colors z-10"
        onclick={copyToolContext}
        title="Copy tool context"
        type="button"
      >
        <Copy size={14} />
      </button>

      <!-- Header -->
      <div class="flex items-center gap-2.5 relative z-10">
        <div class="flex h-[22px] w-[22px] items-center justify-center rounded-[6px] bg-[#27272a] text-[10px] font-bold text-[#a1a1aa]">
          {toolIcon}
        </div>
        <span class="font-medium text-[#e5e5e5] text-[13px] tracking-wide">{ctx?.toolName || "Tool"}</span>
      </div>

      <!-- Connecting line and content -->
      <div class="relative ml-[11px] border-l border-[#3f3f46] pl-[18px] py-2.5">
        {#if ctx?.arguments && (!ctx?.result || Object.keys(ctx?.arguments || {}).length > 0)}
          <div class="mb-2.5">
            <details class="group/args">
              <summary class="inline-flex cursor-pointer items-center rounded-md bg-[#18181b] px-2.5 py-1 font-mono text-[11px] text-[#a1a1aa] hover:bg-[#27272a] transition-colors border border-transparent list-none [&::-webkit-details-marker]:hidden">
                Arguments
              </summary>
              <pre
                class="mt-2 overflow-x-auto whitespace-pre-wrap wrap-break-word rounded-md bg-[#18181b] p-3 font-mono text-[11px] text-[#a1a1aa] border border-[#27272a] max-h-60"
              >{formatValue(ctx?.arguments)}</pre>
            </details>
          </div>
        {/if}

        {#if ctx?.result}
          <div class="">
            <details class="group/res">
              <summary class="inline-flex cursor-pointer items-center rounded-md bg-[#18181b] px-2.5 py-1 font-mono text-[11px] text-[#a1a1aa] hover:bg-[#27272a] transition-colors border border-transparent list-none [&::-webkit-details-marker]:hidden">
                Resultado
              </summary>
              <pre
                class="mt-2 overflow-x-auto whitespace-pre-wrap wrap-break-word rounded-md bg-[#18181b] p-3 font-mono text-[11px] text-[#a1a1aa] border border-[#27272a] max-h-60"
              >{formatValue(ctx?.result)}</pre>
            </details>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="flex items-center gap-2.5 relative z-10">
        {#if isStreaming && !ctx?.result}
          <div class="flex h-[22px] w-[22px] items-center justify-center bg-transparent">
            <div class="h-1.5 w-1.5 rounded-full bg-muted-foreground/60 animate-pulse"></div>
          </div>
          <TextShimmer duration={1.5} class="text-muted-foreground text-[13px] font-medium">Executando...</TextShimmer>
        {:else}
          <div class="flex h-[22px] w-[22px] items-center justify-center bg-transparent">
            <CheckCircle2 size={15} class="text-muted-foreground/60" />
          </div>
          <span class="text-muted-foreground/80 font-medium text-[13px]">Concluído</span>
        {/if}
      </div>
    </div>
  </details>
</div>
