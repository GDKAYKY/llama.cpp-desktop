<script lang="ts">
  import { cn } from "$shared/cn.js";
  import {
    Paperclip,
    CirclePlus,
    FileCode,
    Globe,
    ArrowUp,
    Mic,
    AtSign,
    Search,
    MessageSquare,
    Brain,
    ListChecks,
    Activity,
    CodeXml,
    Wrench,
    Package,
    X,
    ChevronDown,
    Hand,
    LaptopMinimal,
  } from "lucide-svelte";
  import { SiModelcontextprotocol } from "@icons-pack/svelte-simple-icons";
  import { mcpStore } from "$lib/stores/mcp.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { extractTextFromPDF } from "$lib/utils/pdf-extractor.js";
  import { notifications } from "$lib/shared/notifications";

  /** @type {{
   *   userInput: string,
   *   modelLoaded: boolean,
   *   isLoading: boolean,
   *   onKeydown: (e: KeyboardEvent) => void,
   *   onInput: () => void,
   *   onSend: () => void,
   *   textarea: HTMLTextAreaElement,
   *   selectedModel?: any
   * }} */
  let {
    userInput = $bindable(),
    modelLoaded,
    isLoading,
    onKeydown,
    onInput,
    onSend,
    textarea = $bindable(),
    selectedModel,
  } = $props();

  let isDropdownOpen = $state(false);
  let isSlashMenuOpen = $state(false);
  let slashQuery = $state("");
  let slashSelectedIndex = $state(0);
  let slashMode = $state("commands");
  let selectedMcps = $state<Array<{ id: string; name: string }>>([]);
  let pendingWebSearchMcpId = $state<string | null>(null);
  let mcpSelectedIndex = $state(0);
  let mcpQuery = $state("");
  const DEFAULT_WEB_SEARCH_MCP_ID = "tavily";

  const slashItems = [
    {
      id: "code-review",
      label: "Code review",
      description: "Review changes and risks",
      icon: CodeXml,
      section: "Commands",
      command: "/code-review",
    },
    {
      id: "feedback",
      label: "Feedback",
      description: "Share feedback about the response",
      icon: MessageSquare,
      section: "Commands",
      command: "/feedback",
    },
    {
      id: "mcp",
      label: "MCP",
      description: "Show MCP server status",
      icon: SiModelcontextprotocol,
      section: "Commands",
      command: "/mcp",
      action: "open-mcp-modal",
    },
    {
      id: "personality",
      label: "Personality",
      description: "Set assistant style",
      icon: Brain,
      section: "Commands",
      command: "/personality",
    },
    {
      id: "plan",
      label: "Plan mode",
      description: "Turn plan mode on",
      icon: ListChecks,
      section: "Commands",
      command: "/plan",
    },
    {
      id: "status",
      label: "Status",
      description: "Show thread id and usage",
      icon: Activity,
      section: "Commands",
      command: "/status",
    },
    {
      id: "skill-creator",
      label: "Skill Creator",
      description: "Create or update a skill",
      icon: Wrench,
      section: "Skills",
      command: "/skill-creator",
    },
    {
      id: "skill-installer",
      label: "Skill Installer",
      description: "Install curated skills",
      icon: Package,
      section: "Skills",
      command: "/skill-installer",
    },
  ];

  const filteredSlashItems = $derived.by(() => {
    const raw = slashQuery.trim().toLowerCase();
    const query = raw.replace(/^\/+/, "");
    if (!query) return slashItems;
    const matches = slashItems.filter((item) => {
      const label = item.label.toLowerCase();
      const description = item.description.toLowerCase();
      const id = item.id.toLowerCase();
      const command = item.command.toLowerCase();
      const commandBare = command.replace(/^\//, "");
      return (
        label.includes(query) ||
        description.includes(query) ||
        id.includes(query) ||
        command.includes(query) ||
        commandBare.includes(query)
      );
    });
    if (matches.length > 0) return matches;
    return slashItems.filter(
      (item) => item.command.toLowerCase() === `/${query}`,
    );
  });

  const slashSections = ["Commands", "Skills"];

  const slashItemsBySection = $derived.by(() => {
    return slashSections.map((section) => ({
      section,
      items: filteredSlashItems.filter((item) => item.section === section),
    }));
  });

  const flatSlashItems = $derived.by(() => {
    return slashItemsBySection.flatMap((group) => group.items);
  });

  const filteredMcpDropdownServers = $derived.by(() => {
    const query = mcpQuery.trim().toLowerCase();
    if (!query) return mcpStore.servers;
    return mcpStore.servers.filter((server) => {
      const name = server.name.toLowerCase();
      const id = server.id.toLowerCase();
      return name.includes(query) || id.includes(query);
    });
  });

  function toggleDropdown(e: MouseEvent) {
    if (e) e.stopPropagation();
    isDropdownOpen = !isDropdownOpen;
  }

  function handleClickOutside() {
    isDropdownOpen = false;
    mcpQuery = "";
  }

  function insertMention(mention: string) {
    if (!textarea) {
      userInput = `${userInput}${mention} `;
      return;
    }
    const start = textarea.selectionStart ?? userInput.length;
    const end = textarea.selectionEnd ?? userInput.length;
    userInput = `${userInput.slice(0, start)}${mention} ${userInput.slice(end)}`;
    queueMicrotask(() => {
      textarea.focus();
      const nextPos = start + mention.length + 1;
      textarea.setSelectionRange(nextPos, nextPos);
    });
  }

  function resolveWebSearchMcpId() {
    const provider = settingsStore.settings.webSearchProvider ?? "tavily";
    if (provider === "custom") {
      const custom = settingsStore.settings.webSearchMcpId?.trim();
      return custom || DEFAULT_WEB_SEARCH_MCP_ID;
    }
    return DEFAULT_WEB_SEARCH_MCP_ID;
  }

  function handleWebSearch() {
    const targetId = resolveWebSearchMcpId();
    if (!targetId) return;
    pendingWebSearchMcpId = targetId;
    isDropdownOpen = false;
  }

  function getSlashMatch() {
    const cursor = textarea?.selectionStart ?? userInput.length;
    const before = userInput.slice(0, cursor);
    const lastSlash = before.lastIndexOf("/");
    if (lastSlash === -1) return null;
    const prevChar = lastSlash > 0 ? before[lastSlash - 1] : "";
    if (prevChar && !/\s/.test(prevChar)) return null;
    const token = before.slice(lastSlash);
    if (token.length === 1) {
      return { start: lastSlash, end: cursor, token, query: "" };
    }
    if (/\s/.test(token)) return null;
    return { start: lastSlash, end: cursor, token, query: token.slice(1) };
  }

  function updateSlashState() {
    const match = getSlashMatch();
    if (!match) {
      isSlashMenuOpen = false;
      slashQuery = "";
      slashSelectedIndex = 0;
      slashMode = "commands";
      mcpSelectedIndex = 0;
      return;
    }
    isSlashMenuOpen = true;
    slashQuery = match.query.replace(/^\//, "");
    slashMode = slashQuery.toLowerCase().startsWith("mcp") ? "mcp" : "commands";
  }

  function handleInputEvent() {
    onInput?.();
    updateSlashState();
  }

  function handleKeydownEvent(e: KeyboardEvent) {
    if (e.key === "Escape" && isSlashMenuOpen) {
      isSlashMenuOpen = false;
      slashQuery = "";
      slashSelectedIndex = 0;
      slashMode = "commands";
      mcpSelectedIndex = 0;
      return;
    }
    if (isSlashMenuOpen && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
      e.preventDefault();
      const delta = e.key === "ArrowDown" ? 1 : -1;
      if (slashMode === "mcp") {
        if (mcpStore.servers.length === 0) return;
        mcpSelectedIndex =
          (mcpSelectedIndex + delta + mcpStore.servers.length) %
          mcpStore.servers.length;
      } else {
        if (flatSlashItems.length === 0) return;
        slashSelectedIndex =
          (slashSelectedIndex + delta + flatSlashItems.length) %
          flatSlashItems.length;
      }
      return;
    }
    if (isSlashMenuOpen && e.key === "Tab") {
      e.preventDefault();
      const delta = e.shiftKey ? -1 : 1;
      if (slashMode === "mcp") {
        if (mcpStore.servers.length === 0) return;
        mcpSelectedIndex =
          (mcpSelectedIndex + delta + mcpStore.servers.length) %
          mcpStore.servers.length;
      } else {
        if (flatSlashItems.length === 0) return;
        slashSelectedIndex =
          (slashSelectedIndex + delta + flatSlashItems.length) %
          flatSlashItems.length;
      }
      return;
    }
    if (e.key === "Enter" && isSlashMenuOpen) {
      if (slashMode === "mcp") {
        const server = mcpStore.servers[mcpSelectedIndex];
        if (server) {
          e.preventDefault();
          if (!selectedMcps.some((item) => item.id === server.id)) {
            selectedMcps = [
              ...selectedMcps,
              { id: server.id, name: server.name },
            ];
          }
          removeSlashToken();
          isSlashMenuOpen = false;
          slashMode = "commands";
          return;
        }
      } else {
        const item = flatSlashItems[slashSelectedIndex];
        if (item) {
          e.preventDefault();
          handleSlashSelect(item);
          return;
        }
      }
    }
    onKeydown?.(e);
    queueMicrotask(updateSlashState);
  }

  function insertSlashCommand(command: string) {
    const match = getSlashMatch();
    if (!match) {
      userInput = `${userInput}${command} `;
    } else {
      userInput = `${userInput.slice(0, match.start)}${command} ${userInput.slice(match.end)}`;
    }
    isSlashMenuOpen = false;
    slashQuery = "";
    slashSelectedIndex = 0;
    slashMode = "commands";
    queueMicrotask(() => {
      textarea?.focus();
      const nextPos = (match?.start ?? userInput.length) + command.length + 1;
      textarea?.setSelectionRange(nextPos, nextPos);
    });
  }

  function removeSlashToken() {
    const match = getSlashMatch();
    if (match) {
      userInput = `${userInput.slice(0, match.start)}${userInput.slice(match.end)}`;
    }
  }

  function removeMcpToken(id: string) {
    const pattern = new RegExp(
      `(?:^|\\s)/mcp:${id.replace(/[.*+?^${}()|[\\]\\\\]/g, "\\$&")}(?=\\s|$)`,
      "i",
    );
    userInput = userInput
      .replace(pattern, "")
      .replace(/\s{2,}/g, " ")
      .trimStart();
    selectedMcps = selectedMcps.filter((item) => item.id !== id);
  }

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (selectedMcps.length > 0) {
      const mentions = selectedMcps.map((item) => `/mcp:${item.id}`).join(" ");
      const composed = `${userInput} ${mentions}`.trim();
      userInput = composed;
      selectedMcps = [];
    }
    if (pendingWebSearchMcpId) {
      const composed = `${userInput} @mcp:${pendingWebSearchMcpId}`.trim();
      userInput = composed;
      pendingWebSearchMcpId = null;
    }
    onSend();
  }

  function handleSlashSelect(item: (typeof slashItems)[number]) {
    if (item.action === "open-mcp-modal") {
      slashQuery = "";
      slashSelectedIndex = 0;
      slashMode = "mcp";
      mcpSelectedIndex = 0;
      return;
    }
    insertSlashCommand(item.command);
  }

  async function handleFileUpload(e: Event) {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    if (
      file.type !== "application/pdf" &&
      !file.name.toLowerCase().endsWith(".pdf")
    ) {
      notifications.error("Only PDF files are supported currently.");
      return;
    }

    try {
      notifications.message("Extracting text from PDF...");
      const text = await extractTextFromPDF(file);
      userInput = userInput
        ? `${userInput}\n\n[PDF: ${file.name}]\n${text}\n`
        : `[PDF: ${file.name}]\n${text}\n`;
      notifications.success("PDF text extracted successfully.");
      isDropdownOpen = false;
    } catch (err) {
      notifications.error("Failed to parse PDF.");
    } finally {
      target.value = "";
    }
  }

  $effect(() => {
    if (!isSlashMenuOpen) return;
    if (slashMode === "mcp") {
      if (mcpStore.servers.length === 0) {
        mcpSelectedIndex = 0;
        return;
      }
      if (mcpSelectedIndex >= mcpStore.servers.length) {
        mcpSelectedIndex = mcpStore.servers.length - 1;
      }
    } else {
      if (flatSlashItems.length === 0) {
        slashSelectedIndex = 0;
        return;
      }
      if (slashSelectedIndex >= flatSlashItems.length) {
        slashSelectedIndex = flatSlashItems.length - 1;
      }
    }
  });
</script>

<svelte:window on:click={handleClickOutside} />

<div class="mx-auto w-full max-w-[40rem] px-4 md:px-6 lg:max-w-[48rem]">
  <div class="relative">
    {#if isSlashMenuOpen}
      <div
        class="absolute bottom-[calc(100%+10px)] left-0 z-100 flex w-[320px] flex-col overflow-hidden rounded-2xl border border-white/10 bg-[#1f1f1f] p-2 shadow-2xl"
        role="menu"
        tabindex="-1"
        onclick={(e: MouseEvent) => e.stopPropagation()}
        onkeydown={() => {}}
      >
        {#if slashMode === "mcp"}
          <div
            class="flex items-center gap-2 rounded-xl bg-black/20 px-3 py-2 text-xs text-muted-foreground"
          >
            <SiModelcontextprotocol size={14} />
            <span>MCP Servers</span>
          </div>
          <div class="mt-2 max-h-[320px] overflow-y-auto pr-1">
            {#if mcpStore.servers.length === 0}
              <div class="px-3 py-2 text-xs text-muted-foreground">
                Nenhum MCP configurado
              </div>
            {:else}
              {#each mcpStore.servers as server, index}
                <button
                  type="button"
                  class={cn(
                    "flex w-full cursor-pointer items-center gap-3 rounded-xl border-none bg-transparent px-3 py-2 text-left text-sm text-foreground transition-colors hover:bg-white/6",
                    index === mcpSelectedIndex && "bg-white/8",
                  )}
                  onclick={() => {
                    if (!selectedMcps.some((item) => item.id === server.id)) {
                      selectedMcps = [
                        ...selectedMcps,
                        { id: server.id, name: server.name },
                      ];
                    }
                    removeSlashToken();
                    isSlashMenuOpen = false;
                    slashMode = "commands";
                  }}
                >
                  <SiModelcontextprotocol size={16} />
                  <div class="flex flex-col gap-0.5">
                    <span class="text-sm">{server.name}</span>
                    <span class="text-[11px] text-muted-foreground font-mono">
                      {server.id}
                    </span>
                  </div>
                </button>
              {/each}
            {/if}
          </div>
        {:else}
          <div
            class="flex items-center gap-2 rounded-xl bg-black/20 px-3 py-2 text-xs text-muted-foreground"
          >
            <Search size={14} />
            <span>Search</span>
            <span class="ml-auto text-[11px] text-muted-foreground/80"
              >/{slashQuery}</span
            >
          </div>

          <div class="mt-2 max-h-[320px] overflow-y-auto pr-1">
            {#if filteredSlashItems.length === 0}
              <div class="px-3 py-2 text-xs text-muted-foreground">
                Nenhum comando encontrado
              </div>
            {:else}
              {#each slashItemsBySection as group}
                {#if group.items.length > 0}
                  <div
                    class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground/70"
                  >
                    {group.section}
                  </div>
                  {#each group.items as item}
                    {@const Icon = item.icon}
                    <button
                      type="button"
                      class={cn(
                        "flex w-full cursor-pointer items-center gap-3 rounded-xl border-none bg-transparent px-3 py-2 text-left text-sm text-foreground transition-colors hover:bg-white/6",
                        flatSlashItems.indexOf(item) === slashSelectedIndex &&
                          "bg-white/8",
                      )}
                      onclick={() => handleSlashSelect(item)}
                    >
                      <Icon size={16} />
                      <div class="flex flex-col gap-0.5">
                        <span class="text-sm">{item.label}</span>
                        <span class="text-[11px] text-muted-foreground">
                          {item.description}
                        </span>
                      </div>
                    </button>
                  {/each}
                {/if}
              {/each}
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    <form class="transition-all" onsubmit={handleSubmit}>
      <div
        class="relative flex min-h-[120px] flex-col overflow-hidden rounded-[1.65rem] border border-white/12 bg-[#2f2f2f]/95 shadow-[0_16px_50px_rgba(0,0,0,0.32)] backdrop-blur-xl"
      >
        <div class="flex-1 px-4 pb-1 pt-3">
          <textarea
            bind:this={textarea}
            bind:value={userInput}
            oninput={handleInputEvent}
            onkeydown={handleKeydownEvent}
            placeholder="Message llama-desktop..."
            rows="1"
            class="max-h-[25dvh] min-h-[2.75rem] w-full resize-none border-none bg-transparent py-1 text-base leading-6 text-foreground outline-none placeholder:text-muted-foreground placeholder:opacity-70"
          ></textarea>
        </div>

        <div
          class="grid grid-cols-[minmax(0,auto)_auto_minmax(0,1fr)] items-center gap-[5px] px-2 pb-2"
        >
          <div class="flex min-w-0 items-center gap-[5px]">
            <div class="relative">
              <button
                type="button"
                class="flex h-9 w-9 cursor-pointer items-center justify-center rounded-full border border-transparent bg-transparent p-0 text-muted-foreground transition-colors hover:bg-white/7 hover:text-foreground"
                onclick={toggleDropdown}
                title="Add files and more"
                aria-label="Add files and more"
              >
                <CirclePlus size={20} />
              </button>

              {#if isDropdownOpen}
                <div
                  class="absolute bottom-[calc(100%+12px)] left-0 z-100 w-[200px] overflow-hidden rounded-xl border border-border bg-secondary p-1.5 shadow-lg"
                  role="menu"
                  tabindex="-1"
                  onclick={(e: MouseEvent) => e.stopPropagation()}
                  onkeydown={() => {}}
                >
                  <button
                    type="button"
                    class="flex w-full cursor-pointer items-center gap-3 rounded-lg border-none bg-transparent px-3.5 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-white/8"
                    onclick={() =>
                      document.getElementById("file-upload-input")?.click()}
                  >
                    <Paperclip size={18} />
                    <span>Upload from computer</span>
                  </button>
                  <button
                    type="button"
                    class="flex w-full cursor-pointer items-center gap-3 rounded-lg border-none bg-transparent px-3.5 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-white/8"
                  >
                    <FileCode size={18} />
                    <span>Search my files</span>
                  </button>
                  <button
                    type="button"
                    class="flex w-full cursor-pointer items-center gap-3 rounded-lg border-none bg-transparent px-3.5 py-2.5 text-left text-sm text-foreground transition-colors hover:bg-white/8"
                    onclick={handleWebSearch}
                  >
                    <Globe size={18} />
                    <span>Search the web</span>
                  </button>
                </div>
              {/if}
            </div>

            <button
              type="button"
              class="hidden h-8 min-w-0 cursor-pointer items-center gap-1.5 rounded-full border border-transparent px-2 text-sm leading-[18px] text-muted-foreground transition-colors hover:bg-white/7 hover:text-foreground sm:flex"
              title="Approval mode"
            >
              <Hand size={18} class="shrink-0" />
              <span class="max-w-36 truncate text-left">Ask for approval</span>
              <ChevronDown
                size={14}
                class="shrink-0 text-muted-foreground/75"
              />
            </button>

            {#if selectedMcps.length > 0}
              {#each selectedMcps as selectedMcp}
                <button
                  type="button"
                  class="group inline-flex items-center gap-2 rounded-full bg-white/5 px-2.5 py-1 text-xs text-foreground/90 transition-colors hover:bg-white/10"
                  title="Remove MCP"
                  onclick={() => removeMcpToken(selectedMcp.id)}
                >
                  <span
                    class="relative inline-flex h-4 w-4 items-center justify-center rounded-full bg-white/10 text-foreground/70"
                  >
                    <span class="contents group-hover:hidden">
                      <SiModelcontextprotocol size={12} />
                    </span>
                    <X size={12} class="hidden group-hover:block" />
                  </span>
                  <span>{selectedMcp?.name}</span>
                </button>
              {/each}
            {/if}
            {#if pendingWebSearchMcpId}
              <button
                type="button"
                class="group inline-flex items-center gap-2 rounded-full bg-white/5 px-2.5 py-1 text-xs text-foreground/90 transition-colors hover:bg-white/10"
                title="Remove Web Search"
                onclick={() => {
                  pendingWebSearchMcpId = null;
                }}
              >
                <span
                  class="relative inline-flex h-4 w-4 items-center justify-center rounded-full bg-white/10 text-foreground/70"
                >
                  <span class="contents group-hover:hidden">
                    <Globe size={12} />
                  </span>
                  <X size={12} class="hidden group-hover:block" />
                </span>
                <span>Web search</span>
              </button>
            {/if}
          </div>

          <div class="flex items-center"></div>

          <div class="flex min-w-0 items-center justify-end gap-2">
            <button
              type="button"
              class="flex h-9 w-9 cursor-pointer items-center justify-center rounded-full border border-transparent bg-transparent p-0 text-muted-foreground transition-colors hover:bg-white/7 hover:text-foreground"
              title="Voice input"
              aria-label="Voice input"
            >
              <Mic size={20} />
            </button>

            <button
              type="submit"
              disabled={!userInput.trim() || isLoading || !modelLoaded}
              class="flex h-9 w-9 shrink-0 cursor-pointer items-center justify-center rounded-full border-none bg-white text-black transition-all hover:enabled:bg-white/90 disabled:cursor-not-allowed disabled:bg-white/10 disabled:text-white/30"
              title="Send message"
              aria-label="Send message"
            >
              <ArrowUp size={20} strokeWidth={2.5} />
            </button>
          </div>
        </div>
      </div>
    </form>
    <input
      type="file"
      id="file-upload-input"
      accept=".pdf"
      class="hidden"
      onchange={handleFileUpload}
    />
  </div>
</div>
