<script lang="ts">
  import { ShieldAlert, Check, X } from "lucide-svelte";
  import type { PendingPermission } from "$lib/stores/chat.svelte";

  let { permission } = $props<{ permission: PendingPermission }>();
</script>

<div class="pointer-events-auto mb-4 flex w-full max-w-2xl flex-col overflow-hidden rounded-lg border border-[#3f3f46] bg-[#1e1e1e] shadow-lg">
  <div class="flex items-center gap-3 border-b border-[#3f3f46] bg-[#27272a] px-4 py-3">
    <ShieldAlert size={18} class="text-amber-500" />
    <h3 class="text-sm font-semibold text-[#e5e5e5]">Permissão Necessária</h3>
  </div>
  <div class="p-4 text-sm text-[#a1a1aa]">
    <p class="mb-3">
      O servidor MCP <strong class="text-white">{permission.serverId}</strong> está solicitando permissão para executar a ferramenta:
    </p>
    <div class="mb-4 rounded-md bg-[#18181b] p-3 font-mono text-xs text-[#e5e5e5] border border-[#27272a] max-h-[200px] overflow-auto">
      <div class="font-bold text-purple-400 mb-1">{permission.toolName}</div>
      <pre class="whitespace-pre-wrap">{JSON.stringify(permission.args, null, 2)}</pre>
    </div>
    <div class="flex justify-end gap-2">
      <button 
        class="flex items-center gap-2 rounded-md border border-[#3f3f46] bg-[#27272a] px-4 py-2 text-sm font-medium text-[#e5e5e5] transition-colors hover:bg-[#3f3f46]"
        onclick={() => permission.resolve(false)}
      >
        <X size={16} />
        Negar
      </button>
      <button 
        class="flex items-center gap-2 rounded-md bg-purple-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-purple-700"
        onclick={() => permission.resolve(true)}
      >
        <Check size={16} />
        Permitir
      </button>
    </div>
  </div>
</div>
