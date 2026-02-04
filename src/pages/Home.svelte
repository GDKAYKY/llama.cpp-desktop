<script lang="ts">
  import { tick, onMount } from "svelte";
  import ChatHeader from "$components/layout/ChatHeader.svelte";
  import ChatMessages from "$components/chat/ChatMessages.svelte";
  import ChatForm from "$components/chat/ChatForm.svelte";
  import { toast } from "svelte-sonner";
  import { chatStore } from "$lib/stores/chat.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { serverStore } from "$lib/stores/server.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { mcpStore } from "$lib/stores/mcp.svelte";

  let userInput = $state("");
  let messagesEnd: any = $state();
  let textarea: any = $state();
  let isDropdownOpen = $state(false);

  $effect(() => {
    if (chatStore.messages.length) {
      scrollToBottom();
    }
  });

  onMount(async () => {
    await mcpStore.init();
  });

  async function scrollToBottom() {
    await tick();
    if (messagesEnd) {
      messagesEnd.scrollIntoView({ behavior: "smooth" });
    }
  }

  async function sendMessage() {
    if (!userInput.trim() || chatStore.isLoading) return;

    if (!serverStore.isRunning) {
      toast.error("Server is not running. Please select a model to start.");
      return;
    }

    const currentInput = userInput;
    userInput = "";
    resetTextareaHeight();

    await chatStore.send(currentInput);

    if (chatStore.error) {
      toast.error(`Failed to send message: ${chatStore.error}`);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
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

  function toggleDropdown(e: MouseEvent) {
    if (e) e.stopPropagation();
    isDropdownOpen = !isDropdownOpen;
  }

  async function selectModel(model: any) {
    modelsStore.selectModel(model);
    isDropdownOpen = false;

    // Start server logic
    const binaryPath = settingsStore.settings.llamaDirectory;
    const modelPath = model.model_file_path;
    const port = settingsStore.settings.serverPort;

    if (!binaryPath) {
      toast.error("Llama Server path not configured. Please go to Settings.");
      return;
    }

    if (!modelPath) {
      toast.error("Model path not found for selected model.");
      return;
    }

    try {
      if (
        serverStore.isRunning &&
        serverStore.currentConfig?.model_path === modelPath
      ) {
        toast.success(`Using already running model: ${model.name}`);
        return;
      }

      toast.info("Starting llama-server...");
      if (serverStore.isRunning) {
        await serverStore.stopServer();
      }
      await serverStore.startServer(binaryPath, modelPath, port);

      // Short delay to allow startup
      setTimeout(() => {
        if (serverStore.error) {
          toast.error(`Server failed to start: ${serverStore.error}`);
        } else {
          toast.success(`Server started with ${model.name}`);
        }
      }, 1000);
    } catch (err) {
      toast.error(`Failed to start server: ${err}`);
    }
  }

  function handleClickOutside(e: MouseEvent) {
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

<ChatHeader
  isSidebarOpen={uiStore.isSidebarOpen}
  toggleSidebar={() => uiStore.toggleSidebar()}
  isLoading={chatStore.isLoading}
  {toggleDropdown}
  selectedModel={modelsStore.selectedModel}
  {isDropdownOpen}
  models={modelsStore.models}
  {selectModel}
  {handleClickOutside}
  modelLoaded={chatStore.modelLoaded}
/>

{#if isEmpty}
  <div class="flex grow items-center justify-center p-5">
    <div class="w-full max-w-3xl text-center">
      <h1 class="mb-2 text-3xl font-semibold tracking-tight">llama.cpp</h1>
      <p class="mb-10 text-lg text-muted-foreground">
        Type a message or upload files to get started
      </p>

      <div class="w-full">
        <ChatForm
          bind:userInput
          modelLoaded={serverStore.isRunning}
          isLoading={chatStore.isLoading}
          onKeydown={handleKeydown}
          onInput={handleInput}
          onSend={sendMessage}
          bind:textarea
          selectedModel={modelsStore.selectedModel}
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

  <!-- Floating Chat Form at the bottom of the main scroll container -->
  <div
    class="pointer-events-none sticky bottom-0 z-10 mt-auto flex w-full flex-col items-center pb-2"
    style="background: linear-gradient(to bottom, transparent 40%, #212121 40%)"
  >
    <div class="pointer-events-auto w-full flex flex-col items-center">
      <ChatForm
        bind:userInput
        modelLoaded={serverStore.isRunning}
        isLoading={chatStore.isLoading}
        onKeydown={handleKeydown}
        onInput={handleInput}
        onSend={sendMessage}
        bind:textarea
        selectedModel={modelsStore.selectedModel}
      />
      <p
        class="mt-2 text-center text-[0.75rem] text-muted-foreground opacity-80"
      >
        Llama-desktop can make mistakes. Check important info.
      </p>
    </div>
  </div>
{/if}
