<script>
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  import { Toaster } from "svelte-sonner";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";
  import { onMount } from "svelte";

  let { children } = $props();

  onMount(async () => {
    await settingsStore.init();
    await modelsStore.refresh();
  });
</script>

<Toaster richColors position="bottom-right" />
<ModeWatcher />
{@render children()}
