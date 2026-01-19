<script>
    import { onMount } from "svelte";
    import {
        loadConfig,
        saveConfig,
        resetConfig,
        getConfigPath,
    } from "$lib/config.js";
    import { selectModelsDirectory } from "$lib/models.js";

    /** @type {{ models_directory: string | null, llamaPath: string | null, theme: string, language: string, max_tokens: number, temperature: number, auto_save_chat: boolean, chat_history_limit: number }} */
    let config = $state({
        models_directory: null,
        llamaPath: null,
        theme: "dark",
        language: "en",
        max_tokens: 2048,
        temperature: 0.7,
        auto_save_chat: true,
        chat_history_limit: 50,
    });

    let configPath = $state("");
    let loading = $state(false);
    let saving = $state(false);
    /** @type {{ type: string, text: string }} */
    let message = $state({ type: "", text: "" });
    let unsavedChanges = $state(false);

    onMount(async () => {
        await loadConfiguration();
        await loadConfigPath();
    });

    async function loadConfiguration() {
        try {
            loading = true;
            config = (await loadConfig()) || config;
        } catch (err) {
            const errorMessage =
                err instanceof Error ? err.message : String(err);
            showMessage(
                "error",
                `Failed to load configuration: ${errorMessage}`,
            );
        } finally {
            loading = false;
        }
    }

    async function loadConfigPath() {
        try {
            configPath = await getConfigPath();
        } catch (err) {
            console.error("Failed to get config path:", err);
        }
    }

    async function handleSave() {
        try {
            saving = true;
            await saveConfig(config);
            unsavedChanges = false;
            showMessage("success", "Configuration saved successfully!");
        } catch (err) {
            const errorMessage =
                err instanceof Error ? err.message : String(err);
            showMessage(
                "error",
                `Failed to save configuration: ${errorMessage}`,
            );
        } finally {
            saving = false;
        }
    }

    async function handleReset() {
        if (
            !confirm("Are you sure you want to reset all settings to defaults?")
        ) {
            return;
        }

        try {
            loading = true;
            config = (await resetConfig()) || config;
            unsavedChanges = false;
            showMessage("success", "Configuration reset to defaults");
        } catch (err) {
            const errorMessage =
                err instanceof Error ? err.message : String(err);
            showMessage(
                "error",
                `Failed to reset configuration: ${errorMessage}`,
            );
        } finally {
            loading = false;
        }
    }

    async function handleSelectModelsDirectory() {
        try {
            const selected = await selectModelsDirectory();
            if (selected) {
                config.models_directory = selected;
                unsavedChanges = true;
            }
        } catch (err) {
            const errorMessage =
                err instanceof Error ? err.message : String(err);
            showMessage("error", `Failed to select directory: ${errorMessage}`);
            console.error(err);
        }
    }

    async function handleSelectLlamaPath() {
        try {
            const selected = await selectModelsDirectory();
            if (selected) {
                config.llamaPath = selected;
                unsavedChanges = true;
            }
        } catch (err) {
            const errorMessage =
                err instanceof Error ? err.message : String(err);
            showMessage("error", `Failed to select directory: ${errorMessage}`);
            console.error(err);
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

<svelte:head>
    <title>Settings - Llama Desktop</title>
</svelte:head>

<div class="settings-container">
    <div class="settings-header">
        <h1>Settings</h1>
        <div class="header-actions">
            <button
                class="btn-secondary"
                onclick={handleReset}
                disabled={loading || saving}
            >
                Reset to Defaults
            </button>
            <button
                class="btn-primary"
                onclick={handleSave}
                disabled={loading || saving || !unsavedChanges}
            >
                {saving ? "Saving..." : "Save Changes"}
            </button>
            <a href="/" class="btn-exit" title="Exit Settings">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                >
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
            </a>
        </div>
    </div>

    {#if message.text}
        <div class="message {message.type}">
            {message.text}
        </div>
    {/if}

    {#if loading}
        <div class="loading">Loading configuration...</div>
    {:else}
        <div class="settings-content">
            <!-- Models Section -->
            <section class="settings-section">
                <h2>Models</h2>
                <div class="setting-group">
                    <label for="models-dir">
                        <span class="label-text">Models Directory</span>
                        <span class="label-description"
                            >Location where your Ollama models are stored</span
                        >
                    </label>
                    <div class="input-with-button">
                        <input
                            id="models-dir"
                            type="text"
                            value={config.models_directory || ""}
                            placeholder="Select models directory..."
                            readonly
                        />
                        <button
                            class="btn-secondary"
                            onclick={handleSelectModelsDirectory}
                        >
                            Browse
                        </button>
                    </div>
                </div>

                <div class="setting-group">
                    <label for="llama-path">
                        <span class="label-text">llama.cpp Path</span>
                        <span class="label-description"
                            >Location of llama.cpp executables folder</span
                        >
                    </label>
                    <div class="input-with-button">
                        <input
                            id="llama-path"
                            type="text"
                            value={config.llamaPath || ""}
                            placeholder="Select llama.cpp directory..."
                            readonly
                        />
                        <button
                            class="btn-secondary"
                            onclick={handleSelectLlamaPath}
                        >
                            Browse
                        </button>
                    </div>
                </div>
            </section>

            <!-- Appearance Section -->
            <section class="settings-section">
                <h2>Appearance</h2>
                <div class="setting-group">
                    <label for="theme">
                        <span class="label-text">Theme</span>
                        <span class="label-description"
                            >Choose your preferred color theme</span
                        >
                    </label>
                    <select
                        id="theme"
                        bind:value={config.theme}
                        onchange={handleChange}
                    >
                        <option value="dark">Dark</option>
                        <option value="light">Light</option>
                        <option value="auto">Auto (System)</option>
                    </select>
                </div>

                <div class="setting-group">
                    <label for="language">
                        <span class="label-text">Language</span>
                        <span class="label-description">Interface language</span
                        >
                    </label>
                    <select
                        id="language"
                        bind:value={config.language}
                        onchange={handleChange}
                    >
                        <option value="en">English</option>
                        <option value="es">Español</option>
                        <option value="fr">Français</option>
                        <option value="de">Deutsch</option>
                        <option value="zh">中文</option>
                    </select>
                </div>
            </section>

            <!-- Model Parameters Section -->
            <section class="settings-section">
                <h2>Model Parameters</h2>
                <div class="setting-group">
                    <label for="max-tokens">
                        <span class="label-text">Max Tokens</span>
                        <span class="label-description"
                            >Maximum number of tokens to generate (128-8192)</span
                        >
                    </label>
                    <div class="range-input">
                        <input
                            id="max-tokens"
                            type="range"
                            min="128"
                            max="8192"
                            step="128"
                            bind:value={config.max_tokens}
                            oninput={handleChange}
                        />
                        <span class="range-value">{config.max_tokens}</span>
                    </div>
                </div>

                <div class="setting-group">
                    <label for="temperature">
                        <span class="label-text">Temperature</span>
                        <span class="label-description"
                            >Controls randomness (0.0-2.0). Lower is more
                            focused, higher is more creative</span
                        >
                    </label>
                    <div class="range-input">
                        <input
                            id="temperature"
                            type="range"
                            min="0"
                            max="2"
                            step="0.1"
                            bind:value={config.temperature}
                            oninput={handleChange}
                        />
                        <span class="range-value"
                            >{config.temperature.toFixed(1)}</span
                        >
                    </div>
                </div>
            </section>

            <!-- Chat Settings Section -->
            <section class="settings-section">
                <h2>Chat Settings</h2>
                <div class="setting-group">
                    <label class="checkbox-label">
                        <input
                            type="checkbox"
                            bind:checked={config.auto_save_chat}
                            onchange={handleChange}
                        />
                        <div>
                            <span class="label-text"
                                >Auto-save Chat History</span
                            >
                            <span class="label-description"
                                >Automatically save conversations</span
                            >
                        </div>
                    </label>
                </div>

                <div class="setting-group">
                    <label for="history-limit">
                        <span class="label-text">Chat History Limit</span>
                        <span class="label-description"
                            >Number of recent chats to keep (10-100)</span
                        >
                    </label>
                    <div class="range-input">
                        <input
                            id="history-limit"
                            type="range"
                            min="10"
                            max="100"
                            step="5"
                            bind:value={config.chat_history_limit}
                            oninput={handleChange}
                        />
                        <span class="range-value"
                            >{config.chat_history_limit}</span
                        >
                    </div>
                </div>
            </section>

            <!-- About Section -->
            <section class="settings-section">
                <h2>About</h2>
                <div class="about-info">
                    <p><strong>Llama Desktop</strong></p>
                    <p>Version 0.1.0</p>
                    {#if configPath}
                        <p class="config-path">
                            <strong>Config file:</strong><br />
                            <code>{configPath}</code>
                        </p>
                    {/if}
                </div>
            </section>
        </div>
    {/if}
</div>

<style>
    .settings-container {
        max-width: 900px;
        margin: 0 auto;
        padding: 40px 20px;
        color: var(--color-text-primary);
        height: 100vh;
        overflow-y: auto;
    }

    .settings-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 30px;
        padding-bottom: 20px;
        border-bottom: 1px solid var(--color-border);
    }

    .settings-header h1 {
        margin: 0;
        font-size: 32px;
        font-weight: 600;
    }

    .header-actions {
        display: flex;
        gap: 12px;
    }

    .message {
        padding: 12px 16px;
        border-radius: 8px;
        margin-bottom: 20px;
        font-size: 14px;
    }

    .message.success {
        background-color: rgba(34, 197, 94, 0.1);
        border: 1px solid rgba(34, 197, 94, 0.3);
        color: #4ade80;
    }

    .message.error {
        background-color: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.3);
        color: #f87171;
    }

    .loading {
        text-align: center;
        padding: 40px;
        color: var(--color-text-secondary);
    }

    .settings-content {
        display: flex;
        flex-direction: column;
        gap: 32px;
    }

    .settings-section {
        background-color: var(--color-bg-secondary);
        border: 1px solid var(--color-border);
        border-radius: 12px;
        padding: 24px;
    }

    .settings-section h2 {
        margin: 0 0 20px 0;
        font-size: 20px;
        font-weight: 600;
        color: white;
    }

    .setting-group {
        margin-bottom: 24px;
    }

    .setting-group:last-child {
        margin-bottom: 0;
    }

    label {
        display: block;
        margin-bottom: 8px;
    }

    .label-text {
        display: block;
        font-weight: 500;
        font-size: 14px;
        color: white;
        margin-bottom: 4px;
    }

    .label-description {
        display: block;
        font-size: 13px;
        color: var(--color-text-secondary);
    }

    input[type="text"],
    select {
        width: 100%;
        padding: 10px 12px;
        background-color: var(--color-input-bg);
        border: 1px solid var(--color-border);
        border-radius: 6px;
        color: white;
        font-size: 14px;
        font-family: inherit;
    }

    input[type="text"]:focus,
    select:focus {
        outline: none;
        border-color: var(--color-accent);
    }

    input[type="text"]:read-only {
        cursor: default;
        color: var(--color-text-secondary);
    }

    .input-with-button {
        display: flex;
        gap: 8px;
    }

    .input-with-button input {
        flex: 1;
    }

    .range-input {
        display: flex;
        align-items: center;
        gap: 16px;
    }

    input[type="range"] {
        flex: 1;
        height: 6px;
        background: var(--color-border);
        border-radius: 3px;
        outline: none;
        -webkit-appearance: none;
    }

    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 18px;
        height: 18px;
        background: var(--color-accent);
        cursor: pointer;
        border-radius: 50%;
    }

    input[type="range"]::-moz-range-thumb {
        width: 18px;
        height: 18px;
        background: var(--color-accent);
        cursor: pointer;
        border-radius: 50%;
        border: none;
    }

    .range-value {
        min-width: 50px;
        text-align: right;
        font-weight: 500;
        color: white;
    }

    .checkbox-label {
        display: flex;
        align-items: flex-start;
        gap: 12px;
        cursor: pointer;
    }

    input[type="checkbox"] {
        width: 20px;
        height: 20px;
        margin-top: 2px;
        cursor: pointer;
        accent-color: var(--color-accent);
    }

    .btn-primary,
    .btn-secondary,
    .btn-exit {
        padding: 10px 20px;
        border-radius: 6px;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
        border: none;
        font-family: inherit;
    }

    .btn-primary {
        background-color: var(--color-accent);
        color: white;
    }

    .btn-primary:hover:not(:disabled) {
        background-color: var(--color-accent-hover);
    }

    .btn-primary:disabled {
        background-color: #565869;
        cursor: not-allowed;
        opacity: 0.5;
    }

    .btn-secondary {
        background-color: transparent;
        color: white;
        border: 1px solid var(--color-border);
    }

    .btn-secondary:hover:not(:disabled) {
        background-color: rgba(255, 255, 255, 0.1);
    }

    .btn-secondary:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .btn-exit {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 40px;
        height: 40px;
        padding: 0;
        background-color: transparent;
        border: 1px solid var(--color-border);
        color: white;
        text-decoration: none;
    }

    .btn-exit:hover {
        background-color: rgba(255, 255, 255, 0.1);
        border-color: #ff6b6b;
        color: #ff6b6b;
    }

    .about-info {
        color: var(--color-text-secondary);
        font-size: 14px;
        line-height: 1.6;
    }

    .about-info p {
        margin: 8px 0;
    }

    .config-path {
        margin-top: 16px;
        padding-top: 16px;
        border-top: 1px solid var(--color-border);
    }

    .config-path code {
        display: inline-block;
        margin-top: 4px;
        padding: 8px 12px;
        background-color: var(--color-input-bg);
        border-radius: 4px;
        font-size: 12px;
        word-break: break-all;
    }
</style>
