<script>
  import { chatStore } from "$lib/stores/chat.svelte";
  import { goto } from "$app/navigation";
  import { cn } from "$lib/utils/cn.js";
  import {
    Search,
    SquarePen,
    PanelLeftClose,
    PanelLeftOpen,
    Image,
    Grid3x3,
    FolderOpen,
    User,
    ChevronDown,
    Ellipsis,
    Sparkle,
    Settings,
    HelpCircle,
    LogOut,
    MessageSquare,
    Box,
  } from "lucide-svelte";
  import { page } from "$app/state";

  /** @type {{ isSidebarOpen: boolean, toggleSidebar: () => void }} */
  let { isSidebarOpen, toggleSidebar } = $props();

  let activeMenu = $state("imagens");
  let showChatHistory = $state(true);
  let showProfileMenu = $state(false);

  const menuItems = [
    { id: "chat", label: "Chat", icon: MessageSquare, path: "/" },
    { id: "models", label: "Modelos", icon: Box, path: "/models" },
    {
      id: "settings",
      label: "Configurações",
      icon: Settings,
      path: "/settings",
    },
  ];

  const chatHistory = [
    { id: "1", title: "Llama cpp", active: false },
    { id: "2", title: "Estrutura Tauri e Rust", active: false },
    { id: "3", title: "Erro DISM pacote permanente", active: false },
    { id: "4", title: "Remover KB5074109 Passo a...", active: false },
    { id: "5", title: "Erro no comando DISM", active: false },
    { id: "6", title: "Direto ao ponto", active: false },
    { id: "7", title: "Start Menu e Electron", active: false },
    { id: "8", title: "Erro 0x800F0825 Windows", active: false },
  ];

  function handleMenuClick(item) {
    if (item.id === "chat" && page.url.pathname === "/") {
      chatStore.clear();
    } else {
      goto(item.path);
    }
  }

  function toggleChatHistory() {
    showChatHistory = !showChatHistory;
  }

  function toggleProfileMenu() {
    showProfileMenu = !showProfileMenu;
  }

  function handleProfileMenuClick(/** @type {string} */ action) {
    if (action === "settings") {
      goto("/settings");
    } else {
      console.log("Profile action:", action);
    }
    showProfileMenu = false;
  }
</script>

<aside
  class={cn(
    "z-[100] flex h-screen shrink-0 flex-col overflow-hidden bg-sidebar transition-[width,padding] duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]",
    isSidebarOpen ? "w-[260px] p-0" : "w-[60px] p-0",
  )}
>
  <div
    class={cn(
      "flex h-full flex-col transition-[width,padding] duration-300",
      isSidebarOpen ? "w-[260px] p-4 px-3" : "w-[60px] px-1 py-4",
    )}
  >
    <div
      class={cn(
        "flex min-h-[48px] items-center gap-1 p-1 transition-[margin-bottom] duration-300",
        isSidebarOpen
          ? "mb-8 flex-row justify-start"
          : "mb-4 flex-col justify-center gap-2",
      )}
    >
      <button
        class="flex cursor-pointer items-center justify-center rounded-lg bg-transparent p-2 text-[#b4b4b4] transition-all duration-200 hover:bg-[#2f2f2f] hover:text-white border-none"
        onclick={toggleSidebar}
        title={isSidebarOpen ? "Close sidebar" : "Open sidebar"}
      >
        {#if isSidebarOpen}
          <PanelLeftClose size={22} strokeWidth={1.5} />
        {:else}
          <PanelLeftOpen size={22} strokeWidth={1.5} />
        {/if}
      </button>
    </div>

    <div
      class={cn(
        "scrollbar-hide flex grow flex-col gap-6 overflow-y-auto px-1 -mx-1 transition-opacity duration-300",
        !isSidebarOpen && "hidden",
      )}
    >
      <nav class="flex flex-col gap-2">
        {#each menuItems as item}
          <button
            class={cn(
              "flex w-full cursor-pointer items-center gap-3 rounded-lg border-none bg-transparent p-3 px-4 text-left text-[0.95rem] font-medium text-[#ececec] transition-colors duration-200 hover:bg-[#2f2f2f]",
              page.url.pathname === item.path && "bg-[#3f3f3f] text-white",
            )}
            onclick={() => handleMenuClick(item)}
            title={item.label}
          >
            <div class="flex shrink-0 items-center justify-center text-inherit">
              {#if item.icon}
                <item.icon size={20} strokeWidth={1.5} />
              {/if}
            </div>
            <span class="overflow-hidden text-ellipsis whitespace-nowrap"
              >{item.label}</span
            >
          </button>
        {/each}

        <div class="flex flex-col gap-2">
          <button
            class="flex cursor-pointer items-center justify-between border-none bg-transparent p-2 px-3 text-sm font-medium text-[#999] transition-colors duration-200 hover:text-[#ccc]"
            onclick={toggleChatHistory}
            title={showChatHistory
              ? "Collapse chat history"
              : "Expand chat history"}
          >
            <span class="whitespace-nowrap">Seus chats</span>
            <div
              class={cn(
                "flex shrink-0 items-center justify-center transition-transform duration-300",
                showChatHistory && "rotate-180",
              )}
            >
              <ChevronDown size={18} strokeWidth={1.5} />
            </div>
          </button>

          {#if showChatHistory}
            <div class="flex flex-col gap-1">
              {#each chatHistory as chat}
                <button
                  class={cn(
                    "relative flex w-full cursor-pointer items-center justify-between overflow-hidden rounded-lg border-none bg-transparent p-2.5 px-3 text-left text-sm text-[#ececec] transition-colors duration-200 hover:bg-[#2f2f2f]",
                    chat.active && "bg-[#3f3f3f] text-white font-medium",
                  )}
                  title={chat.title}
                >
                  <span
                    class="mr-2 grow overflow-hidden text-ellipsis whitespace-nowrap"
                    >{chat.title}</span
                  >
                  <div
                    class="absolute right-2 flex items-center opacity-0 transition-opacity duration-200 bg-gradient-to-r from-transparent via-[#2f2f2f] to-[#2f2f2f] pl-4 group-hover:opacity-100"
                  >
                    <div
                      class="flex h-6 w-6 items-center justify-center rounded hover:bg-[#3f3f3f]"
                      role="button"
                      tabindex="0"
                      onclick={(e) => e.stopPropagation()}
                      onkeydown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                          e.stopPropagation();
                        }
                      }}
                      title="Chat options"
                    >
                      <Ellipsis size={16} />
                    </div>
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </nav>
    </div>

    <div
      class={cn(
        "relative mt-auto border-t border-[#2f2f2f] pt-4 transition-[padding] duration-300",
        !isSidebarOpen && "border-t-0 pt-2",
      )}
    >
      <div class="relative">
        <button
          class="w-full cursor-pointer rounded-lg bg-transparent p-0 transition-colors duration-200 hover:bg-[#2f2f2f] border-none focus-visible:outline-2 focus-visible:outline-[#0084ff] focus-visible:outline-offset-2"
          onclick={toggleProfileMenu}
          title="User menu"
        >
          <div
            class={cn(
              "flex items-center gap-3 rounded-lg p-2.5 px-3 transition-colors duration-200",
              !isSidebarOpen && "justify-center px-1 py-2",
            )}
          >
            <div class="flex items-center text-[#b4b4b4]">
              <User size={24} strokeWidth={1.5} />
            </div>
            <span
              class={cn(
                "text-sm font-medium text-[#ececec] transition-opacity duration-300",
                !isSidebarOpen && "hidden",
              )}>Antigravity User</span
            >
          </div>
        </button>

        {#if showProfileMenu}
          <div
            class="absolute bottom-full left-0 right-0 z-[1000] mb-2 min-w-[240px] animate-in fade-in slide-in-from-bottom-2 rounded-xl border border-[#3f3f3f] bg-[#2a2a2a] shadow-[0_10px_25px_rgba(0,0,0,0.5)]"
          >
            <div
              class="flex items-center gap-3 border-b border-[#3f3f3f] p-3 px-4"
            >
              <div
                class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-[#3f3f3f] text-[#b4b4b4]"
              >
                <User size={20} strokeWidth={1.5} />
              </div>
              <div class="flex min-w-0 flex-col gap-1">
                <div
                  class="overflow-hidden text-ellipsis whitespace-nowrap text-sm font-semibold text-[#ececec]"
                >
                  Kayky Vitor
                </div>
                <div
                  class="overflow-hidden text-ellipsis whitespace-nowrap text-[0.75rem] text-[#999]"
                >
                  @kaykyvitorgp
                </div>
              </div>
            </div>

            <div class="my-2 h-px bg-[#3f3f3f]"></div>

            <button
              class="flex w-full cursor-pointer items-center gap-3 border-none bg-transparent p-3 px-4 text-left text-sm font-medium text-[#ececec] transition-colors duration-200 hover:bg-[#3f3f3f]"
              onclick={() => handleProfileMenuClick("upgrade")}
            >
              <Sparkle size={18} strokeWidth={1.5} />
              <span>Fazer upgrade do plano</span>
            </button>

            <button
              class="flex w-full cursor-pointer items-center gap-3 border-none bg-transparent p-3 px-4 text-left text-sm font-medium text-[#ececec] transition-colors duration-200 hover:bg-[#3f3f3f]"
              onclick={() => handleProfileMenuClick("customize")}
            >
              <Settings size={18} strokeWidth={1.5} />
              <span>Personalização</span>
            </button>

            <button
              class="flex w-full cursor-pointer items-center gap-3 border-none bg-transparent p-3 px-4 text-left text-sm font-medium text-[#ececec] transition-colors duration-200 hover:bg-[#3f3f3f]"
              onclick={() => handleProfileMenuClick("settings")}
            >
              <Settings size={18} strokeWidth={1.5} />
              <span>Configurações</span>
            </button>

            <div class="my-2 h-px bg-[#3f3f3f]"></div>

            <button
              class="flex w-full cursor-pointer items-center gap-3 border-none bg-transparent p-3 px-4 text-left text-sm font-medium text-[#ececec] transition-colors duration-200 hover:bg-[#3f3f3f]"
              onclick={() => handleProfileMenuClick("help")}
            >
              <HelpCircle size={18} strokeWidth={1.5} />
              <span>Ajuda</span>
            </button>

            <button
              class="flex w-full cursor-pointer items-center gap-3 border-none bg-transparent p-3 px-4 text-left text-sm font-medium text-[#ff6b6b] transition-colors duration-200 hover:bg-red-500/10"
              onclick={() => handleProfileMenuClick("logout")}
            >
              <LogOut size={18} strokeWidth={1.5} />
              <span>Sair</span>
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>
</aside>
