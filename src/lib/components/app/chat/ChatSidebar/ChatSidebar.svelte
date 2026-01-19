<script>
  import { chatStore } from "$lib/stores/chat.svelte";
  /** @type {{ isSidebarOpen: boolean }} */
  let { isSidebarOpen } = $props();
</script>

<aside class="sidebar" class:closed={!isSidebarOpen}>
  <div class="sidebar-inner">
    <div class="sidebar-header">
      <button
        class="new-chat-btn"
        onclick={() => chatStore.clear()}
        title="Start new conversation"
      >
        <h1 class="title">llama.cpp</h1>
      </button>
    </div>

    <div class="sidebar-content scrollbar-hide">
      <div class="section-label">Conversations</div>
      <div class="history-list">
        <div class="history-item active">Current Conversation</div>
        <div class="history-item">Recent Chat 1</div>
        <div class="history-item">Recent Chat 2</div>
      </div>
    </div>

    <div class="sidebar-footer">
      <nav class="footer-nav">
        <a href="/models" class="nav-item">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><path
              d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
            ></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"
            ></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg
          >
          <span>Models</span>
        </a>
        <a href="/settings" class="nav-item">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><circle cx="12" cy="12" r="3"></circle><path
              d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
            ></path></svg
          >
          <span>Settings</span>
        </a>
      </nav>
    </div>
  </div>
</aside>

<style>
  .sidebar {
    width: 260px;
    background-color: var(--sidebar);
    border-right: 1px solid var(--sidebar-border);
    display: flex;
    flex-direction: column;
    height: 100vh;
    transition:
      width 0.3s cubic-bezier(0.4, 0, 0.2, 1),
      transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 100;
    overflow: hidden;
  }

  .sidebar-inner {
    width: 260px;
    height: 100%;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar.closed {
    width: 0;
    border-right-width: 0;
    transform: translateX(-260px);
  }

  .sidebar-header {
    padding: 1.5rem 1rem 1rem;
  }

  .new-chat-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
    padding: 0 0.5rem;
  }

  .title {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--sidebar-foreground);
    margin: 0;
  }

  .sidebar-content {
    flex-grow: 1;
    overflow-y: auto;
    padding: 1rem 0.5rem;
  }

  .section-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--muted-foreground);
    padding: 0.5rem 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .history-item {
    padding: 0.625rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--sidebar-foreground);
    cursor: pointer;
    transition: background 0.2s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .history-item:hover {
    background-color: var(--sidebar-accent);
  }

  .history-item.active {
    background-color: var(--sidebar-accent);
    font-weight: 500;
  }

  .sidebar-footer {
    padding: 1rem 0.5rem;
    border-top: 1px solid var(--sidebar-border);
  }

  .footer-nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0.625rem 0.75rem;
    border-radius: 0.5rem;
    color: var(--sidebar-foreground);
    text-decoration: none;
    font-size: 0.875rem;
    transition: background 0.2s;
  }

  .nav-item:hover {
    background-color: var(--sidebar-accent);
  }
</style>
