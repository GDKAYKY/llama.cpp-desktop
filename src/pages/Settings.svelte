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
  import {
    X,
    Save,
    RotateCcw,
    Box,
    Palette,
    Sliders,
    MessageSquare,
    Info,
    FolderOpen,
    Globe,
    Monitor,
    Zap,
    Thermometer,
    Hash,
    Cpu,
  } from "lucide-svelte";

  let configPath = $state("");
  let loading = $state(false);
  let saving = $state(false);
  /** @type {{ type: string, text: string }} */
  let message = $state({ type: "", text: "" });
  let unsavedChanges = $state(false);

  onMount(async () => {
    loading = true;
    await settingsStore.init();
    await loadConfigPath();
    loading = false;
  });

  async function loadConfigPath() {
    try {
      configPath = await getConfigPath();
    } catch (err) {
      // Silence error as requested
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
        maxTokens: settingsStore.settings.maxTokens,
        temperature: settingsStore.settings.temperature,
        autoSaveChat: settingsStore.settings.autoSaveChat,
        chatHistoryLimit: settingsStore.settings.chatHistoryLimit,
        serverPort: settingsStore.settings.serverPort,
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

  /**
   * @param {string} type
   * @param {string} text
   */
  function showMessage(type, text) {
    message = { type, text };
    setTimeout(() => {
      message = { type: "", text: "" };
    }, 5000);
  }

  function handleChange() {
    unsavedChanges = true;
  }
</script>

<div
  class="mx-auto h-full max-w-[900px] overflow-y-auto px-5 py-10 text-foreground"
>
  <div
    class="mb-8 flex items-center justify-between border-b border-border/60 pb-6"
  >
    <div>
      <h1
        class="text-3xl font-bold tracking-tight leading-none text-foreground"
      >
        Settings
      </h1>
      <p class="mt-1 text-sm text-muted-foreground leading-normal">
        Manage your preferences and application configuration
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button
        class="inline-flex items-center gap-2 rounded-lg border border-border bg-transparent px-4 py-2 text-sm font-medium transition-colors hover:bg-muted disabled:opacity-50"
        onclick={handleReset}
        disabled={loading || saving}
        aria-label="Reset to Defaults"
      >
        <RotateCcw size={16} />
        <span class="hidden sm:inline">Reset</span>
      </button>
      <button
        class="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-50"
        onclick={handleSave}
        disabled={loading || saving || !unsavedChanges}
        aria-label="Save Changes"
      >
        {#if saving}
          <div
            class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
          />
        {:else}
          <Save size={16} />
        {/if}
        <span>{saving ? "Saving..." : "Save Changes"}</span>
      </button>
      <a
        href="/"
        class="flex h-9 w-9 items-center justify-center rounded-lg border border-border text-muted-foreground transition-colors hover:border-destructive hover:bg-destructive/10 hover:text-destructive"
        title="Exit Settings"
        aria-label="Exit Settings"
      >
        <X size={18} />
      </a>
    </div>
  </div>

  {#if message.text}
    <div
      class={cn(
        "mb-5 rounded-lg border px-4 py-3 text-sm",
        message.type === "success"
          ? "border-green-500/30 bg-green-500/10 text-green-400"
          : "border-red-500/30 bg-red-500/10 text-red-400",
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
    <div class="flex flex-col gap-8">
      <!-- Models Section -->
      <section class="rounded-xl border border-border/60 bg-card p-6 shadow-sm">
        <div
          class="mb-6 flex items-center gap-3 border-b border-border/40 pb-4"
        >
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
          <div class="space-y-1.5">
            <label for="models-dir" class="block cursor-pointer">
              <span
                class="flex items-center gap-2 text-sm font-medium leading-none"
              >
                <FolderOpen size={14} class="text-muted-foreground" />
                Models Directory
              </span>
            </label>
            <div class="flex gap-2">
              <input
                id="models-dir"
                type="text"
                value={settingsStore.settings.modelsDirectory || ""}
                placeholder="Select models directory..."
                readonly
                class="grow rounded-md border border-border bg-muted/50 px-3 py-2 text-sm text-foreground outline-none transition-all focus:border-primary focus:ring-1 focus:ring-primary/20"
              />
              <button
                class="inline-flex cursor-pointer items-center justify-center rounded-md border border-border bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground"
                onclick={handleSelectModelsDirectory}
              >
                Browse
              </button>
            </div>
          </div>

          <div class="space-y-1.5">
            <label for="llama_directory" class="block cursor-pointer">
              <span
                class="flex items-center gap-2 text-sm font-medium leading-none"
              >
                <Cpu size={14} class="text-muted-foreground" />
                Llama Server Binary
              </span>
            </label>
            <div class="flex gap-2">
              <input
                id="llama_directory"
                type="text"
                value={settingsStore.settings.llamaDirectory || ""}
                placeholder="Select llama-server binary path..."
                readonly
                class="grow rounded-md border border-border bg-muted/50 px-3 py-2 text-sm text-foreground outline-none transition-all focus:border-primary focus:ring-1 focus:ring-primary/20"
              />
              <button
                class="inline-flex cursor-pointer items-center justify-center rounded-md border border-border bg-background px-4 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground"
                onclick={handleSelectLlamaDirectory}
              >
                Browse
              </button>
            </div>
            <p class="text-xs text-muted-foreground leading-relaxed pl-1">
              Path to the backend executable (llama-server)
            </p>
          </div>
        </div>
      </section>

      <!-- Appearance Section -->
      <section class="rounded-xl border border-border/60 bg-card p-6 shadow-sm">
        <div
          class="mb-6 flex items-center gap-3 border-b border-border/40 pb-4"
        >
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

        <div class="grid gap-6 sm:grid-cols-2">
          <div class="space-y-2">
            <label for="theme" class="block cursor-pointer">
              <span
                class="flex items-center gap-2 text-sm font-medium leading-none"
              >
                <Monitor size={14} class="text-muted-foreground" />
                Theme
              </span>
            </label>
            <div class="relative">
              <select
                id="theme"
                bind:value={settingsStore.settings.theme}
                onchange={handleChange}
                class="w-full cursor-pointer appearance-none rounded-md border border-border bg-muted/50 px-3 py-2 text-sm text-foreground outline-none transition-all focus:border-primary focus:ring-1 focus:ring-primary/20"
              >
                <option value="dark">Dark Mode</option>
                <option value="light">Light Mode</option>
                <option value="auto">System Default</option>
              </select>
              <div
                class="pointer-events-none absolute inset-y-0 right-3 flex items-center text-muted-foreground"
              >
                <svg
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                  ><path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 9l-7 7-7-7"
                  /></svg
                >
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <label for="language" class="block cursor-pointer">
              <span
                class="flex items-center gap-2 text-sm font-medium leading-none"
              >
                <Globe size={14} class="text-muted-foreground" />
                Language
              </span>
            </label>
            <div class="relative">
              <select
                id="language"
                bind:value={settingsStore.settings.language}
                onchange={handleChange}
                class="w-full cursor-pointer appearance-none rounded-md border border-border bg-muted/50 px-3 py-2 text-sm text-foreground outline-none transition-all focus:border-primary focus:ring-1 focus:ring-primary/20"
              >
                <option value="en">English (US)</option>
                <option value="es">Español</option>
                <option value="fr">Français</option>
                <option value="de">Deutsch</option>
                <option value="zh">中文</option>
              </select>
              <div
                class="pointer-events-none absolute inset-y-0 right-3 flex items-center text-muted-foreground"
              >
                <svg
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                  ><path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 9l-7 7-7-7"
                  /></svg
                >
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Model Parameters Section -->
      <section class="rounded-xl border border-border/60 bg-card p-6 shadow-sm">
        <div
          class="mb-6 flex items-center gap-3 border-b border-border/40 pb-4"
        >
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-orange-500/10 text-orange-500"
          >
            <Sliders size={18} />
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

        <div class="grid gap-8 sm:grid-cols-2">
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <label for="max-tokens" class="block cursor-pointer">
                <span
                  class="flex items-center gap-2 text-sm font-medium leading-none"
                >
                  <Hash size={14} class="text-muted-foreground" />
                  Max Tokens
                </span>
              </label>
              <span
                class="rounded bg-muted px-2 py-0.5 text-xs font-mono text-foreground"
              >
                {settingsStore.settings.maxTokens}
              </span>
            </div>
            <input
              id="max-tokens"
              type="range"
              min="128"
              max="8192"
              step="128"
              bind:value={settingsStore.settings.maxTokens}
              oninput={handleChange}
              class="h-2 w-full cursor-pointer appearance-none rounded-full bg-secondary accent-primary outline-none"
            />
            <p class="text-xs text-muted-foreground leading-relaxed">
              Upper limit on generated response length (128-8192)
            </p>
          </div>

          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <label for="temperature" class="block cursor-pointer">
                <span
                  class="flex items-center gap-2 text-sm font-medium leading-none"
                >
                  <Thermometer size={14} class="text-muted-foreground" />
                  Temperature
                </span>
              </label>
              <span
                class="rounded bg-muted px-2 py-0.5 text-xs font-mono text-foreground"
              >
                {settingsStore.settings.temperature.toFixed(1)}
              </span>
            </div>
            <input
              id="temperature"
              type="range"
              min="0"
              max="2"
              step="0.1"
              bind:value={settingsStore.settings.temperature}
              oninput={handleChange}
              class="h-2 w-full cursor-pointer appearance-none rounded-full bg-secondary accent-primary outline-none"
            />
            <p class="text-xs text-muted-foreground leading-relaxed">
              Creativity vs Focus ({settingsStore.settings.temperature < 0.7
                ? "Precise"
                : settingsStore.settings.temperature > 1.2
                  ? "Creative"
                  : "Balanced"})
            </p>
          </div>
        </div>
      </section>

      <!-- Chat & System Section -->
      <section class="rounded-xl border border-border/60 bg-card p-6 shadow-sm">
        <div
          class="mb-6 flex items-center gap-3 border-b border-border/40 pb-4"
        >
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

        <div class="flex flex-col gap-6">
          <div
            class="flex items-start gap-3 rounded-lg border border-border/40 bg-muted/20 p-4 transition-colors hover:bg-muted/40"
          >
            <input
              type="checkbox"
              id="auto-save"
              bind:checked={settingsStore.settings.autoSaveChat}
              onchange={handleChange}
              class="mt-1 h-4 w-4 cursor-pointer rounded border-border text-primary focus:ring-primary"
            />
            <label for="auto-save" class="block cursor-pointer">
              <span class="block text-sm font-medium leading-none"
                >Auto-save Chat History</span
              >
              <span
                class="mt-1 block text-xs text-muted-foreground leading-relaxed"
                >Automatically save conversations locally for continuity.</span
              >
            </label>
          </div>

          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <label for="history-limit" class="block cursor-pointer">
                <span class="block text-sm font-medium leading-none"
                  >Chat History Retention</span
                >
              </label>
              <span
                class="rounded bg-muted px-2 py-0.5 text-xs font-mono text-foreground"
              >
                {settingsStore.settings.chatHistoryLimit} items
              </span>
            </div>
            <input
              id="history-limit"
              type="range"
              min="10"
              max="100"
              step="5"
              bind:value={settingsStore.settings.chatHistoryLimit}
              oninput={handleChange}
              class="h-2 w-full cursor-pointer appearance-none rounded-full bg-secondary accent-primary outline-none"
            />
            <p class="text-xs text-muted-foreground leading-relaxed">
              Limit the number of recent chats stored (10-100)
            </p>
          </div>
        </div>
      </section>

      <!-- About Section -->
      <section class="rounded-xl border border-border/60 bg-card p-6 shadow-sm">
        <div class="mb-4 flex items-center gap-3">
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg bg-slate-500/10 text-slate-500"
          >
            <Info size={18} />
          </div>
          <h2 class="text-lg font-semibold leading-tight">About</h2>
        </div>

        <div class="space-y-4 text-sm text-muted-foreground">
          <div class="flex justify-between border-b border-border/40 pb-2">
            <span class="font-medium text-foreground">Llama Desktop</span>
            <span>v0.1.0</span>
          </div>
          {#if configPath}
            <div>
              <p
                class="mb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground"
              >
                Configuration File
              </p>
              <code
                class="block break-all rounded-md bg-muted px-3 py-2 text-xs font-mono text-foreground"
                >{configPath}</code
              >
            </div>
          {/if}
        </div>
      </section>
    </div>
  {/if}
</div>
