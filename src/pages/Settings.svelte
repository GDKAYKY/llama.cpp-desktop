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
  import { X } from "lucide-svelte";

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
    class="mb-8 flex items-center justify-between border-b border-border pb-5"
  >
    <h1 class="text-3xl font-semibold">Settings</h1>
    <div class="flex items-center gap-3">
      <button
        class="cursor-pointer rounded-lg border border-border bg-transparent px-5 py-2.5 text-sm font-medium transition-colors hover:bg-white/5 disabled:opacity-50"
        onclick={handleReset}
        disabled={loading || saving}
      >
        Reset to Defaults
      </button>
      <button
        class="cursor-pointer rounded-lg bg-primary px-5 py-2.5 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-50"
        onclick={handleSave}
        disabled={loading || saving || !unsavedChanges}
      >
        {saving ? "Saving..." : "Save Changes"}
      </button>
      <a
        href="/"
        class="flex h-10 w-10 items-center justify-center rounded-lg border border-border transition-colors hover:border-destructive hover:bg-destructive/10 hover:text-destructive"
        title="Exit Settings"
      >
        <X size={20} />
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
      <section class="rounded-xl border border-border bg-secondary p-6">
        <h2 class="mb-5 text-xl font-semibold">Models</h2>
        <div class="mb-6 flex flex-col gap-6">
          <div class="space-y-2">
            <label for="models-dir" class="block cursor-pointer">
              <span class="block text-sm font-medium">Models Directory</span>
              <span class="block text-xs text-muted-foreground"
                >Location where your models are stored</span
              >
            </label>
            <div class="flex gap-2">
              <input
                id="models-dir"
                type="text"
                value={settingsStore.settings.modelsDirectory || ""}
                placeholder="Select models directory..."
                readonly
                class="grow rounded-md border border-border bg-input px-3 py-2 text-sm text-foreground outline-none read-only:text-muted-foreground focus:border-primary"
              />
              <button
                class="cursor-pointer rounded-md border border-border bg-transparent px-4 py-2 text-sm font-medium transition-colors hover:bg-white/5"
                onclick={handleSelectModelsDirectory}
              >
                Browse
              </button>
            </div>

            <div class="space-y-2">
              <label for="llama_directory" class="block cursor-pointer">
                <span class="block text-sm font-medium"
                  >Llama Server Binary Directory</span
                >
                <span class="block text-xs text-muted-foreground"
                  >Location of llama-server executable</span
                >
              </label>
              <div class="flex gap-2">
                <input
                  id="llama_directory"
                  type="text"
                  value={settingsStore.settings.llamaDirectory || ""}
                  placeholder="Select llama-server binary path..."
                  readonly
                  class="grow rounded-md border border-border bg-input px-3 py-2 text-sm text-foreground outline-none read-only:text-muted-foreground focus:border-primary"
                />
                <button
                  class="cursor-pointer rounded-md border border-border bg-transparent px-4 py-2 text-sm font-medium transition-colors hover:bg-white/5"
                  onclick={handleSelectLlamaDirectory}
                >
                  Browse
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Appearance Section -->
      <section class="rounded-xl border border-border bg-secondary p-6">
        <h2 class="mb-5 text-xl font-semibold">Appearance</h2>
        <div class="flex flex-col gap-6">
          <div class="space-y-2">
            <label for="theme" class="block cursor-pointer">
              <span class="block text-sm font-medium">Theme</span>
              <span class="block text-xs text-muted-foreground"
                >Choose your preferred color theme</span
              >
            </label>
            <select
              id="theme"
              bind:value={settingsStore.settings.theme}
              onchange={handleChange}
              class="w-full cursor-pointer rounded-md border border-border bg-input px-3 py-2 text-sm text-foreground outline-none focus:border-primary"
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
              <option value="auto">Auto (System)</option>
            </select>
          </div>

          <div class="space-y-2">
            <label for="language" class="block cursor-pointer">
              <span class="block text-sm font-medium">Language</span>
              <span class="block text-xs text-muted-foreground"
                >Interface language</span
              >
            </label>
            <select
              id="language"
              bind:value={settingsStore.settings.language}
              onchange={handleChange}
              class="w-full cursor-pointer rounded-md border border-border bg-input px-3 py-2 text-sm text-foreground outline-none focus:border-primary"
            >
              <option value="en">English</option>
              <option value="es">Español</option>
              <option value="fr">Français</option>
              <option value="de">Deutsch</option>
              <option value="zh">中文</option>
            </select>
          </div>
        </div>
      </section>

      <!-- Model Parameters Section -->
      <section class="rounded-xl border border-border bg-secondary p-6">
        <h2 class="mb-5 text-xl font-semibold">Model Parameters</h2>
        <div class="flex flex-col gap-8">
          <div class="space-y-4">
            <label for="max-tokens" class="block cursor-pointer">
              <span class="block text-sm font-medium">Max Tokens</span>
              <span class="block text-xs text-muted-foreground"
                >Maximum number of tokens to generate (128-8192)</span
              >
            </label>
            <div class="flex items-center gap-4">
              <input
                id="max-tokens"
                type="range"
                min="128"
                max="8192"
                step="128"
                bind:value={settingsStore.settings.maxTokens}
                oninput={handleChange}
                class="h-1.5 grow cursor-pointer appearance-none rounded-full bg-border accent-primary outline-none"
              />
              <span class="min-w-[50px] text-right text-sm font-medium"
                >{settingsStore.settings.maxTokens}</span
              >
            </div>
          </div>

          <div class="space-y-4">
            <label for="temperature" class="block cursor-pointer">
              <span class="block text-sm font-medium">Temperature</span>
              <span class="block text-xs text-muted-foreground"
                >Controls randomness (0.0-2.0). Lower is more focused, higher is
                more creative</span
              >
            </label>
            <div class="flex items-center gap-4">
              <input
                id="temperature"
                type="range"
                min="0"
                max="2"
                step="0.1"
                bind:value={settingsStore.settings.temperature}
                oninput={handleChange}
                class="h-1.5 grow cursor-pointer appearance-none rounded-full bg-border accent-primary outline-none"
              />
              <span class="min-w-[50px] text-right text-sm font-medium"
                >{settingsStore.settings.temperature.toFixed(1)}</span
              >
            </div>
          </div>
        </div>
      </section>

      <!-- Chat Settings Section -->
      <section class="rounded-xl border border-border bg-secondary p-6">
        <h2 class="mb-5 text-xl font-semibold">Chat Settings</h2>
        <div class="flex flex-col gap-8">
          <div class="flex items-start gap-3">
            <input
              type="checkbox"
              id="auto-save"
              bind:checked={settingsStore.settings.autoSaveChat}
              onchange={handleChange}
              class="mt-1 h-5 w-5 cursor-pointer accent-primary"
            />
            <label for="auto-save" class="block cursor-pointer">
              <span class="block text-sm font-medium"
                >Auto-save Chat History</span
              >
              <span class="block text-xs text-muted-foreground"
                >Automatically save conversations</span
              >
            </label>
          </div>

          <div class="space-y-4">
            <label for="history-limit" class="block cursor-pointer">
              <span class="block text-sm font-medium">Chat History Limit</span>
              <span class="block text-xs text-muted-foreground"
                >Number of recent chats to keep (10-100)</span
              >
            </label>
            <div class="flex items-center gap-4">
              <input
                id="history-limit"
                type="range"
                min="10"
                max="100"
                step="5"
                bind:value={settingsStore.settings.chatHistoryLimit}
                oninput={handleChange}
                class="h-1.5 grow cursor-pointer appearance-none rounded-full bg-border accent-primary outline-none"
              />
              <span class="min-w-[50px] text-right text-sm font-medium"
                >{settingsStore.settings.chatHistoryLimit}</span
              >
            </div>
          </div>
        </div>
      </section>

      <!-- About Section -->
      <section class="rounded-xl border border-border bg-secondary p-6">
        <h2 class="mb-5 text-xl font-semibold">About</h2>
        <div class="space-y-2 text-sm leading-relaxed text-muted-foreground">
          <p><strong class="text-foreground">Llama Desktop</strong></p>
          <p>Version 0.1.0</p>
          {#if configPath}
            <div class="mt-4 border-t border-border pt-4">
              <p class="mb-2"><strong>Config file:</strong></p>
              <code class="block break-all rounded bg-input p-2 text-xs"
                >{configPath}</code
              >
            </div>
          {/if}
        </div>
      </section>
    </div>
  {/if}
</div>
