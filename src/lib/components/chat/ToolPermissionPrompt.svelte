<script lang="ts">
  import { ChevronRight } from "lucide-svelte";
  import type { PendingPermission } from "$lib/stores/chat.svelte";

  let { permission } = $props<{ permission: PendingPermission }>();
  
  let toolIcon = $derived((permission.toolName || "F").charAt(0).toUpperCase());
</script>

<div class="my-3 text-[13px] text-muted-foreground/90 font-sans pointer-events-auto">
  <div class="mt-3 relative">
    
    <!-- Header mirroring ToolContextItem -->
    <div class="flex items-center gap-2.5 relative z-10">
      <div class="flex h-[22px] w-[22px] items-center justify-center rounded-[6px] bg-[#27272a] text-[10px] font-bold text-[#a1a1aa]">
        {toolIcon}
      </div>
      <span class="font-medium text-[#e5e5e5] text-[13px] tracking-wide">{permission.toolName || "Tool"}</span>
    </div>

    <!-- Connecting line to the prompt box -->
    <div class="relative ml-[11px] border-l border-[#3f3f46] pl-[18px] py-2.5">
      
      <!-- Permission Box exactly as in image.png -->
      <div class="flex w-full flex-col overflow-hidden rounded-xl border border-[#3f3f46] bg-[#1e1e1e] p-2 shadow-lg max-w-[500px]">
        
        <!-- Inner Header row -->
        <div class="flex items-center justify-between px-2 pt-1 pb-2">
          <div class="flex items-center gap-2.5">
            <div class="flex h-[22px] w-[22px] items-center justify-center rounded-[6px] bg-[#27272a] text-[10px] font-bold text-[#a1a1aa]">
              {toolIcon}
            </div>
            <span class="text-[13px] text-[#e5e5e5] font-medium tracking-wide">
              Claude quer usar {permission.toolName} de {permission.serverId}
            </span>
          </div>
          <ChevronRight size={14} class="text-[#a1a1aa]" />
        </div>

        <!-- Action buttons -->
        <div class="flex items-center gap-2 px-1 pb-1 pt-1">
          <button 
            class="flex-1 flex items-center justify-center gap-2 rounded-lg bg-[#e5e5e5] px-3 py-1.5 text-[13px] font-semibold text-black transition-opacity hover:opacity-90"
            onclick={() => permission.resolve(true)}
          >
            Sempre permitir 
            <div class="flex items-center gap-0.5 rounded-[4px] bg-black/10 px-1 py-0.5 text-[10px] text-black/60 font-medium">
              Enter
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>
            </div>
          </button>
          <button 
            class="flex-1 flex items-center justify-center gap-2 rounded-lg bg-[#27272a] px-3 py-1.5 text-[13px] font-semibold text-[#e5e5e5] transition-colors hover:bg-[#3f3f46] border border-[#3f3f46]"
            onclick={() => permission.resolve(false)}
          >
            Negar 
            <span class="rounded-[4px] bg-black/20 px-1 py-0.5 text-[10px] text-[#a1a1aa] font-medium">Esc</span>
          </button>
        </div>
      </div>
      
    </div>
  </div>
</div>
