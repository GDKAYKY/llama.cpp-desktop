<script>
  import { tick } from "svelte";
  import {
    ChatSidebar,
    ChatHeader,
    ChatMessages,
    ChatForm,
  } from "$lib/components/app";
  import { chatStore } from "$lib/stores/chat.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";

  let userInput = $state("");
  let isSidebarOpen = $state(true);
  let messagesEnd = $state();
  let textarea = $state();
  let isDropdownOpen = $state(false);

  $effect(() => {
    if (chatStore.messages.length) {
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
    if (!userInput.trim() || chatStore.isLoading) return;

    const currentInput = userInput;
    userInput = "";
    resetTextareaHeight();

    await chatStore.send(currentInput);
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

  function toggleDropdown(e) {
    if (e) e.stopPropagation();
    isDropdownOpen = !isDropdownOpen;
  }

  async function selectModel(model) {
    modelsStore.selectModel(model);
    isDropdownOpen = false;
    await chatStore.loadModel();
  }

  function handleClickOutside(e) {
    const dropdown = document.querySelector(".model-dropdown");
    const selector = document.querySelector(".model-selector-btn");

    if (dropdown && selector && e.target instanceof Node) {
      if (!dropdown.contains(e.target) && !selector.contains(e.target)) {
        isDropdownOpen = false;
      }
    }
  }

  let isEmpty = $derived(
    chatStore.messages.length === 0 && !chatStore.isLoading,
  );
</script>

<div class="app-container">
  <ChatSidebar {isSidebarOpen} />

  <main class="main-content">
    <ChatHeader
      {isSidebarOpen}
      {toggleSidebar}
      isLoading={chatStore.isLoading || chatStore.isModelLoading}
      {toggleDropdown}
      selectedModel={modelsStore.selectedModel}
      {isDropdownOpen}
      models={modelsStore.models}
      {selectModel}
      {handleClickOutside}
      modelLoaded={chatStore.modelLoaded}
      llama_cpp_path={settingsStore.settings.llamaPath}
    />

    {#if isEmpty}
      <div class="welcome-screen">
        <div class="welcome-container">
          <h1 class="welcome-title">llama.cpp</h1>
          <p class="welcome-subtitle">
            Type a message or upload files to get started
          </p>

          <div class="welcome-form-wrapper">
            <ChatForm
              bind:userInput
              modelLoaded={chatStore.modelLoaded}
              isLoading={chatStore.isLoading}
              onKeydown={handleKeydown}
              onInput={handleInput}
              onSend={sendMessage}
              bind:textarea
            />
          </div>
        </div>
      </div>
    {:else}
      <ChatMessages
        messages={chatStore.messages}
        isLoading={chatStore.isLoading}
        bind:messagesEnd
      />

      <div class="chat-form-wrapper">
        <ChatForm
          bind:userInput
          modelLoaded={chatStore.modelLoaded}
          isLoading={chatStore.isLoading}
          onKeydown={handleKeydown}
          onInput={handleInput}
          onSend={sendMessage}
          bind:textarea
        />
      </div>
    {/if}
  </main>
</div>

<style>
  .app-container {
    display: flex;
    height: 100vh;
    width: 100vw;
  }

  .main-content {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--background);
    position: relative;
    overflow: hidden;
  }

  .welcome-screen {
    flex-grow: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
  }

  .welcome-container {
    width: 100%;
    max-width: 768px;
    text-align: center;
  }

  .welcome-title {
    font-size: 2rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: var(--foreground);
    letter-spacing: -0.025em;
  }

  .welcome-subtitle {
    font-size: 1.125rem;
    color: var(--muted-foreground);
    margin-bottom: 2.5rem;
  }

  .welcome-form-wrapper {
    width: 100%;
  }

  .chat-form-wrapper {
    position: sticky;
    bottom: 0;
    width: 100%;
    z-index: 10;
  }
</style>
