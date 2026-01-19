<script>
  import { onMount, tick } from "svelte";
  import { invokeCommand } from "$lib/ipc";

  let messages = $state([
    { role: "assistant", content: "Hello! How can I help you today?" },
  ]);
  let userInput = $state("");
  let isSidebarOpen = $state(true);
  let messagesEnd = $state();
  let textarea = $state();

  $effect(() => {
    if (messages.length) {
      scrollToBottom();
    }
  });

  async function scrollToBottom() {
    await tick();
    if (messagesEnd) {
      messagesEnd.scrollIntoView({ behavior: "smooth" });
    }
  }

  async function sendMessage() {
    if (!userInput.trim()) return;

    const userMessage = { role: "user", content: userInput };
    messages = [...messages, userMessage];
    const currentInput = userInput;
    userInput = "";
    resetTextareaHeight();

    // Simulate assistant response or call backend
    // greetMsg = await invokeCommand("greet", { name: userInput });

    // For now, let's just simulate a response
    setTimeout(async () => {
      messages = [
        ...messages,
        {
          role: "assistant",
          content: `You said: "${currentInput}". This is a mock response in the ChatGPT-style UI.`,
        },
      ];
    }, 500);
  }

  function handleKeydown(e) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function toggleSidebar() {
    isSidebarOpen = !isSidebarOpen;
  }

  function handleInput() {
    if (textarea) {
      textarea.style.height = "auto";
      textarea.style.height = textarea.scrollHeight + "px";
    }
  }

  function resetTextareaHeight() {
    if (textarea) {
      textarea.style.height = "auto";
    }
  }
</script>

<div class="app-container">
  <!-- Sidebar -->
  <aside class="sidebar" class:closed={!isSidebarOpen}>
    <button class="new-chat">
      <span class="plus-icon">+</span> New chat
    </button>

    <div class="chat-history">
      <div class="history-item active">Current Conversation</div>
      <div class="history-item">Yesterday's Chat</div>
      <div class="history-item">Project Ideas</div>
      <div class="history-item">Recipe: Chocolate Cake</div>
    </div>

    <div class="user-profile">
      <div class="avatar">U</div>
      <span>User Account</span>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    <header class="chat-header">
      <button class="menu-toggle" onclick={toggleSidebar}>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><line x1="3" y1="12" x2="21" y2="12"></line><line
            x1="3"
            y1="6"
            x2="21"
            y2="6"
          ></line><line x1="3" y1="18" x2="21" y2="18"></line></svg
        >
      </button>
      <div class="model-info">Llama 3.1</div>
    </header>

    <div class="messages-container">
      {#each messages as msg}
        <div class="message-row {msg.role}">
          <div class="message-wrapper">
            <div class="avatar-small">{msg.role === "user" ? "U" : "AI"}</div>
            <div class="message-content">
              {msg.content}
            </div>
          </div>
        </div>
      {/each}
      <div bind:this={messagesEnd}></div>
    </div>

    <footer class="input-area">
      <div class="input-container">
        <textarea
          placeholder="Message ChatGPT..."
          bind:value={userInput}
          bind:this={textarea}
          onkeydown={handleKeydown}
          oninput={handleInput}
          rows="1"
        ></textarea>
        <button
          class="send-button"
          onclick={sendMessage}
          disabled={!userInput.trim()}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><line x1="22" y1="2" x2="11" y2="13"></line><polygon
              points="22 2 15 22 11 13 2 9 22 2"
            ></polygon></svg
          >
        </button>
      </div>
      <p class="disclaimer">ChatGPT can make mistakes. Check important info.</p>
    </footer>
  </main>
</div>

<style>
  .app-container {
    display: flex;
    height: 100vh;
    width: 100vw;
  }

  .sidebar {
    width: 260px;
    background-color: var(--color-bg-sidebar);
    display: flex;
    flex-direction: column;
    padding: 8px;
    transition:
      transform 0.3s ease,
      width 0.3s ease;
    flex-shrink: 0;
  }

  .sidebar.closed {
    width: 0;
    padding: 0;
    overflow: hidden;
    transform: translateX(-100%);
  }

  .new-chat {
    background: transparent;
    border: 1px solid var(--color-border);
    color: white;
    padding: 12px;
    border-radius: 6px;
    text-align: left;
    cursor: pointer;
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
    transition: background 0.2s;
  }

  .new-chat:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  .plus-icon {
    font-size: 18px;
  }

  .chat-history {
    flex-grow: 1;
    overflow-y: auto;
  }

  .history-item {
    padding: 12px;
    border-radius: 6px;
    font-size: 14px;
    color: var(--color-text-secondary);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
  }

  .history-item:hover {
    background-color: #2a2b32;
    color: white;
  }

  .history-item.active {
    background-color: #343541;
    color: white;
  }

  .user-profile {
    border-top: 1px solid var(--color-border);
    padding-top: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
    color: white;
    font-size: 14px;
  }

  .avatar {
    width: 32px;
    height: 32px;
    background-color: #5436da;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
  }

  .main-content {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-primary);
    position: relative;
  }

  .chat-header {
    height: 60px;
    display: flex;
    align-items: center;
    padding: 0 16px;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-text-secondary);
  }

  .menu-toggle {
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 8px;
    margin-right: 12px;
    border-radius: 4px;
  }

  .menu-toggle:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  .model-info {
    font-weight: 500;
    font-size: 16px;
    color: white;
  }

  .messages-container {
    flex-grow: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .message-row {
    padding: 24px 0;
  }

  .message-row.assistant {
    background-color: var(--color-bg-secondary);
  }

  .message-wrapper {
    max-width: 768px;
    margin: 0 auto;
    display: flex;
    gap: 24px;
    padding: 0 24px;
  }

  .avatar-small {
    width: 30px;
    height: 30px;
    border-radius: 2px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: bold;
  }

  .user .avatar-small {
    background-color: #5436da;
    color: white;
  }

  .assistant .avatar-small {
    background-color: var(--color-accent);
    color: white;
  }

  .message-content {
    line-height: 1.6;
    font-size: 16px;
    color: var(--color-text-primary);
    white-space: pre-wrap;
  }

  .input-area {
    padding: 24px 0 48px;
    background: linear-gradient(transparent, var(--color-bg-primary) 50%);
  }

  .input-container {
    max-width: 768px;
    margin: 0 auto;
    position: relative;
    background-color: var(--color-input-bg);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    padding: 10px 14px;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    display: flex;
    align-items: flex-end;
  }

  textarea {
    flex-grow: 1;
    background: transparent;
    border: none;
    color: white;
    resize: none;
    font-size: 16px;
    max-height: 200px;
    outline: none;
    padding: 4px 0;
    font-family: inherit;
  }

  .send-button {
    background-color: var(--color-accent);
    color: white;
    border: none;
    border-radius: 6px;
    padding: 6px;
    cursor: pointer;
    transition: background-color 0.2s;
    margin-left: 8px;
  }

  .send-button:disabled {
    background-color: transparent;
    color: #565869;
    cursor: not-allowed;
  }

  .send-button:not(:disabled):hover {
    background-color: var(--color-accent-hover);
  }

  .disclaimer {
    text-align: center;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.5);
    margin-top: 12px;
  }
</style>
