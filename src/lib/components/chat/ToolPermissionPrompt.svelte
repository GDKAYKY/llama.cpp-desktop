<script lang="ts">
  import { ChevronRight, ChevronDown, Copy } from "lucide-svelte";
  import type { PendingPermission } from "$lib/stores/chat.svelte";
  import { notifications } from "$lib/shared/notifications";

  let { permission } = $props<{ permission: PendingPermission }>();

  let toolIcon = $derived((permission.toolName || "F").charAt(0).toUpperCase());
  let isDropdownOpen = $state(false);
  let showDetails = $state(false);

  function formatValue(value: any) {
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

  function highlightJson(value: any) {
    if (value === null || value === undefined) return "";
    let jsonStr = formatValue(value);

    return jsonStr.replace(
      /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*")(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?/g,
      function (match: string, p1: string, p2: string, p3: string, p4: string) {
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

  async function copyRequest() {
    try {
      const args = formatValue(permission.args ?? "");
      await navigator.clipboard.writeText(args);
      notifications.success("Request copied to clipboard");
    } catch (err) {
      notifications.error("Failed to copy request");
    }
  }
</script>

<svelte:window onclick={() => (isDropdownOpen = false)} />

<div
  class="my-3 text-[13px] text-muted-foreground/90 font-sans pointer-events-auto"
>
  <div class="mt-3 relative">
    <!-- Header mirroring ToolContextItem -->
    <div class="flex items-center gap-2.5 relative z-10">
      <div
        class="flex h-[22px] w-[22px] items-center justify-center rounded-[6px] bg-[#27272a] text-[10px] font-bold text-[#a1a1aa]"
      >
        {toolIcon}
      </div>
      <span class="font-medium text-[#e5e5e5] text-[13px] tracking-wide"
        >{permission.toolName || "Tool"}</span
      >
    </div>

    <!-- Connecting line to the prompt box -->
    <div class="relative ml-[11px] border-l border-[#3f3f46] pl-[18px] py-2.5">
      <!-- Permission Box exactly as in image.png -->
      <div
        class="flex w-full flex-col rounded-xl border border-[#3f3f46] bg-[#1e1e1e] p-2 shadow-lg max-w-[500px]"
      >
        <!-- Inner Header row -->
        <div
          class="flex items-center justify-between px-2 pt-1 pb-2 cursor-pointer hover:bg-white/5 rounded-lg transition-colors"
          role="button"
          tabindex="0"
          onclick={() => (showDetails = !showDetails)}
          onkeydown={(e) => e.key === "Enter" && (showDetails = !showDetails)}
        >
          <div class="flex items-center gap-2.5">
            <div
              class="flex h-[22px] w-[22px] items-center justify-center rounded-[6px] bg-[#27272a] text-[10px] font-bold text-[#a1a1aa]"
            >
              {toolIcon}
            </div>
            <span class="text-[13px] text-[#e5e5e5] font-medium tracking-wide">
              Claude quer usar {permission.toolName} de {permission.serverId}
            </span>
          </div>
          <ChevronRight
            size={14}
            class="text-[#a1a1aa] transition-transform duration-200 {showDetails
              ? 'rotate-90'
              : ''}"
          />
        </div>

        {#if showDetails}
          <div class="px-2 pb-2">
            <div
              class="rounded-lg bg-[#18181b] p-3 cursor-text w-full"
              onclick={(e) => e.stopPropagation()}
              role="presentation"
            >
              <div
                class="flex items-center justify-between text-[12px] font-bold text-[#e5e5e5] mb-2"
              >
                <span>Request</span>
                <button
                  class="p-0.5 text-[#a1a1aa] hover:text-[#e5e5e5] transition-colors"
                  onclick={copyRequest}
                  title="Copy request"
                  type="button"
                >
                  <Copy size={12} />
                </button>
              </div>
              <pre
                class="font-mono text-[12px] text-[#e5e5e5] whitespace-pre-wrap leading-relaxed">{@html highlightJson(
                  permission.args,
                )}</pre>
            </div>
          </div>
        {/if}

        <!-- Action buttons -->
        <div class="flex items-center gap-2 px-1 pb-1 pt-1 ml-9">
          <div class="relative flex items-center rounded-lg bg-[#e5e5e5]">
            <button
              class="flex items-center justify-center gap-2 px-3 py-1.5 text-[13px] font-semibold text-black transition-opacity hover:opacity-90 rounded-l-lg"
              onclick={() => permission.resolve(true)}
            >
              Sempre permitir
              <div
                class="flex items-center gap-0.5 rounded-[4px] bg-black/10 px-1 py-0.5 text-[10px] text-black/60 font-medium"
              >
                Enter
              </div>
            </button>
            <div class="h-4 w-[1px] bg-black/20"></div>
            <button
              class="flex items-center justify-center px-2 py-1.5 text-black transition-opacity hover:opacity-90 rounded-r-lg"
              onclick={(e) => {
                e.stopPropagation();
                isDropdownOpen = !isDropdownOpen;
              }}
            >
              <ChevronDown size={14} class="opacity-70" />
            </button>

            {#if isDropdownOpen}
              <div
                class="absolute bottom-[110%] left-0 min-w-full rounded-lg border border-[#3f3f46] bg-[#1e1e1e] p-1 shadow-lg z-[9999]"
              >
                <button
                  class="w-full text-left rounded-[6px] px-2 py-1.5 text-[13px] text-[#e5e5e5] hover:bg-[#27272a] transition-colors whitespace-nowrap font-medium"
                  onclick={(e) => {
                    e.stopPropagation();
                    isDropdownOpen = false;
                    permission.resolve(true);
                  }}
                >
                  Apenas uma vez
                </button>
              </div>
            {/if}
          </div>
          <button
            class="flex items-center justify-center gap-2 rounded-lg bg-[#27272a] px-3 py-1.5 text-[13px] font-semibold text-[#e5e5e5] transition-colors hover:bg-[#3f3f46]"
            onclick={() => permission.resolve(false)}
          >
            Negar
            <span
              class="rounded-[4px] bg-black/20 px-1 py-0.5 text-[10px] text-[#a1a1aa] font-medium"
              >Esc</span
            >
          </button>
        </div>
      </div>
    </div>
  </div>
</div>
