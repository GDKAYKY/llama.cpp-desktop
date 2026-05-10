<script>
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { chatStore } from "$lib/stores/chat.svelte";
  import { onMount } from "svelte";
  import ChatSidebar from "$components/layout/sidebar/ChatSidebar.svelte";
  import Notification from "$components/ui/models/Notification.svelte";
  import { notificationState } from "$lib/shared/notifications";

  let { children } = $props();

  onMount(async () => {
    await settingsStore.init();
    await modelsStore.refresh();
    await chatStore.initialize();
  });
</script>

<ModeWatcher />

{#if $notificationState.visible}
  <Notification
    message={$notificationState.message}
    loading={$notificationState.loading}
  />
{/if}

<div
  class="flex h-screen w-screen bg-background text-foreground overflow-hidden"
>
  <ChatSidebar
    isSidebarOpen={uiStore.isSidebarOpen}
    toggleSidebar={() => uiStore.toggleSidebar()}
  />
  <main class="relative flex grow flex-col overflow-y-auto bg-background">
    {@render children()}
  </main>
</div>
