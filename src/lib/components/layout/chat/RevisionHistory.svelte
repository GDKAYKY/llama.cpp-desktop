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
  } from "lucide-svelte";

  // We hardcode the exact representation from the image to get a 1:1 match

  let isResizing = $state(false);
  let sidebarWidth = $state(250);
  let startX = $state(0);
  let startWidth = $state(250);
  let showFileTree = $state(true);
  let showMoreMenu = $state(false);

  function startResize(e: MouseEvent) {
    isResizing = true;
    startX = e.clientX;
    startWidth = sidebarWidth;
    e.preventDefault();
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
</script>

<div
  class="flex flex-col h-full w-full text-[#cccccc] font-sans overflow-hidden text-[13px]"
>
  <!-- Top Header Bar -->
  <div
    class="flex items-center justify-between px-3 py-2 border-b border-[#333333] shrink-0"
  >
    <div class="flex items-center space-x-2 shrink-0">
      <span class="font-medium text-[#cccccc]">Não marcadas para commit</span>
      <span
        class="bg-[#333333] text-[#cccccc] px-1.5 py-0.5 rounded text-xs font-medium"
        >7</span
      >
      <ChevronDown size={14} class="text-[#cccccc]" />
      <span class="text-[#89d185] text-xs ml-2">+348</span>
      <span class="text-[#f14c4c] text-xs ml-1">-89</span>
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
        title="Ir para o arquivo"><FileSymlink size={14} /></button
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
        <Folder size={14} />
      </button>

      <button
        class="flex items-center space-x-1.5 border border-[#454545] bg-[#2d2d2d] hover:bg-[#3d3d3d] text-[#ececec] px-3 py-1 rounded-full text-xs transition-colors ml-1"
      >
        <GitCommit size={14} class="rotate-90" />
        <span>Comitar ou enviar</span>
      </button>

      <button
        class="flex items-center space-x-1.5 border border-[#333333] bg-transparent text-[#666666] px-3 py-1 rounded-full text-xs ml-1 cursor-not-allowed"
      >
        <GitPullRequest size={14} />
        <span>Criar PR</span>
      </button>
    </div>
  </div>

  <!-- Content Area (Split View) -->
  <div class="flex-1 flex overflow-hidden relative w-full">
    <!-- Left Main Area (Diff View) -->
    <div class="flex-1 flex flex-col min-w-0 relative">
      <!-- Diff Content Scroll Area -->
      <div class="flex-1 overflow-y-auto bg-[#1e1e1e] pb-24 relative">
        <!-- File 1: MessageProcessorService.cs -->
        <div class="mt-4 px-4">
          <div class="flex items-center justify-between py-1 mb-1">
            <div
              class="flex items-center space-x-2 text-[#cccccc] truncate min-w-0"
            >
              <File size={14} class="shrink-0 text-[#858585]" />
              <span class="truncate text-[13px]">
                <span class="text-[#858585]">...es/Messaging/</span><span
                  class="font-medium text-[#ececec]"
                  >MessageProcessorService.cs</span
                >
              </span>
            </div>
            <div class="flex items-center space-x-3 shrink-0 text-xs">
              <span class="text-[#89d185]">+3</span>
              <span class="text-[#f14c4c]">-2</span>
              <ExternalLink
                size={14}
                class="text-[#858585] hover:text-[#cccccc] cursor-pointer"
              />
            </div>
          </div>

          <div
            class="rounded-md border border-[#333333] bg-[#1e1e1e] overflow-hidden"
          >
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
                  <span class="text-[#d4d4d4]">
                    data.RemoteJid, data.FromMe, data.MessageType, data.Se</span
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
                  <span class="text-[#6a9955]">
                    // -- fromMe -> No Operation (fluxo continua para isMedia)</span
                  >
                </div>
              </div>

              <div class="flex bg-[#4b1818] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#f14c4c]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#4b1818]"
                >
                  60
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#c586c0]"> if</span>
                  <span class="text-[#d4d4d4]">(data.FromMe ==</span>
                  <span class="text-[#569cd6]">true</span><span
                    class="text-[#d4d4d4]">)</span
                  ><span class="text-[#ffd700]">{"{"}</span>
                </div>
              </div>
              <div class="flex bg-[#4b1818] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#f14c4c]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#4b1818]"
                >
                  61
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#6a9955]"> // return false;</span>
                </div>
              </div>

              <div class="flex bg-[#1e3422] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#89d185]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                >
                  60
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#c586c0]"> if</span>
                  <span class="text-[#d4d4d4]">(data.FromMe ==</span>
                  <span class="text-[#569cd6]">true</span><span
                    class="text-[#d4d4d4]">)</span
                  >
                </div>
              </div>
              <div class="flex bg-[#1e3422] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#89d185]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                >
                  61
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#ffd700]"> {"{"}</span>
                </div>
              </div>
              <div class="flex bg-[#1e3422] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#89d185]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                >
                  62
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#c586c0]"> return</span>
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
                  <span class="text-[#ffd700]"> {"}"}</span>
                </div>
              </div>
              <div class="flex bg-[#1e1e1e] hover:bg-[#2a2d2e]">
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none"
                >
                  64
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#6a9955]">
                    // -- isMedia --------------------------------------------</span
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
                  <span class="text-[#569cd6]"> var</span>
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
          </div>
        </div>

        <!-- File 2: AdminShell.svelte -->
        <div class="mt-6 px-4">
          <div class="flex items-center justify-between py-1 mb-1">
            <div
              class="flex items-center space-x-2 text-[#cccccc] truncate min-w-0"
            >
              <span
                class="text-[#ff3e00] shrink-0 font-bold text-sm leading-none"
                style="margin-top: 2px;">S</span
              >
              <span class="truncate text-[13px]">
                <span class="text-[#858585]">...end/src/components/</span><span
                  class="font-medium text-[#ececec]">AdminShell.svelte</span
                >
              </span>
            </div>
            <div class="flex items-center space-x-3 shrink-0 text-xs">
              <span class="text-[#89d185]">+145</span>
              <span class="text-[#f14c4c]">-68</span>
              <ExternalLink
                size={14}
                class="text-[#858585] hover:text-[#cccccc] cursor-pointer"
              />
            </div>
          </div>

          <div
            class="rounded-md border border-[#333333] bg-[#1e1e1e] overflow-hidden"
          >
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

              <div class="flex bg-[#1e3422] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#89d185]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                >
                  130
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#6a9955]"> &lt;!--</span>
                </div>
              </div>
              <div class="flex bg-[#1e3422] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#89d185]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                >
                  131
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#6a9955]">
                    RESPONSIVE LAYOUT STRATEGY:</span
                  >
                </div>
              </div>
              <div class="flex bg-[#1e3422] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#89d185]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                >
                  132
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#6a9955]">
                    - Uses viewport units (vh, vw, dvh, dvw)</span
                  >
                </div>
              </div>
              <div class="flex bg-[#1e3422] relative group">
                <div
                  class="absolute left-0 top-0 bottom-0 w-1 bg-[#89d185]"
                ></div>
                <div
                  class="w-10 shrink-0 text-right pr-3 py-0.5 text-[#858585] select-none bg-[#1e3422]"
                >
                  133
                </div>
                <div class="pl-4 py-0.5 flex-1 whitespace-pre">
                  <span class="text-[#6a9955]">
                    - Mobile-first approach with breakpoints: sm (640px), md
                    (768p</span
                  >
                </div>
              </div>
            </div>
          </div>
        </div>
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
        class="flex-col bg-[#1e1e1e] border-l border-[#333333] hidden lg:flex shrink-0 relative"
        style="width: {sidebarWidth}px"
      >
        <!-- Resize Handle -->
        <button
          type="button"
          class="absolute -left-1 top-0 bottom-0 w-2 cursor-col-resize hover:bg-[#007fd4] z-20 opacity-0 hover:opacity-100 transition-opacity border-0 p-0 bg-transparent"
          aria-label="Resize panel"
          onmousedown={startResize}
        ></button>

        <div class="p-3 border-b border-[#333333]">
          <div
            class="relative flex items-center bg-[#2d2d2d] rounded border border-[#3c3c3c] overflow-hidden focus-within:border-[#007fd4]"
          >
            <Search size={14} class="ml-2 text-[#858585]" />
            <input
              type="text"
              placeholder="Filtrar arquivos..."
              class="w-full bg-transparent text-[#cccccc] text-[13px] px-2 py-1.5 focus:outline-none"
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
              <span class="truncate">backend / Services / Messaging</span>
            </div>

            <div
              class="flex items-center py-1 pl-8 pr-3 cursor-pointer bg-[#37373d] text-[#ffffff]"
            >
              <File size={14} class="text-[#858585] mr-1.5 shrink-0" />
              <span class="truncate">MessageProcessorService.cs</span>
            </div>

            <!-- frontend -->
            <div
              class="flex items-center py-1 px-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <ChevronDown size={16} class="text-[#cccccc] mr-1 shrink-0" />
              <span class="truncate">frontend</span>
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <ChevronDown size={16} class="text-[#cccccc] mr-1 shrink-0" />
              <span class="truncate">src</span>
            </div>

            <div
              class="flex items-center py-1 pl-11 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <ChevronDown size={16} class="text-[#cccccc] mr-1 shrink-0" />
              <span class="truncate">components</span>
            </div>

            <div
              class="flex items-center py-1 pl-16 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#ff3e00] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">S</span
              >
              <span class="truncate">AdminShell.svelte</span>
            </div>

            <div
              class="flex items-center py-1 pl-11 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#519aba] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">#</span
              >
              <span class="truncate">app.css</span>
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#cbcb41] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">{"{}"}</span
              >
              <span class="truncate">package-lock.json</span>
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#cbcb41] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">{"{}"}</span
              >
              <span class="truncate">package.json</span>
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#51ba74] mr-1.5 font-bold text-[10px] leading-none shrink-0 border border-[#51ba74] rounded-[2px] px-[2px]"
                >M&darr;</span
              >
              <span class="truncate">RESPONSIVE_DESIGN.md</span>
            </div>

            <div
              class="flex items-center py-1 pl-7 pr-3 cursor-pointer hover:bg-[#2a2d2e] text-[#cccccc]"
            >
              <span
                class="text-[#c75fd6] mr-1.5 font-bold text-sm leading-none shrink-0"
                style="margin-top: 2px;">⚡</span
              >
              <span class="truncate">vite.config.js</span>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
