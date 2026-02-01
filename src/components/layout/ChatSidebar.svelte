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
  } from "lucide-svelte";
  import { page } from "$app/state";

  let { isSidebarOpen, toggleSidebar } = $props();

  let showChatHistory = $state(true);
  let showProfileMenu = $state(false);

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
            {#each chatHistory as chat}
              <button
                class={cn(
                  "group relative flex h-9 w-full items-center justify-between overflow-hidden rounded-lg px-3 text-left text-sm text-[#ececec] transition-colors hover:bg-[#2f2f2f]",
                  chat.active && "bg-[#242424] text-white font-medium",
                )}
              >
                <span class="truncate">{chat.title}</span>
                <div
                  class="absolute right-2 flex items-center opacity-0 transition-opacity group-hover:opacity-100"
                >
                  <div
                    class="flex h-6 w-6 items-center justify-center rounded hover:bg-[#3f3f3f]"
                    onclick={(e) => e.stopPropagation()}
                    role="button"
                    tabindex="0"
                    onkeydown={(e) => e.key === "Enter" && e.stopPropagation()}
                    aria-label="Options"
                  >
                    <Ellipsis size={14} />
                  </div>
                </div>
              </button>
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
              Antigravity User
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
