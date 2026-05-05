<script>
  import { onMount } from "svelte";
  import {
    selectModelsDirectory,
    selectLlamaDirectory,
  } from "$lib/services/models";
  import { getConfigPath } from "$lib/config/index";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { modelsStore } from "$lib/stores/models.svelte";
  import { cn } from "$shared/cn.js";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { Switch, Slider } from "bits-ui";
  import {
    X,
    Save,
    RotateCcw,
    Box,
    Palette,
    Sliders as SlidersIcon,
    MessageSquare,
    Info,
    FolderOpen,
    Globe,
    Monitor,
    Thermometer,
    Hash,
    Cpu,
    Search,
    FileCode,
    Moon,
    Sun,
  } from "lucide-svelte";
  import Dropdown from "../components/ui/Dropdown.svelte";

  let configPath = $state("");
  let loading = $state(false);
  let saving = $state(false);
  let message = $state({ type: "", text: "" });
  let unsavedChanges = $state(false);

  // Slider values as arrays for bits-ui
  let maxTokensValue = $state([settingsStore.settings.maxTokens]);
  let temperatureValue = $state([settingsStore.settings.temperature]);
  let contextSizeValue = $state([settingsStore.settings.contextSize]);
  let historyLimitValue = $state([settingsStore.settings.chatHistoryLimit]);

  $effect(() => {
    maxTokensValue = [settingsStore.settings.maxTokens];
    temperatureValue = [settingsStore.settings.temperature];
    contextSizeValue = [settingsStore.settings.contextSize];
    historyLimitValue = [settingsStore.settings.chatHistoryLimit];
  });

  const themeItems = [
    { label: "Dark Mode", value: "dark", icon: Moon },
    { label: "Light Mode", value: "light", icon: Sun },
    { label: "System Default", value: "auto", icon: Monitor },
  ];

  const languageItems = [
    { label: "English (US)", value: "en" },
    { label: "Español", value: "es" },
    { label: "Français", value: "fr" },
    { label: "Deutsch", value: "de" },
    { label: "中文", value: "zh" },
  ];

  const providerItems = [
    { label: "Tavily (default)", value: "tavily" },
    { label: "Custom MCP", value: "custom" },
  ];

  onMount(async () => {
    loading = true;
    await settingsStore.init();
    if (settingsStore.error) {
      showMessage("error", settingsStore.error);
    }
    await loadConfigPath();
    loading = false;
  });

  async function loadConfigPath() {
    try {
      configPath = await getConfigPath();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      showMessage("error", `Failed to load config path: ${errorMessage}`);
    }
  }

  async function handleSave() {
    try {
      saving = true;
      const configObj = {
        modelsDirectory: settingsStore.settings.modelsDirectory,
        llamaDirectory: settingsStore.settings.llamaDirectory,
        theme: settingsStore.settings.theme,
        language: settingsStore.settings.language,
        maxTokens: maxTokensValue[0],
        contextSize: contextSizeValue[0],
        temperature: temperatureValue[0],
        autoSaveChat: settingsStore.settings.autoSaveChat,
        chatHistoryLimit: historyLimitValue[0],
        serverPort: settingsStore.settings.serverPort,
        webSearchProvider: settingsStore.settings.webSearchProvider,
        webSearchMcpId: settingsStore.settings.webSearchMcpId,
      };
      await settingsStore.update(configObj);
      await modelsStore.refresh();
      unsavedChanges = false;
      showMessage("success", "Configuration saved successfully!");
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      showMessage("error", `Failed to save configuration: ${errorMessage}`);
    } finally {
      saving = false;
    }
  }

  async function handleReset() {
    if (!confirm("Are you sure you want to reset all settings to defaults?")) {
      return;
    }

    try {
      loading = true;
      await settingsStore.reset();
      await modelsStore.refresh();
      unsavedChanges = false;
      showMessage("success", "Configuration reset to defaults");
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      showMessage("error", `Failed to reset configuration: ${errorMessage}`);
    } finally {
      loading = false;
    }
  }

  async function handleSelectModelsDirectory() {
    try {
      await modelsStore.selectDirectory();
      if (modelsStore.modelsRoot) {
        unsavedChanges = true;
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      showMessage("error", `Failed to select directory: ${errorMessage}`);
    }
  }

  async function handleSelectLlamaDirectory() {
    try {
      const selected = await selectLlamaDirectory();
      if (selected) {
        settingsStore.settings.llamaDirectory = selected;
        unsavedChanges = true;
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      showMessage("error", `Failed to select binary: ${errorMessage}`);
    }
  }

  function showMessage(type, text) {
    message = { type, text };
    setTimeout(() => {
      message = { type: "", text: "" };
    }, 5000);
  }

  function handleChange() {
    unsavedChanges = true;
  }

  async function handleOpenConfigFile() {
    if (!configPath) return;
    try {
      await openPath(configPath);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      showMessage("error", `Failed to open config file: ${errorMessage}`);
    }
  }
</script>

<div class="p-6">
  <div class="mb-8 flex items-center justify-between pb-6">
    <div>
      <div class="flex items-center gap-3">
        <div
          class="flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10 text-primary"
        >
          <SlidersIcon size={20} />
        </div>
        <h1 class="text-3xl font-bold tracking-tight">Settings</h1>
      </div>
      <p class="mt-1 text-sm text-muted-foreground">
        Manage your preferences and application configuration
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button
        class="inline-flex items-center gap-2 rounded-lg bg-transparent px-4 py-2 text-sm font-medium transition-colors hover:bg-muted disabled:opacity-50"
        onclick={handleReset}
        disabled={loading || saving}
      >
        <RotateCcw size={16} />
        <span class="hidden sm:inline">Reset</span>
      </button>
      <button
        class="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-50"
        onclick={handleSave}
        disabled={loading || saving || !unsavedChanges}
      >
        {#if saving}
          <div
            class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
          ></div>
        {:else}
          <Save size={16} />
        {/if}
        {saving ? "Saving..." : "Save"}
      </button>
      <a
        href="/"
        class="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive"
      >
        <X size={18} />
      </a>
    </div>
  </div>

  {#if message.text}
    <div
      class={cn(
        "mb-5 rounded-lg px-4 py-3 text-sm",
        message.type === "success"
          ? "bg-green-500/10 text-green-400"
          : "bg-red-500/10 text-red-400",
      )}
    >
      {message.text}
    </div>
  {/if}

  {#if loading}
    <div class="py-10 text-center text-muted-foreground">
      Loading configuration...
    </div>
  {:else}
    <div class="flex flex-col gap-6">
      <!-- Models Section -->
      <section class="rounded-xl bg-card p-6 shadow-sm">
        <div class="mb-6 flex items-center gap-3 pb-4">
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500"
          >
            <Box size={18} />
          </div>
          <div>
            <h2 class="text-lg font-semibold leading-tight">
              Models Configuration
            </h2>
            <p class="text-xs text-muted-foreground leading-relaxed">
              Manage paths for models and executables
            </p>
          </div>
        </div>

        <div class="flex flex-col gap-5">
          <div class="space-y-2">
            <label
              for="models-dir"
              class="flex items-center gap-2 text-sm font-medium"
            >
              <FolderOpen size={14} class="text-muted-foreground" />
              Models Directory
            </label>
            <div class="flex gap-2">
              <input
                id="models-dir"
                type="text"
                value={settingsStore.settings.modelsDirectory || ""}
                placeholder="Select models directory..."
                readonly
                class="flex-1 rounded-md bg-muted/50 px-3 py-2 text-sm text-foreground outline-none transition-all focus:ring-1 focus:ring-primary/20"
              />
              <button
                class="inline-flex items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground"
                onclick={handleSelectModelsDirectory}
              >
                Browse
              </button>
            </div>
          </div>

          <div class="space-y-2">
            <label
              for="llama_directory"
              class="flex items-center gap-2 text-sm font-medium"
            >
              <Cpu size={14} class="text-muted-foreground" />
              Llama.cpp Binary Directory
            </label>
            <div class="flex gap-2">
              <input
                id="llama_directory"
                type="text"
                value={settingsStore.settings.llamaDirectory || ""}
                placeholder="Select llama-server binary path..."
                readonly
                class="flex-1 rounded-md bg-muted/50 px-3 py-2 text-sm text-foreground outline-none transition-all focus:ring-1 focus:ring-primary/20"
              />
              <button
                class="inline-flex items-center justify-center rounded-md bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground"
                onclick={handleSelectLlamaDirectory}
              >
                Browse
              </button>
            </div>
            <p class="text-xs text-muted-foreground">
              Path to the backend executable (llama-server)
            </p>
          </div>
        </div>
      </section>

      <!-- Appearance Section -->
      <section class="rounded-xl bg-card p-6 shadow-sm">
        <div class="mb-6 flex items-center gap-3 pb-4">
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-purple-500/10 text-purple-500"
          >
            <Palette size={18} />
          </div>
          <div>
            <h2 class="text-lg font-semibold leading-tight">Appearance</h2>
            <p class="text-xs text-muted-foreground leading-relaxed">
              Customize the look and feel
            </p>
          </div>
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <Dropdown
            label="Theme"
            items={themeItems}
            bind:value={settingsStore.settings.theme}
            onSelect={handleChange}
            placeholder="Select theme"
          />
          <Dropdown
            label="Language"
            items={languageItems}
            bind:value={settingsStore.settings.language}
            onSelect={handleChange}
            placeholder="Select language"
          />
        </div>
      </section>

      <!-- Model Parameters Section -->
      <section class="rounded-xl bg-card p-6 shadow-sm">
        <div class="mb-6 flex items-center gap-3 pb-4">
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-orange-500/10 text-orange-500"
          >
            <SlidersIcon size={18} />
          </div>
          <div>
            <h2 class="text-lg font-semibold leading-tight">
              Inference Parameters
            </h2>
            <p class="text-xs text-muted-foreground leading-relaxed">
              Fine-tune model generation behavior
            </p>
          </div>
        </div>

        <div class="space-y-6">
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <label class="flex items-center gap-2 text-sm font-medium">
                <Hash size={14} class="text-muted-foreground" />
                Max Tokens
              </label>
              <span class="rounded bg-muted px-2 py-0.5 text-xs font-mono">
                {maxTokensValue[0]}
              </span>
            </div>
            <Slider.Root
              bind:value={maxTokensValue}
              min={128}
              max={8192}
              step={128}
              onValueChange={handleChange}
              class="relative flex w-full touch-none select-none items-center"
            >
              <Slider.Track
                class="relative h-2 w-full grow overflow-hidden rounded-full bg-secondary"
              >
                <Slider.Range class="absolute h-full bg-primary" />
              </Slider.Track>
              <Slider.Thumb
                class="block h-5 w-5 rounded-full border-2 border-primary bg-background ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
              />
            </Slider.Root>
            <p class="text-xs text-muted-foreground">
              Upper limit on generated response length (128-8192)
            </p>
          </div>

          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <label class="flex items-center gap-2 text-sm font-medium">
                <Thermometer size={14} class="text-muted-foreground" />
                Temperature
              </label>
              <span class="rounded bg-muted px-2 py-0.5 text-xs font-mono">
                {temperatureValue[0].toFixed(1)}
              </span>
            </div>
            <Slider.Root
              bind:value={temperatureValue}
              min={0}
              max={2}
              step={0.1}
              onValueChange={handleChange}
              class="relative flex w-full touch-none select-none items-center"
            >
              <Slider.Track
                class="relative h-2 w-full grow overflow-hidden rounded-full bg-secondary"
              >
                <Slider.Range class="absolute h-full bg-primary" />
              </Slider.Track>
              <Slider.Thumb
                class="block h-5 w-5 rounded-full border-2 border-primary bg-background ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
              />
            </Slider.Root>
            <p class="text-xs text-muted-foreground">
              Creativity vs Focus ({temperatureValue[0] < 0.7
                ? "Precise"
                : temperatureValue[0] > 1.2
                  ? "Creative"
                  : "Balanced"})
            </p>
          </div>

          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <label class="flex items-center gap-2 text-sm font-medium">
                <Hash size={14} class="text-muted-foreground" />
                Context Size
              </label>
              <span class="rounded bg-muted px-2 py-0.5 text-xs font-mono">
                {contextSizeValue[0]}
              </span>
            </div>
            <Slider.Root
              bind:value={contextSizeValue}
              min={1024}
              max={32768}
              step={1024}
              onValueChange={handleChange}
              class="relative flex w-full touch-none select-none items-center"
            >
              <Slider.Track
                class="relative h-2 w-full grow overflow-hidden rounded-full bg-secondary"
              >
                <Slider.Range class="absolute h-full bg-primary" />
              </Slider.Track>
              <Slider.Thumb
                class="block h-5 w-5 rounded-full border-2 border-primary bg-background ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
              />
            </Slider.Root>
            <p class="text-xs text-muted-foreground">
              Total context window size for the model (1024-32768)
            </p>
          </div>
        </div>
      </section>

      <!-- Chat & System Section -->
      <section class="rounded-xl bg-card p-6 shadow-sm">
        <div class="mb-6 flex items-center gap-3 pb-4">
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-green-500/10 text-green-500"
          >
            <MessageSquare size={18} />
          </div>
          <div>
            <h2 class="text-lg font-semibold leading-tight">System & Chat</h2>
            <p class="text-xs text-muted-foreground leading-relaxed">
              Session handling and history
            </p>
          </div>
        </div>

        <div class="space-y-6">
          <div class="flex items-center justify-between space-x-4">
            <label for="auto-save" class="flex flex-col space-y-1">
              <span class="text-sm font-medium">Auto-save Chat History</span>
              <span class="text-xs text-muted-foreground">
                Automatically save conversations locally for continuity
              </span>
            </label>
            <Switch.Root
              id="auto-save"
              checked={settingsStore.settings.autoSaveChat}
              onCheckedChange={(checked) => {
                settingsStore.settings.autoSaveChat = checked;
                handleChange();
              }}
              class="peer inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input"
            >
              <Switch.Thumb
                class="pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0"
              />
            </Switch.Root>
          </div>

          <div class="h-px bg-border"></div>

          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium">Chat History Retention</span>
              <span class="rounded bg-muted px-2 py-0.5 text-xs font-mono">
                {historyLimitValue[0]} items
              </span>
            </div>
            <Slider.Root
              bind:value={historyLimitValue}
              min={10}
              max={100}
              step={5}
              onValueChange={handleChange}
              class="relative flex w-full touch-none select-none items-center"
            >
              <Slider.Track
                class="relative h-2 w-full grow overflow-hidden rounded-full bg-secondary"
              >
                <Slider.Range class="absolute h-full bg-primary" />
              </Slider.Track>
              <Slider.Thumb
                class="block h-5 w-5 rounded-full border-2 border-primary bg-background ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
              />
            </Slider.Root>
            <p class="text-xs text-muted-foreground">
              Limit the number of recent chats stored (10-100)
            </p>
          </div>
        </div>
      </section>

      <!-- Web Search Section -->
      <section class="rounded-xl bg-card p-6 shadow-sm">
        <div class="mb-6 flex items-center gap-3 pb-4">
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-cyan-500/10 text-cyan-500"
          >
            <Search size={18} />
          </div>
          <div>
            <h2 class="text-lg font-semibold leading-tight">Web Search</h2>
            <p class="text-xs text-muted-foreground leading-relaxed">
              Choose the MCP server used for web search
            </p>
          </div>
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Dropdown
              label="Default Provider"
              items={providerItems}
              bind:value={settingsStore.settings.webSearchProvider}
              onSelect={handleChange}
              placeholder="Select provider"
            />
            <p class="text-xs text-muted-foreground">
              Tavily uses MCP server id <span class="font-mono">tavily</span>
            </p>
          </div>

          <div class="space-y-2">
            <label
              for="web-search-mcp"
              class="flex items-center gap-2 text-sm font-medium"
            >
              <Globe size={14} class="text-muted-foreground" />
              Custom MCP ID
            </label>
            <input
              id="web-search-mcp"
              type="text"
              value={settingsStore.settings.webSearchMcpId ?? ""}
              oninput={(e) => {
                settingsStore.settings.webSearchMcpId =
                  e.currentTarget.value || null;
                handleChange();
              }}
              placeholder="my-web-search-mcp"
              disabled={settingsStore.settings.webSearchProvider !== "custom"}
              class="w-full rounded-md bg-muted/50 px-3 py-2 text-sm text-foreground outline-none transition-all focus:ring-1 focus:ring-primary/20 disabled:opacity-60"
            />
            <p class="text-xs text-muted-foreground">
              When custom is selected, this MCP server id will be used
            </p>
          </div>
        </div>
      </section>

      <!-- About Section -->
      <section class="rounded-xl bg-card p-6 shadow-sm">
        <div class="mb-4 flex items-center gap-3">
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-slate-500/10 text-slate-500"
          >
            <Info size={18} />
          </div>
          <h2 class="text-lg font-semibold leading-tight">About</h2>
        </div>

        <div class="space-y-4">
          <div class="flex justify-between pb-2">
            <span class="font-medium">Llama Desktop</span>
            <span class="text-sm text-muted-foreground">v0.1.0</span>
          </div>
          {#if configPath}
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <span
                  class="text-xs font-medium uppercase tracking-wider text-muted-foreground"
                >
                  Configuration File
                </span>
                <button
                  class="inline-flex items-center gap-2 rounded-md bg-transparent px-3 py-1.5 text-xs font-medium transition-colors hover:bg-muted"
                  onclick={handleOpenConfigFile}
                >
                  <FileCode size={12} />
                  Open File
                </button>
              </div>
              <code
                class="block break-all rounded-md bg-muted px-3 py-2 text-xs font-mono"
              >
                {configPath}
              </code>
            </div>
          {/if}
        </div>
      </section>
    </div>
  {/if}
</div>
