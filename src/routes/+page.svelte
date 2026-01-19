<script>
  import { tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { loadModelLibrary } from "$lib/models.js";

  let messages = $state([
    { role: "assistant", content: "Hello! How can I help you today?" },
  ]);
  let userInput = $state("");
  let isSidebarOpen = $state(true);
  let messagesEnd = $state();
  let textarea = $state();
  /** @type {Array<{name: string, full_identifier: string, model_file_path: string}>} */
  let models = $state([]);
  let selectedModel = $state(null);
  let isDropdownOpen = $state(false);
  let libraryPath = $state("");
  let isLoading = $state(false);
  let modelLoaded = $state(false);
  let llama_cpp_path = $state("");

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
    if (!userInput.trim() || !modelLoaded) return;

    const userMessage = { role: "user", content: userInput };
    messages = [...messages, userMessage];
    const currentInput = userInput;
    userInput = "";
    resetTextareaHeight();

    try {
      isLoading = true;
      const response = await invoke("send_message", {
        message: currentInput,
      });

      messages = [
        ...messages,
        {
          role: "assistant",
          content: response,
        },
      ];
    } catch (error) {
      messages = [
        ...messages,
        {
          role: "assistant",
          content: `Error: ${error}`,
        },
      ];
    } finally {
      isLoading = false;
    }
  }

  /**
   * @param {KeyboardEvent} e
   */
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

  async function loadModels() {
    try {
      const { loadConfig } = await import("$lib/config.js");
      const config = await loadConfig();
      const modelsDir = config.models_directory || "";
      llama_cpp_path = config.llamaPath || "";
      if (modelsDir) {
        libraryPath = `${modelsDir}/modelLibrary.json`;
        const loadedModels = await loadModelLibrary(libraryPath);
        if (loadedModels && loadedModels.length > 0) {
          models = loadedModels.map((m) => ({
            name: `${m.name}:${m.version}`,
            full_identifier: m.full_identifier,
            model_file_path: m.model_file_path,
          }));
          if (models.length > 0) {
            selectedModel = models[0];
          }
        }
      }
    } catch (err) {
      console.error("Failed to load models:", err);
    }
  }

  function toggleDropdown() {
    isDropdownOpen = !isDropdownOpen;
  }

  /**
   * @param {any} model
   */
  async function selectModel(model) {
    selectedModel = model;
    isDropdownOpen = false;
    await loadModel();
  }

  async function loadModel() {
    if (!selectedModel || !selectedModel.model_file_path) {
      console.error("No model selected or model path missing");
      return;
    }

    if (!llama_cpp_path) {
      messages = [
        ...messages,
        {
          role: "system",
          content:
            "Error: llama.cpp path not configured. Please set it in Settings.",
        },
      ];
      return;
    }

    try {
      isLoading = true;
      await invoke("init_llama", {
        llamaPath: llama_cpp_path,
        modelPath: selectedModel.model_file_path,
      });
      modelLoaded = true;
      messages = [
        ...messages,
        {
          role: "system",
          content: `Model "${selectedModel.name}" loaded successfully`,
        },
      ];
    } catch (error) {
      messages = [
        ...messages,
        {
          role: "system",
          content: `Failed to load model: ${error}`,
        },
      ];
    } finally {
      isLoading = false;
    }
  }

  /**
   * @param {MouseEvent} e
   */
  function handleClickOutside(e) {
    const dropdown = document.querySelector(".model-dropdown");
    if (dropdown && e.target instanceof Node && !dropdown.contains(e.target)) {
      isDropdownOpen = false;
    }
  }

  $effect(() => {
    loadModels();
  });
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

    <div class="sidebar-footer">
      <a href="/models" class="nav-link">
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
        >
          <path
            d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
          ></path>
          <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
          <line x1="12" y1="22.08" x2="12" y2="12"></line>
        </svg>
        Models
      </a>
      <a href="/settings" class="nav-link">
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
        >
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M12 1v6m0 6v6m-9-9h6m6 0h6"></path>
          <path
            d="m4.93 4.93 4.24 4.24m5.66 5.66 4.24 4.24m0-14.14-4.24 4.24m-5.66 5.66-4.24 4.24"
          ></path>
        </svg>
        Settings
      </a>
    </div>

    <div class="user-profile">
      <div class="avatar">U</div>
      <span>User Account</span>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    <header class="chat-header">
      <button
        class="menu-toggle"
        onclick={toggleSidebar}
        aria-label="Toggle sidebar"
      >
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
      <div class="model-dropdown">
        <button
          class="model-selector-btn"
          onclick={toggleDropdown}
          aria-label="Select model"
          disabled={isLoading}
        >
          {selectedModel ? selectedModel.name : "Select a model"}
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
            class:rotated={isDropdownOpen}
          >
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </button>
        {#if isDropdownOpen}
          <div
            class="dropdown-menu"
            role="listbox"
            tabindex="0"
            onmousedown={handleClickOutside}
          >
            {#if models.length > 0}
              {#each models as model}
                <button
                  class="dropdown-item"
                  class:active={selectedModel?.name === model.name}
                  onclick={() => selectModel(model)}
                  role="option"
                  aria-selected={selectedModel?.name === model.name}
                >
                  {model.name}
                </button>
              {/each}
            {:else}
              <div class="dropdown-empty">No models found</div>
            {/if}
          </div>
        {/if}
      </div>
      {#if modelLoaded}
        <div class="model-status">
          <span class="status-dot"></span>
          Model loaded
        </div>
      {/if}
      {#if !llama_cpp_path}
        <div class="warning-status">⚠ llama.cpp path not set</div>
      {/if}
    </header>

    <div class="messages-container">
      {#each messages as msg}
        <div class="message-row {msg.role}">
          <div class="message-wrapper">
            <div class="avatar-small">
              {msg.role === "user" ? "U" : msg.role === "system" ? "⚙" : "AI"}
            </div>
            <div class="message-content">
              {msg.content}
            </div>
          </div>
        </div>
      {/each}
      {#if isLoading}
        <div class="message-row assistant">
          <div class="message-wrapper">
            <div class="avatar-small">AI</div>
            <div class="message-content typing">
              <span></span><span></span><span></span>
            </div>
          </div>
        </div>
      {/if}
      <div bind:this={messagesEnd}></div>
    </div>

    <footer class="input-area">
      <div class="input-container">
        <textarea
          placeholder={modelLoaded ? "Message..." : "Load a model first..."}
          bind:value={userInput}
          bind:this={textarea}
          onkeydown={handleKeydown}
          oninput={handleInput}
          rows="1"
          disabled={!modelLoaded || isLoading}
        ></textarea>
        <button
          class="send-button"
          onclick={sendMessage}
          disabled={!userInput.trim() || !modelLoaded || isLoading}
          aria-label="Send message"
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
      <p class="disclaimer">Local LLM powered by llama.cpp</p>
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

  .sidebar-footer {
    border-top: 1px solid var(--color-border);
    padding-top: 8px;
    margin-bottom: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 6px;
    color: var(--color-text-secondary);
    text-decoration: none;
    font-size: 14px;
    transition: all 0.2s;
  }

  .nav-link:hover {
    background-color: #2a2b32;
    color: white;
  }

  .nav-link svg {
    flex-shrink: 0;
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
    gap: 16px;
  }

  .menu-toggle {
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 8px;
    border-radius: 4px;
  }

  .menu-toggle:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }

  .model-dropdown {
    position: relative;
  }

  .model-selector-btn {
    background: transparent;
    border: 1px solid var(--color-border);
    color: white;
    padding: 8px 12px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
    transition: background-color 0.2s;
  }

  .model-selector-btn:hover:not(:disabled) {
    background-color: rgba(255, 255, 255, 0.1);
  }

  .model-selector-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .model-selector-btn svg {
    transition: transform 0.2s;
  }

  .model-selector-btn svg.rotated {
    transform: rotate(180deg);
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    background-color: var(--color-bg-sidebar);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    margin-top: 4px;
    min-width: 200px;
    max-height: 300px;
    overflow-y: auto;
    z-index: 1000;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 10px 12px;
    background: transparent;
    border: none;
    color: var(--color-text-secondary);
    text-align: left;
    cursor: pointer;
    font-size: 14px;
    transition: background-color 0.2s;
  }

  .dropdown-item:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .dropdown-item.active {
    background-color: #343541;
    color: white;
    font-weight: 500;
  }

  .dropdown-empty {
    padding: 10px 12px;
    color: var(--color-text-secondary);
    font-size: 14px;
    text-align: center;
  }

  .model-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: #10b981;
    margin-left: auto;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    background-color: #10b981;
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  .warning-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: #f59e0b;
    margin-left: auto;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
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

  .message-row.system {
    background-color: rgba(59, 130, 246, 0.1);
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

  .system .avatar-small {
    background-color: #3b82f6;
    color: white;
  }

  .message-content {
    line-height: 1.6;
    font-size: 16px;
    color: var(--color-text-primary);
    white-space: pre-wrap;
  }

  .message-content.typing {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .message-content.typing span {
    width: 8px;
    height: 8px;
    background-color: var(--color-accent);
    border-radius: 50%;
    animation: bounce 1.4s infinite;
  }

  .message-content.typing span:nth-child(2) {
    animation-delay: 0.2s;
  }

  .message-content.typing span:nth-child(3) {
    animation-delay: 0.4s;
  }

  @keyframes bounce {
    0%,
    80%,
    100% {
      opacity: 0.5;
      transform: translateY(0);
    }
    40% {
      opacity: 1;
      transform: translateY(-10px);
    }
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

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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
