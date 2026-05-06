<script>
  let {
    value = "",
    placeholder = "Edit message...",
    onCancel = () => {},
    onSubmit = () => {},
  } = $props();
</script>

<div class="w-full flex-1 mb-4 relative flex min-w-0 flex-col">
  <div class="bg-secondary/50 border border-border rounded-3xl px-3 py-3">
    <div class="m-2 max-h-[25vh] overflow-auto">
      <div class="grid">
        <textarea
          bind:value
          class="col-start-1 col-end-2 row-start-1 row-end-2 w-full resize-none overflow-hidden p-0 m-0 border-0 bg-transparent text-foreground outline-none focus:ring-0"
          {placeholder}
          onkeydown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              onSubmit();
            }
            if (e.key === "Escape") {
              onCancel();
            }
          }}
        ></textarea>
        <span
          class="invisible col-start-1 col-end-2 row-start-1 row-end-2 p-0 break-all whitespace-pre-wrap"
          >{value}</span
        >
      </div>
    </div>
    <div class="flex justify-end gap-2 mt-2">
      <button
        class="px-4 py-2 rounded-full text-sm font-medium hover:bg-white/5 transition-colors"
        onclick={onCancel}
      >
        Cancelar
      </button>
      <button
        class="px-4 py-2 rounded-full text-sm font-medium bg-foreground text-background hover:opacity-90 transition-opacity"
        onclick={onSubmit}
      >
        Enviar
      </button>
    </div>
  </div>
</div>
