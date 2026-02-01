<script>
  import { goto } from "$app/navigation";
  import { cn } from "$shared/cn.js";
  import {
    PanelLeftClose,
    PanelLeftOpen,
    User,
    ChevronDown,
    Ellipsis,
    Sparkle,
    Settings,
    CircleQuestionMark,
    MessageSquare,
    Box,
    SquarePen,
    Share,
    Users,
    Pencil,
    Pin,
    Archive,
    Trash2,
  } from "lucide-svelte";
  import { DropdownMenu } from "bits-ui";
  import { page } from "$app/state";
  import { chatStore } from "$lib/stores/chat.svelte";

  let { isSidebarOpen, toggleSidebar } = $props();

  let showChatHistory = $state(true);
  let showProfileMenu = $state(false);

  // Track which chat menu is open (by chat ID)
  let activeMenuId = $state(null);

  // Close menus when clicking outside
  function handleGlobalClick(event) {
    // Logic handled by svelte:window or simple backdrop
    activeMenuId = null;
  }

  const menuItems = [
    { id: "chat", label: "Chat", icon: MessageSquare, path: "/" },
    { id: "models", label: "Models", icon: Box, path: "/models" },
    {
      id: "settings",
      label: "Settings",
      icon: Settings,
      path: "/settings",
    },
  ];

  const chatHistory = [
    { id: "1", title: "Llama cpp", active: true },
    { id: "2", title: "Tauri and Rust Structure", active: false },
    { id: "3", title: "DISM permanent package error", active: false },
  ];

  const profileMenuItems = [
    { id: "mcps", label: "MCPS", icon: Sparkle, action: "upgrade" },
    {
      id: "customization",
      label: "Customization",
      icon: Settings,
      action: "customize",
    },
    {
      id: "settings",
      label: "Settings",
      icon: Settings,
      path: "/settings",
    },
    { id: "help", label: "Help", icon: CircleQuestionMark, action: "help" },
  ];

  function toggleChatHistory() {
    showChatHistory = !showChatHistory;
  }

  function toggleProfileMenu() {
    showProfileMenu = !showProfileMenu;
  }

  function handleAction(item) {
    if (item.path) goto(item.path);
    showProfileMenu = false;
    // logic for other actions can be added here
  }
</script>

<svelte:window onclick={handleGlobalClick} />

<aside
  class={cn(
    "z-[100] flex h-screen shrink-0 flex-col overflow-hidden bg-sidebar transition-[width] duration-300",
    isSidebarOpen ? "w-[260px]" : "w-[60px]",
  )}
>
  <div class="flex h-full flex-col">
    <!-- HEADER -->
    <div class="mb-auto shrink-0 pt-2.5 pb-2">
      <div class="flex w-[60px] shrink-0 items-center justify-center">
        <button
          class="flex h-10 w-10 items-center justify-center rounded-lg text-[#b4b4b4] transition-colors hover:bg-[#2f2f2f] hover:text-white"
          onclick={toggleSidebar}
        >
          {#if isSidebarOpen}
            <PanelLeftClose size={20} strokeWidth={1.5} />
          {:else}
            <PanelLeftOpen size={20} strokeWidth={1.5} />
          {/if}
        </button>
      </div>
      <div
        class={cn(
          "overflow-hidden whitespace-nowrap transition-all duration-300",
          isSidebarOpen ? "w-32 opacity-100" : "w-0 opacity-0",
        )}
      ></div>
    </div>

    <!-- TOP NAVIGATION (FIXED) -->
    <nav class="flex shrink-0 flex-col">
      <!-- New Chat Button -->
      <button
        class="group relative flex h-12 w-full cursor-pointer items-center border-none bg-transparent transition-colors duration-200"
        onclick={() => chatStore.clear()}
        title={!isSidebarOpen ? "New Chat" : ""}
      >
        <!-- Background Highlight -->
        <div
          class={cn(
            "absolute transition-all duration-200 rounded-lg z-0",
            isSidebarOpen
              ? "inset-y-1 inset-x-2.5"
              : "inset-y-1 left-[10px] w-10 h-10",
            "bg-transparent group-hover:bg-[#2f2f2f]",
          )}
        ></div>

        <div
          class="relative z-10 flex w-[60px] shrink-0 items-center justify-center"
        >
          <div
            class="flex h-10 w-10 items-center justify-center text-[#b4b4b4] group-hover:text-white transition-colors duration-200"
          >
            <SquarePen size={20} strokeWidth={1.5} />
          </div>
        </div>
        <div
          class={cn(
            "relative z-10 overflow-hidden whitespace-nowrap transition-all duration-300",
            isSidebarOpen ? "w-40 opacity-100 ml-1" : "w-0 opacity-0 ml-0",
          )}
        >
          <span
            class="text-[0.9rem] font-medium text-[#ececec] group-hover:text-white transition-colors duration-200"
          >
            New Chat
          </span>
        </div>
      </button>

      {#each menuItems as item}
        {@const isActive = page.url.pathname === item.path}
        <button
          class="group relative flex h-12 w-full cursor-pointer items-center border-none bg-transparent transition-colors duration-200"
          onclick={() => handleAction(item)}
          title={!isSidebarOpen ? item.label : ""}
        >
          <!-- Background Highlight -->
          <div
            class={cn(
              "absolute transition-all duration-200 rounded-lg z-0",
              isSidebarOpen
                ? "inset-y-1 inset-x-2.5"
                : "inset-y-1 left-[10px] w-10 h-10",
              isActive
                ? "bg-[#3f3f3f]"
                : "bg-transparent group-hover:bg-[#2f2f2f]",
            )}
          ></div>

          <div
            class="relative z-10 flex w-[60px] shrink-0 items-center justify-center"
          >
            <div
              class={cn(
                "flex h-10 w-10 items-center justify-center transition-colors duration-200",
                isActive
                  ? "text-white"
                  : "text-[#b4b4b4] group-hover:text-white",
              )}
            >
              <item.icon size={20} strokeWidth={1.5} />
            </div>
          </div>
          <div
            class={cn(
              "relative z-10 overflow-hidden whitespace-nowrap transition-all duration-300",
              isSidebarOpen ? "w-40 opacity-100 ml-1" : "w-0 opacity-0 ml-0",
            )}
          >
            <span
              class={cn(
                "text-[0.9rem] font-medium transition-colors duration-200",
                isActive
                  ? "text-white"
                  : "text-[#ececec] group-hover:text-white",
              )}
            >
              {item.label}
            </span>
          </div>
        </button>
      {/each}
    </nav>

    <!-- CENTRAL AREA (SCROLLABLE) -->
    <div
      class={cn(
        "scrollbar-hide mt-4 flex grow flex-col overflow-y-auto transition-opacity duration-300",
        !isSidebarOpen && "pointer-events-none opacity-0",
      )}
    >
      <div class="flex flex-col gap-2">
        <button
          class="flex cursor-pointer items-center justify-between p-2 px-6 text-xs font-medium text-[#999] transition-colors hover:text-[#ccc]"
          onclick={toggleChatHistory}
        >
          <span class="whitespace-nowrap uppercase tracking-wider"
            >Your chats</span
          >
          <ChevronDown
            size={14}
            strokeWidth={1.5}
            class={cn("transition-transform", showChatHistory && "rotate-180")}
          />
        </button>

        {#if showChatHistory}
          <div class="flex flex-col gap-0.5 px-3">
            {#each chatStore.history as chat}
              <div
                class={cn(
                  "group relative flex h-9 w-full items-center justify-between overflow-hidden rounded-lg px-3 text-left text-sm text-[#ececec] transition-colors hover:bg-[#2f2f2f]",
                  chatStore.activeConversationId === chat.id &&
                    "bg-[#242424] text-white font-medium",
                )}
              >
                <button
                  class="flex-grow truncate text-left h-full outline-none bg-transparent border-none p-0 cursor-pointer text-inherit font-inherit"
                  onclick={() => chat.id && chatStore.loadConversation(chat.id)}
                >
                  {chat.title}
                </button>

                <div
                  class="absolute right-2 flex items-center opacity-0 transition-opacity group-hover:opacity-100 has-[[data-state=open]]:opacity-100"
                >
                  <DropdownMenu.Root>
                    <DropdownMenu.Trigger
                      class="flex h-6 w-6 items-center justify-center rounded hover:bg-[#3f3f3f] text-[#b4b4b4] hover:text-white transition-colors data-[state=open]:bg-[#3f3f3f] data-[state=open]:text-white"
                    >
                      <Ellipsis size={14} />
                    </DropdownMenu.Trigger>

                    <DropdownMenu.Content
                      class="z-[200] w-56 overflow-hidden rounded-xl border border-[#3f3f3f] bg-[#1e1e1e] p-1 shadow-2xl focus:outline-none"
                      align="end"
                      sideOffset={8}
                    >
                      <DropdownMenu.Item
                        class="flex w-full cursor-pointer items-center gap-2 rounded px-3 py-2 text-sm text-[#ececec] transition-colors outline-none data-[highlighted]:bg-[#2f2f2f] data-[highlighted]:text-white"
                      >
                        <Share size={14} />
                        <span>Share - WIP</span>
                      </DropdownMenu.Item>

                      <DropdownMenu.Item
                        class="flex w-full cursor-pointer items-center gap-2 rounded px-3 py-2 text-sm text-[#ececec] transition-colors outline-none data-[highlighted]:bg-[#2f2f2f] data-[highlighted]:text-white"
                        onclick={() => {
                          /* Rename logic */
                        }}
                      >
                        <Pencil size={14} />
                        <span>Rename</span>
                      </DropdownMenu.Item>

                      <DropdownMenu.Separator
                        class="my-1 h-[1px] bg-[#2f2f2f]"
                      />

                      <DropdownMenu.Item
                        class="flex w-full cursor-pointer items-center gap-2 rounded px-3 py-2 text-sm text-[#ececec] transition-colors outline-none data-[highlighted]:bg-[#2f2f2f] data-[highlighted]:text-white"
                      >
                        <Pin size={14} />
                        <span>Pin Chat</span>
                      </DropdownMenu.Item>

                      <DropdownMenu.Item
                        class="flex w-full cursor-pointer items-center gap-2 rounded px-3 py-2 text-sm text-[#ececec] transition-colors outline-none data-[highlighted]:bg-[#2f2f2f] data-[highlighted]:text-white"
                      >
                        <Archive size={14} />
                        <span>Arquivar</span>
                      </DropdownMenu.Item>

                      <DropdownMenu.Item
                        class="flex w-full cursor-pointer items-center gap-2 rounded px-3 py-2 text-sm text-[#ff6b6b] transition-colors outline-none data-[highlighted]:bg-[#3f2f2f] data-[highlighted]:text-[#ff4b4b]"
                        onclick={() => {
                          if (chat.id) chatStore.deleteChat(chat.id);
                        }}
                      >
                        <Trash2 size={14} />
                        <span>Remove</span>
                      </DropdownMenu.Item>
                    </DropdownMenu.Content>
                  </DropdownMenu.Root>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    <!-- FOOTER (FIXED) -->
    <div class="mt-auto shrink-0 border-t border-[#2f2f2f] pt-2 pb-2">
      <div class="relative">
        <button
          class="group relative flex h-12 w-full cursor-pointer items-center border-none bg-transparent transition-colors duration-200"
          onclick={toggleProfileMenu}
        >
          <!-- Background Highlight -->
          <div
            class={cn(
              "absolute transition-all duration-200 rounded-lg z-0",
              isSidebarOpen
                ? "inset-y-1 inset-x-2"
                : "inset-y-1 left-[10px] w-10 h-10",
              showProfileMenu
                ? "bg-[#3f3f3f]"
                : "bg-transparent group-hover:bg-[#2f2f2f]",
            )}
          ></div>

          <div
            class="relative z-10 flex w-[60px] shrink-0 items-center justify-center"
          >
            <div
              class={cn(
                "flex h-10 w-10 items-center justify-center transition-colors duration-200",
                showProfileMenu
                  ? "text-white"
                  : "text-[#b4b4b4] group-hover:text-white",
              )}
            >
              <User size={22} strokeWidth={1.5} />
            </div>
          </div>
          <div
            class={cn(
              "relative z-10 overflow-hidden whitespace-nowrap transition-all duration-300",
              isSidebarOpen ? "w-40 opacity-100 ml-1" : "w-0 opacity-0 ml-0",
            )}
          >
            <span
              class={cn(
                "text-sm font-medium transition-colors duration-200",
                showProfileMenu
                  ? "text-white"
                  : "text-[#ececec] group-hover:text-white",
              )}
            >
              User
            </span>
          </div>
        </button>

        {#if showProfileMenu && isSidebarOpen}
          <div
            class="absolute bottom-full left-2 right-2 z-1000 mb-2 min-w-[200px] animate-in fade-in slide-in-from-bottom-2 rounded-xl border border-[#3f3f3f] bg-[#2a2a2a] p-1 shadow-2xl"
          >
            <div
              class="flex items-center gap-3 border-b border-[#3f3f3f] p-3 mb-1"
            >
              <div
                class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[#3f3f3f] text-[#b4b4b4]"
              >
                <User size={16} strokeWidth={1.5} />
              </div>
              <span class="truncate text-sm font-semibold text-[#ececec]"
                >User</span
              >
            </div>

            {#each profileMenuItems as item}
              <button
                class="flex w-full items-center gap-3 rounded-lg p-2.5 px-3 text-left text-sm font-medium text-[#ececec] transition-colors hover:bg-[#3f3f3f]"
                onclick={() => handleAction(item)}
              >
                <item.icon size={16} strokeWidth={1.5} />
                <span>{item.label}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</aside>
