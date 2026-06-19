<script lang="ts">
  import {
    ChevronDown,
    ChevronRight,
    ChevronUp,
    Search,
    MoreHorizontal,
    Copy,
    Folder,
    Clipboard,
    EyeOff,
    TextCursor,
    WrapText,
    FoldVertical,
    Image,
    RefreshCw,
    GitCommit,
    GitPullRequest,
    File,
    ExternalLink,
    Undo2,
    Plus,
    Columns,
    FileSymlink,
    FileSearchCorner,
    Folders,
    GitCommitHorizontal,
  } from "lucide-svelte";

  // We hardcode the exact representation from the image to get a 1:1 match

  let { parentWidth = 400 } = $props();

  let isResizing = $state(false);
  let sidebarWidth = $state(250);
  let startX = $state(0);
  let startWidth = $state(250);
  let showFileTree = $state(true);
  let showMoreMenu = $state(false);

  let userSetSidebarWidth = $state(250);
  const minDiffWidth = 180; // Minimum width to keep the diff view readable

  // Keep track of the user's preferred width when resizing the filetree directly
  $effect(() => {
    if (isResizing) {
      userSetSidebarWidth = sidebarWidth;
    }
  });

  // Dynamically adjust sidebarWidth when parentWidth changes
  $effect(() => {
    if (!isResizing) {
      const maxAllowed = parentWidth - minDiffWidth;
      sidebarWidth = Math.max(120, Math.min(userSetSidebarWidth, maxAllowed));
    }
  });

  function startResize(e: MouseEvent) {
    isResizing = true;
    startX = e.clientX;
    startWidth = sidebarWidth;
    e.preventDefault();
    e.stopPropagation();
  }

  $effect(() => {
    function handleMouseMove(e: MouseEvent) {
      if (!isResizing) return;
      const delta = startX - e.clientX;
      sidebarWidth = Math.max(150, Math.min(600, startWidth + delta));
    }

    function handleMouseUp() {
      isResizing = false;
    }

    if (isResizing) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";
    }

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
  });

  function getCollapsedFolder(path: string, currentWidth: number): string {
    if (currentWidth < 280) {
      if (path === "backend / Services / Messaging") {
        return "back... / Servi... / Messa...";
      }
      // Fallback dynamic collapse
      const parts = path.split(" / ");
      return parts
        .map((part) => (part.length > 6 ? part.slice(0, 4) + "..." : part))
        .join(" / ");
    }
    return path;
  }

  function getCollapsedFile(
    name: string,
    indent: number,
    currentWidth: number,
  ): string {
    if (currentWidth < 280) {
      if (name === "MessageProcessorService.cs") return "MessageProcessor...cs";
      if (name === "AdminShell.svelte") return "AdminSh...svelte";
      if (name === "RESPONSIVE_DESIGN.md") return "RESPONSIVE_DES...md";
    }
    // Fallback dynamic collapse
    const limit = Math.max(10, Math.floor((currentWidth - indent - 20) / 9));
    if (name.length <= limit) return name;
    const dotIndex = name.lastIndexOf(".");
    if (dotIndex !== -1 && name.length - dotIndex <= 8) {
      const ext = name.slice(dotIndex);
      const base = name.slice(0, dotIndex);
      const baseShowLen = Math.max(3, limit - ext.length - 3);
      return base.slice(0, baseShowLen) + "..." + ext;
    }
    return name.slice(0, limit - 3) + "...";
  }

  let files = $state([
    {
      id: "message_processor",
      name: "MessageProcessorService.cs",
      pathPrefix: "...nd/Services/Messaging/",
      additions: 3,
      deletions: 2,
      type: "file",
      collapsed: false,
    },
    {
      id: "admin_shell",
      name: "AdminShell.svelte",
      pathPrefix: "...nd/src/components/",
      additions: 145,
      deletions: 68,
      type: "svelte",
      collapsed: false,
    },
    {
      id: "app_css",
      name: "app.css",
      pathPrefix: "frontend/src/",
      additions: 9,
      deletions: 0,
      type: "css",
      collapsed: true,
    },
    {
      id: "package_lock",
      name: "package-lock.json",
      pathPrefix: "...tend/",
      additions: 11,
      deletions: 0,
      type: "json",
      collapsed: true,
    },
  ]);
</script>

<div
  class="flex flex-col h-full w-full text-[#cccccc] bg-background font-sans overflow-hidden text-[13px]"
>
  <!-- Top Header Bar -->
  <div class="flex items-center justify-between px-3 py-1.5 shrink-0">
    <div class="flex items-center space-x-2 shrink-0">
      <span class=" text-[#ffffff]">Não marcadas para commit</span>
      <span
        class="bg-[#333333] text-[#cccccc] px-1.5 py-0.5 rounded-md text-xs font-medium"
        >7</span
      >
      <ChevronDown size={14} class="text-[#cccccc]" />
      <span class="text-[#00C853] text-xs ml-2">+348</span>
      <span class="text-[#ED5B37] text-xs ml-1">-89</span>
    </div>

    <div class="flex items-center space-x-1.5 shrink-0 ml-2">
      <div class="relative">
        <button
          class="p-1.5 rounded transition-colors {showMoreMenu
            ? 'bg-[#333333] text-[#ffffff]'
            : 'hover:bg-[#333333] text-[#cccccc]'}"
          title="Mais Ações..."
          onclick={() => (showMoreMenu = !showMoreMenu)}
        >
          <MoreHorizontal size={14} />
        </button>

        {#if showMoreMenu}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="fixed inset-0 z-40"
            onclick={() => (showMoreMenu = false)}
          ></div>
          <div
            class="absolute right-0 top-full mt-1 w-72 bg-[#252526] border border-[#454545] shadow-lg rounded-md py-1 z-50 text-[13px] text-[#cccccc]"
          >
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-[#04395e] hover:text-white flex items-center"
              ><RefreshCw size={14} class="mr-3 shrink-0 text-[#cccccc]" /> Atualizar</button
            >
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-[#04395e] hover:text-white flex items-center"
              ><WrapText size={14} class="mr-3 shrink-0 text-[#cccccc]" /> Habilitar
              quebra automática de linha</button
            >
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-[#04395e] hover:text-white flex items-center"
              ><FoldVertical size={14} class="mr-3 shrink-0 text-[#cccccc]" /> Minimizar
              todos os diffs</button
            >
            <div class="h-[1px] bg-[#454545] my-1"></div>
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-[#04395e] hover:text-white flex items-center"
              ><File size={14} class="mr-3 shrink-0 text-[#cccccc]" /> Não carregar
              arquivos completos</button
            >
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-[#04395e] hover:text-white flex items-center"
              ><Image size={14} class="mr-3 shrink-0 text-[#cccccc]" /> Habilitar
              prévia detalhada</button
            >
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-[#04395e] hover:text-white flex items-center"
              ><TextCursor size={14} class="mr-3 shrink-0 text-[#cccccc]" /> Habilitar
              diffs por palavra</button
            >
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-[#04395e] hover:text-white flex items-center"
              ><EyeOff size={14} class="mr-3 shrink-0 text-[#cccccc]" /> Ocultar
              espaços em branco</button
            >
            <button
              class="w-full text-left px-4 py-1.5 hover:bg-[#04395e] hover:text-white flex items-center"
              ><Clipboard size={14} class="mr-3 shrink-0 text-[#cccccc]" /> Copie
              o comando git apply</button
            >
          </div>
        {/if}
      </div>

      <button
        class="p-1.5 hover:bg-[#333333] rounded text-[#cccccc] transition-colors"
        title="Ir para o arquivo"><FileSearchCorner size={14} /></button
      >
      <button
        class="p-1.5 hover:bg-[#333333] rounded text-[#cccccc] transition-colors"
        title="Mudar para git dividido"><Columns size={14} /></button
      >
      <button
        class="p-1.5 rounded transition-colors mr-1 {showFileTree
          ? 'bg-[#333333] text-[#ffffff]'
          : 'hover:bg-[#333333] text-[#cccccc]'}"
        title="Mostrar arquivos"
        onclick={() => (showFileTree = !showFileTree)}
      >
        <Folders size={14} />
      </button>

      <button
        class="flex items-center border gap-1 border-[#454545] bg-[#2d2d2d] hover:bg-[#3d3d3d] text-[#ececec] px-3 py-1 rounded-lg text-s transition-colors ml-1"
      >
        <GitCommitHorizontal size={14} />
        <span>Comitar ou enviar</span>
      </button>

      <button
        class="flex items-center border border-[#333333] bg-transparent text-[#666666] px-3 py-1 rounded-lg text-s ml-1 cursor-not-allowed"
      >
        <GitPullRequest size={14} />
        <span>Criar PR</span>
      </button>
    </div>
  </div>

  <!-- Content Area (Split View) -->
  <div class="flex-1 flex overflow-hidden relative w-full">
    <!-- Left Main Area (Diff View) -->
    <div
      class="flex-1 flex flex-col relative"
      style="min-width: {minDiffWidth}px"
    >
      <!-- Diff Content Scroll Area -->
      <div class="flex-1 always-scrollbar bg-background relative">
        {#each files as file (file.id)}
          <div
            class="mr-0.5 transition-all duration-150 {file.collapsed
              ? 'mt-0.5'
              : 'mt-0.5'}  bg-background group/file"
          >
            <!-- File Header -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="flex items-center justify-between px-3 py-3 cursor-pointer select-none bg-[#2d2d2d] group/header {file.collapsed
                ? ''
                : ''}"
              onclick={() => {
                file.collapsed = !file.collapsed;
              }}
            >
              <div class="flex items-center text-[#cccccc] truncate min-w-0">
                {#if file.type === "file"}
                  <File size={14} class="shrink-0 text-[#858585] mr-1.5" />
                {:else if file.type === "svelte"}
                  <span
                    class="text-[#ff3e00] shrink-0 font-bold text-sm leading-none mr-1.5"
                    style="margin-top: 2px;">S</span
                  >
                {:else if file.type === "css"}
                  <span
                    class="text-[#519aba] shrink-0 font-bold text-sm leading-none mr-1.5"
                    style="margin-top: 2px;">#</span
                  >
                {:else if file.type === "json"}
                  <span
                    class="text-[#cbcb41] shrink-0 font-bold text-sm leading-none mr-1.5"
                    style="margin-top: 2px;">{"{}"}</span
                  >
                {/if}
                <span
                  class="truncate text-[13px] mr-1.5"
                  style="direction: rtl; text-align: left;"
                >
                  <span style="direction: ltr; unicode-bidi: embed;">
                    <span class="text-[#858585]">{file.pathPrefix}</span><span
                      class=" text-[#ffffff]">{file.name}</span
                    >
                  </span>
                </span>

                <!-- Chevron only on hover -->
                <div class="w-4 h-4 flex items-center justify-center shrink-0">
                  <span
                    class="opacity-0 group-hover/header:opacity-100 transition-opacity flex items-center justify-center"
                  >
                    {#if file.collapsed}
                      <ChevronRight size={13} class="text-[#cccccc]" />
                    {:else}
                      <ChevronDown size={13} class="text-[#cccccc]" />
                    {/if}
                  </span>
                </div>
              </div>
              <div class="flex items-center space-x-3 shrink-0 text-s">
                <!-- Hover actions for the file header -->
                <div
                  class="opacity-0 group-hover/header:opacity-100 flex items-center space-x-1.5 transition-opacity mr-1"
                >
                  <button
                    class="p-0.5 hover:bg-[#333333] rounded text-[#858585] hover:text-[#cccccc] transition-colors bg-transparent border-0 cursor-pointer"
                    title="Reverter arquivo"
                    onclick={(e) => {
                      e.stopPropagation();
                    }}
                  >
                    <Undo2 size={13} />
                  </button>
                  <button
                    class="p-0.5 hover:bg-[#333333] rounded text-[#858585] hover:text-[#cccccc] transition-colors bg-transparent border-0 cursor-pointer"
                    title="Marcar arquivo para commit"
                    onclick={(e) => {
                      e.stopPropagation();
                    }}
                  >
                    <Plus size={13} />
                  </button>
                </div>
                <span class="text-[#00c853]">+{file.additions}</span>
                {#if file.deletions > 0}
                  <span class="text-[#ED5B37]">-{file.deletions}</span>
                {/if}
                <ExternalLink
                  size={14}
                  class="text-[#858585] hover:text-[#cccccc] cursor-pointer"
                  onclick={(e) => e.stopPropagation()}
                />
              </div>
            </div>

            <!-- File Content -->
            {#if !file.collapsed}
              <div class="bg-[#1e1e1e] overflow-hidden">
                {#if file.id === "message_processor"}
                  <div
                    class="flex items-center px-3 py-1.5 bg-[#2d2d2d] text-[#cccccc] text-xs cursor-pointer border-b border-[#333333]"
                  >
                    <ChevronUp size={14} class="mr-2" />
                    <span>56 unmodified lines</span>
                  </div>

                  <div class="font-mono text-xs leading-5">
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        57
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#d4d4d4]"
                          >data.RemoteJid, data.FromMe, data.MessageType,
                          data.Se</span
                        >
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        58
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre"></div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        59
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#6a9955]"
                          >// -- fromMe -> No Operation (fluxo continua para
                          isMedia)</span
                        >
                      </div>
                    </div>

                    <div class="flex bg-[#4b1818] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#ED5B37]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#4b1818]"
                      >
                        60
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#c586c0]">if</span>
                        <span class="text-[#d4d4d4]">(data.FromMe ==</span>
                        <span class="text-[#569cd6]">true</span><span
                          class="text-[#d4d4d4]">)</span
                        ><span class="text-[#ffd700]">{"{"}</span>
                      </div>
                    </div>
                    <div class="flex bg-[#4b1818] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#ED5B37]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#4b1818]"
                      >
                        61
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#6a9955]">// return false;</span>
                      </div>
                    </div>

                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        60
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#c586c0]">if</span>
                        <span class="text-[#d4d4d4]">(data.FromMe ==</span>
                        <span class="text-[#569cd6]">true</span><span
                          class="text-[#d4d4d4]">)</span
                        >
                      </div>
                      <!-- Hover actions on the right -->
                      <div
                        class="absolute right-4 top-1/2 -translate-y-1/2 hidden group-hover/line:flex items-center bg-[#252526] border border-[#454545] rounded shadow-md z-10 overflow-hidden shrink-0"
                      >
                        <button
                          class="p-1 hover:bg-[#333333] text-[#cccccc] flex items-center justify-center bg-transparent border-0 cursor-pointer"
                          title="Reverter alteração"><Undo2 size={13} /></button
                        >
                        <div class="w-[1px] h-4 bg-[#454545]"></div>
                        <button
                          class="p-1 hover:bg-[#333333] text-[#cccccc] flex items-center justify-center bg-transparent border-0 cursor-pointer"
                          title="Marcar alteração"><Plus size={13} /></button
                        >
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        61
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#ffd700]">{"{"}</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        62
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#c586c0]">return</span>
                        <span class="text-[#569cd6]">false</span><span
                          class="text-[#d4d4d4]">;</span
                        >
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        63
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#ffd700]">{"}"}</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        64
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#6a9955]"
                          >// -- isMedia
                          --------------------------------------------</span
                        >
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        65
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#569cd6]">var</span>
                        <span class="text-[#9cdcfe]">analysisResult</span>
                        = <span class="text-[#ce9178]">"Sem mídia"</span><span
                          class="text-[#d4d4d4]">;</span
                        >
                      </div>
                    </div>
                  </div>

                  <div
                    class="flex items-center px-3 py-1.5 bg-[#2d2d2d] text-[#cccccc] text-xs cursor-pointer border-t border-[#333333]"
                  >
                    <ChevronDown size={14} class="mr-2" />
                    <span>94 unmodified lines</span>
                  </div>
                {:else if file.id === "admin_shell"}
                  <div
                    class="flex items-center px-3 py-1.5 bg-[#2d2d2d] text-[#cccccc] text-xs cursor-pointer border-b border-[#333333]"
                  >
                    <ChevronUp size={14} class="mr-2" />
                    <span>126 unmodified lines</span>
                  </div>

                  <div class="font-mono text-xs leading-5">
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        127
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#808080]">&lt;/</span><span
                          class="text-[#569cd6]">script</span
                        ><span class="text-[#808080]">&gt;</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        128
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre"></div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        129
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#d4d4d4]">{"{"}</span><span
                          class="text-[#c586c0]">#if</span
                        > <span class="text-[#9cdcfe]">data</span><span
                          class="text-[#d4d4d4]">{"}"}</span
                        >
                      </div>
                    </div>

                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        130
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#6a9955]">&lt;!--</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        131
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#6a9955]"
                          >RESPONSIVE LAYOUT STRATEGY:</span
                        >
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        132
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#6a9955]"
                          >- Uses viewport units (vh, vw, dvh, dvw)</span
                        >
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        133
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#6a9955]"
                          >- Mobile-first approach with breakpoints: sm (640px),
                          md (768px)</span
                        >
                      </div>
                    </div>
                  </div>
                {:else if file.id === "app_css"}
                  <div class="font-mono text-xs leading-5">
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        1
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#c586c0]">@import</span>
                        <span class="text-[#ce9178]">"tailwindcss"</span>;
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        2
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#c586c0]">@plugin</span>
                        <span class="text-[#ce9178]">"fluid-tailwindcss"</span>;
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        3
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre"></div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        4
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#569cd6]">@theme</span>
                        <span class="text-[#ffd700]">{"{"}</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        5
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        --color-brand-primary: <span class="text-[#ce9178]"
                          >#007fd4</span
                        >;
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        6
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        --color-brand-secondary: <span class="text-[#ce9178]"
                          >#252526</span
                        >;
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        7
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        --color-brand-bg: <span class="text-[#ce9178]"
                          >#1e1e1e</span
                        >;
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        8
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        --color-brand-border: <span class="text-[#ce9178]"
                          >#333333</span
                        >;
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        9
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#ffd700]">{"}"}</span>
                      </div>
                    </div>
                  </div>
                {:else if file.id === "package_lock"}
                  <div
                    class="flex items-center px-3 py-1.5 bg-[#2d2d2d] text-[#cccccc] text-xs cursor-pointer border-b border-[#333333]"
                  >
                    <ChevronUp size={14} class="mr-2" />
                    <span>14 unmodified lines</span>
                  </div>

                  <div class="font-mono text-xs leading-5">
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        15
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]">
                          "@sveltejs/vite-plugin-svelte"</span
                        >: <span class="text-[#ce9178]">"^6.0.0"</span>,
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        16
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]">
                          "@tailwindcss/vite"</span
                        >: <span class="text-[#ce9178]">"^4.2.2"</span>,
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        17
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "autoprefixer"</span>:
                        <span class="text-[#ce9178]">"^10.4.27"</span>,
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        18
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]">
                          "fluid-tailwindcss"</span
                        >: <span class="text-[#ce9178]">"^1.0.9"</span>,
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        19
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "postcss"</span>:
                        <span class="text-[#ce9178]">"^8.5.9"</span>,
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        20
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "tailwindcss"</span>:
                        <span class="text-[#ce9178]">"^4.2.2"</span>,
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        21
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "vite"</span>:
                        <span class="text-[#ce9178]">"^7.0.0"</span>
                      </div>
                    </div>
                  </div>

                  <div
                    class="flex items-center px-3 py-1.5 bg-[#2d2d2d] text-[#cccccc] text-xs cursor-pointer border-t border-b border-[#333333]"
                  >
                    <ChevronDown size={14} class="mr-2" />
                    <span>1467 unmodified lines</span>
                  </div>

                  <div class="font-mono text-xs leading-5">
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        1489
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#ffd700]">{"}"}</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        1490
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#ffd700]">{"}"}</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                      >
                        1491
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#ffd700]">{"},"}</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        1492
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]">
                          "node_modules/fluid-tailwindcss"</span
                        >: <span class="text-[#ffd700]">{"{"}</span>
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        1493
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "version"</span>:
                        <span class="text-[#ce9178]">"1.0.9"</span>,
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        1494
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "resolved"</span>:
                        <span class="text-[#ce9178]"
                          >"https://registry.npmjs.org/fluid-tailwindcss/-/fluid-tailwindcss-1.0.9.tgz"</span
                        >,
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        1495
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "integrity"</span>:
                        <span class="text-[#ce9178]"
                          >"sha512-1mtVb/ehau/3wAgnrB1I828..."</span
                        >,
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        1496
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "dev"</span>:
                        <span class="text-[#569cd6]">true</span>,
                      </div>
                    </div>
                    <div class="flex bg-[#1e3422] relative group/line">
                      <div
                        class="absolute left-0 top-0 bottom-0 w-1 bg-[#00c853]"
                      ></div>
                      <div
                        class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                      >
                        1497
                      </div>
                      <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                        <span class="text-[#9cdcfe]"> "license"</span>:
                        <span class="text-[#ce9178]">"MIT"</span>,
                      </div>
                    </div>
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Floating Action Bar at the Bottom -->
      <div
        class="absolute bottom-6 left-1/2 -translate-x-1/2 flex items-center bg-[#2d2d2d] rounded-full px-1 py-1 shadow-lg border border-[#454545] whitespace-nowrap scale-[0.6] origin-bottom z-10"
      >
        <button
          class="flex items-center space-x-1.5 px-3 py-1.5 rounded-full hover:bg-[#3d3d3d] text-[#cccccc] text-xs transition-colors shrink-0"
        >
          <Undo2 size={14} class="text-[#858585]" />
          <span>Reverter tudo</span>
        </button>
        <div class="w-[1px] h-4 bg-[#454545] mx-1 shrink-0"></div>
        <button
          class="flex items-center space-x-1.5 px-3 py-1.5 rounded-full hover:bg-[#3d3d3d] text-[#cccccc] text-xs transition-colors shrink-0"
        >
          <Plus size={14} class="text-[#858585]" />
          <span>Marcar tudo para commit</span>
        </button>
      </div>
    </div>

    <!-- Right Sidebar (File Tree) -->
    {#if showFileTree}
      <div
        class="flex-col bg-background border-l border-[#333333] hidden lg:flex shrink-0 relative"
        style="width: {sidebarWidth}px"
      >
        <!-- Resize Handle -->
        <button
          type="button"
          class="absolute left-0 top-0 bottom-0 w-3 -translate-x-1/2 cursor-col-resize group z-50 border-0 p-0 bg-transparent"
          aria-label="Resize panel"
          onmousedown={startResize}
        >
          <div
            class="w-[2px] h-full mx-auto bg-transparent group-hover:bg-white/20 group-active:bg-white/30 transition-colors duration-150"
          ></div>
        </button>

        <div class="p-2">
          <div
            class="relative flex items-center bg-[#2d2d2d] rounded-lg border border-[#3c3c3c] overflow-hidden focus-within:border-[#007fd4]"
          >
            <Search size={14} class="ml-2 text-[#858585]" />
            <input
              type="text"
              placeholder="Filtrar arquivos..."
              class="w-full bg-transparent text-[#cccccc] text-[13px] px-2 py-0.75 focus:outline-none"
            />
          </div>
        </div>

        <div class="flex-1 overflow-y-auto py-2">
          <div class="text-[13px]">
            <!-- backend/Services/Messaging -->
            <div
              class="flex items-center py-1 px-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <ChevronDown size={16} class="text-[#cccccc] mr-1 shrink-0" />
              <span class="truncate"
                >{getCollapsedFolder(
                  "backend / Services / Messaging",
                  sidebarWidth,
                )}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-8 pr-3 cursor-pointer bg-[#37373d] text-[#ffffff]"
            >
              <File size={14} class="text-[#858585] mr-1.5 shrink-0" />
              <span class="truncate"
                >{getCollapsedFile(
                  "MessageProcessorService.cs",
                  32,
                  sidebarWidth,
                )}</span
              >
            </div>

            <!-- frontend -->
            <div
              class="flex items-center py-1 px-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <ChevronDown size={16} class="text-[#cccccc] mr-1 shrink-0" />
              <span class="truncate"
                >{getCollapsedFolder("frontend", sidebarWidth)}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <ChevronDown size={16} class="text-[#cccccc] mr-1 shrink-0" />
              <span class="truncate"
                >{getCollapsedFolder("src", sidebarWidth)}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-11 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <ChevronDown size={16} class="text-[#cccccc] mr-1 shrink-0" />
              <span class="truncate"
                >{getCollapsedFolder("components", sidebarWidth)}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-16 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#ff3e00] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">S</span
              >
              <span class="truncate"
                >{getCollapsedFile("AdminShell.svelte", 64, sidebarWidth)}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-11 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#519aba] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">#</span
              >
              <span class="truncate"
                >{getCollapsedFile("app.css", 44, sidebarWidth)}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#cbcb41] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">{"{}"}</span
              >
              <span class="truncate"
                >{getCollapsedFile("package-lock.json", 28, sidebarWidth)}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#cbcb41] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">{"{}"}</span
              >
              <span class="truncate"
                >{getCollapsedFile("package.json", 28, sidebarWidth)}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#51ba74] mr-1.5 font-bold text-[10px] leading-none shrink-0 border border-[#51ba74] rounded-[2px] px-[2px]"
                >M&darr;</span
              >
              <span class="truncate"
                >{getCollapsedFile(
                  "RESPONSIVE_DESIGN.md",
                  28,
                  sidebarWidth,
                )}</span
              >
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#c75fd6] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">⚡</span
              >
              <span class="truncate"
                >{getCollapsedFile("vite.config.js", 28, sidebarWidth)}</span
              >
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .always-scrollbar {
    overflow-y: scroll !important;
  }
  .always-scrollbar::-webkit-scrollbar {
    width: 10px;
    background-color: transparent;
  }
  .always-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .always-scrollbar::-webkit-scrollbar-thumb {
    background-color: rgba(121, 121, 121, 0.4);
    border: 2px solid transparent;
    background-clip: padding-box;
  }
  .always-scrollbar::-webkit-scrollbar-thumb:hover {
    background-color: rgba(100, 100, 100, 0.7);
  }
</style>
