<script>
  import { ChevronDown, Copy, Wrench } from "lucide-svelte";
  import { toast } from "svelte-sonner";

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
      toast.success("Tool context copied to clipboard");
    } catch (err) {
      toast.error("Failed to copy tool context");
    }
  }
</script>

<details class="group rounded-md border-none bg-neutral-900 px-2 py-2">
  <summary
    class="flex cursor-pointer items-center gap-2 text-[12px] text-foreground/80"
  >
    <Wrench size={12} class="text-muted-foreground/70" />
    <span class="text-[10px] tracking-wider text-muted-foreground/60">
      {isStreaming ? "Calling MCP Tool" : "Called MCP Tool"}
    </span>
    <span>{ctx?.toolName || "tool"}</span>

    <ChevronDown
      size={12}
      class="ml-auto text-muted-foreground/70 transition-transform -rotate-90 group-open:rotate-0"
    />
  </summary>
  <div class="relative pt-2">
    <button
      class="absolute right-0 top-0 p-1 text-muted-foreground/70 hover:text-foreground transition-colors"
      onclick={copyToolContext}
      title="Copy tool context"
      type="button"
    >
      <Copy size={12} />
    </button>
    {#if ctx?.toolCallId}
      <div class="text-[10px] text-muted-foreground/60">
        {ctx.toolCallId}
      </div>
    {/if}
    <div class="mt-2 text-[11px] text-muted-foreground/60">
      <details class="group">
        <summary
          class="flex cursor-pointer items-center gap-2 text-[11px] text-muted-foreground/60"
        >
          Arguments
          <ChevronDown
            size={10}
            class="ml-auto text-muted-foreground/60 transition-transform -rotate-90 group-open:rotate-0"
          />
        </summary>
        <pre
          class="mt-1 whitespace-pre-wrap wrap-break-word rounded-md bg-background/40 px-2 py-1 text-[11px] text-muted-foreground/70">
{formatValue(ctx?.arguments)}
        </pre>
      </details>
    </div>
    <div class="mt-2 text-[11px] text-muted-foreground/60">
      <details class="group">
        <summary
          class="flex cursor-pointer items-center gap-2 text-[11px] text-muted-foreground/60"
        >
          Result
          <ChevronDown
            size={10}
            class="ml-auto text-muted-foreground/60 transition-transform -rotate-90 group-open:rotate-0"
          />
        </summary>
        <pre
          class="mt-1 whitespace-pre-wrap wrap-break-word rounded-md bg-background/40 px-2 py-1 text-[11px] text-muted-foreground/70">
{formatValue(ctx?.result)}
        </pre>
      </details>
    </div>
  </div>
</details>
