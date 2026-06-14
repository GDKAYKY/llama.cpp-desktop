<script>
  import { ChevronDown, ChevronRight, CheckCircle2, Copy } from "lucide-svelte";
  import { cn } from "$shared/cn.js";
  import TextShimmer from "$components/ui/TextShimmer.svelte";
  import { notifications } from "$lib/shared/notifications";

  let { ctx, isStreaming = false } = $props();
  let showResult = $state(false);

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

  function highlightJson(value) {
    if (value === null || value === undefined) return "";
    let jsonStr = formatValue(value);

    return jsonStr.replace(
      /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*")(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?/g,
      function (match, p1, p2, p3, p4) {
        if (p3) {
          return `<span class="text-[#fca5a5]">${p1}</span><span class="text-[#e5e5e5]">${p3}</span>`;
        } else if (p1) {
          return `<span class="text-[#4ade80]">${p1}</span>`;
        } else if (p4) {
          return `<span class="text-[#c084fc]">${match}</span>`;
        } else {
          return `<span class="text-[#22d3ee]">${match}</span>`;
        }
      },
    );
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

  async function copyRequest() {
    try {
      const args = formatValue(ctx?.arguments ?? "");
      await navigator.clipboard.writeText(args);
      notifications.success("Request copied to clipboard");
    } catch (err) {
      notifications.error("Failed to copy request");
    }
  }

  async function copyResult() {
    try {
      const result = formatValue(ctx?.result ?? "");
      await navigator.clipboard.writeText(result);
      notifications.success("Result copied to clipboard");
    } catch (err) {
      notifications.error("Failed to copy result");
    }
  }

  // Get first letter of tool name for the icon
  let toolIcon = $derived((ctx?.toolName || "F").charAt(0).toUpperCase());
</script>

<div class="my-3 text-[14px] text-muted-foreground/90 font-sans">
  <details class="group" open>
    <summary
      class="inline-flex cursor-pointer items-center gap-1 font-medium hover:text-foreground transition-colors list-none [&::-webkit-details-marker]:hidden"
    >
      <span>{ctx?.toolName || "tool"}</span>
      <ChevronDown
        size={14}
        class="text-muted-foreground/70 transition-transform -rotate-90 group-open:rotate-0"
      />
    </summary>

    <div class="mt-3 relative">
      <!-- Header -->
      <div class="flex items-center gap-2.5 relative z-10">
        <div
          class="flex h-[22px] w-[22px] items-center justify-center rounded-[6px] bg-[#27272a] text-[10px] font-bold text-[#a1a1aa]"
        >
          {toolIcon}
        </div>
        <span
          class="font-medium text-muted-foreground/70 text-[13px] tracking-wide"
          >{ctx?.toolName || "Tool"}</span
        >
      </div>

      <!-- Connecting line and content -->
      <div
        class="relative ml-[11px] border-l border-[#3f3f46] pl-[18px] pb-2.5 pt-1.5"
      >
        {#if !showResult}
          <button
            class="inline-flex cursor-pointer items-center rounded-md bg-[#18181b] px-1.5 py-0.75 font-mono text-[10px] text-[#a1a1aa] transition-colors border border-transparent hover:bg-[#27272a] mb-4"
            onclick={() => (showResult = true)}
          >
            Result
          </button>
        {:else}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="flex w-full flex-col overflow-y-auto overflow-x-hidden rounded-xl border border-[#27272a] bg-[#1e1e1e] p-2 shadow-lg max-w-[500px] max-h-[400px] cursor-pointer"
            onclick={(e) => {
              if (window.getSelection()?.toString().length > 0) return;
              showResult = false;
            }}
          >
            {#if ctx?.arguments && (!ctx?.result || Object.keys(ctx?.arguments || {}).length > 0)}
              <div
                class="rounded-lg bg-[#18181b] p-3 mb-2 cursor-text w-full"
                onclick={(e) => e.stopPropagation()}
              >
                <div
                  class="flex items-center justify-between text-[12px] font-bold text-[#e5e5e5] mb-2"
                >
                  <span>Request</span>
                  <button
                    class="p-0.5 text-muted-foreground/50 hover:text-foreground transition-colors"
                    onclick={copyRequest}
                    title="Copy request"
                    type="button"
                  >
                    <Copy size={12} />
                  </button>
                </div>
                <pre
                  class="font-mono text-[12px] text-[#e5e5e5] whitespace-pre-wrap leading-relaxed">{@html highlightJson(
                    ctx?.arguments,
                  )}</pre>
              </div>
            {/if}

            {#if ctx?.result}
              {@const isError =
                typeof ctx?.result === "object" &&
                ctx.result !== null &&
                "isError" in ctx.result
                  ? ctx.result.isError
                  : false}
              <div
                class={cn(
                  "rounded-lg p-3 cursor-text w-full",
                  isError
                    ? "bg-[#3f1c1c] text-[#e5e5e5]"
                    : "bg-[#18181b] text-[#e5e5e5]",
                )}
                onclick={(e) => e.stopPropagation()}
              >
                <div
                  class="flex items-center justify-between text-[12px] font-bold text-[#e5e5e5] mb-2"
                >
                  <span>{isError ? "Error" : "Result"}</span>
                  <button
                    class="p-0.5 text-muted-foreground/50 hover:text-[#e5e5e5] transition-colors"
                    onclick={copyResult}
                    title="Copy result"
                    type="button"
                  >
                    <Copy size={12} />
                  </button>
                </div>
                <pre
                  class="font-mono text-[12px] text-[#e5e5e5] whitespace-pre-wrap leading-relaxed">{@html highlightJson(
                    ctx?.result,
                  )}</pre>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="flex items-center gap-2.5 relative z-10">
        {#if isStreaming && !ctx?.result}
          <div
            class="flex h-[22px] w-[22px] items-center justify-center bg-transparent"
          >
            <div
              class="h-1.5 w-1.5 rounded-full bg-muted-foreground/60 animate-pulse"
            ></div>
          </div>
          <TextShimmer
            duration={1.5}
            class="text-muted-foreground text-[13px] font-medium"
            >Executando...</TextShimmer
          >
        {:else}
          <div
            class="flex h-[22px] w-[22px] items-center justify-center bg-transparent"
          >
            <CheckCircle2 size={15} class="text-muted-foreground/60" />
          </div>
          <span class="text-muted-foreground/80 font-medium text-[13px]"
            >Concluído</span
          >
        {/if}
      </div>
    </div>
  </details>
</div>
