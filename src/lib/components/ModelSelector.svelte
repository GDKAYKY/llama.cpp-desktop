<script>
    import {
        selectModelsDirectory,
        scanModelsDirectory,
        saveModelLibrary,
        loadModelLibrary,
    } from "../models.js";
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    let modelsRoot = $state("");
    let models = $state([]);
    let selectedModel = $state(null);
    let loading = $state(false);
    let error = $state("");
    let libraryPath = $state("");
    let successMessage = $state("");

    async function handleSelectDirectory() {
        try {
            error = "";
            successMessage = "";
            const selected = await selectModelsDirectory();
            if (selected) {
                modelsRoot = selected;
                libraryPath = `${selected}/modelLibrary.json`;
                await loadExistingLibrary();
            }
        } catch (err) {
            error = `Failed to select directory: ${err.message}`;
            console.error(err);
        }
    }

    async function loadExistingLibrary() {
        try {
            loading = true;
            const existingModels = await loadModelLibrary(libraryPath);
            if (existingModels.length > 0) {
                models = existingModels;
                successMessage = `Loaded ${existingModels.length} model(s) from library`;
            }
        } catch (err) {
            console.log("No existing library found, will scan directory");
        } finally {
            loading = false;
        }
    }

    async function handleScanDirectory() {
        if (!modelsRoot) {
            error = "Please select a models directory first";
            return;
        }

        try {
            loading = true;
            error = "";
            successMessage = "";
            models = await scanModelsDirectory(modelsRoot);

            if (models.length > 0) {
                await saveModelLibrary(libraryPath, models);
                successMessage = `Found and saved ${models.length} model(s)`;
            } else {
                error = "No models found in the selected directory";
            }
        } catch (err) {
            error = `Failed to scan directory: ${err.message}`;
            console.error(err);
        } finally {
            loading = false;
        }
    }

    function handleSelectModel(model) {
        selectedModel = model;
        successMessage = "";
    }

    function handleLoadModel() {
        if (!selectedModel) {
            error = "Please select a model first";
            return;
        }

        // Dispatch event to parent component
        dispatch("modelSelected", {
            model: selectedModel,
        });

        successMessage = `Model "${selectedModel.name}:${selectedModel.version}" is ready to use`;
    }

    function formatSize(bytes) {
        const gb = bytes / 1024 ** 3;
        return `${gb.toFixed(2)} GB`;
    }
</script>

<div class="model-selector">
    <div class="header">
        <h2>Model Selection</h2>
    </div>

    <div class="directory-selection">
        <button onclick={handleSelectDirectory} disabled={loading}>
            Select Models Directory
        </button>
        {#if modelsRoot}
            <p class="selected-path">Selected: {modelsRoot}</p>
        {/if}
    </div>

    {#if modelsRoot}
        <div class="actions">
            <button onclick={handleScanDirectory} disabled={loading}>
                {loading ? "Scanning..." : "Scan for Models"}
            </button>
        </div>
    {/if}

    {#if error}
        <div class="error">{error}</div>
    {/if}

    {#if successMessage}
        <div class="success">{successMessage}</div>
    {/if}

    {#if models.length > 0}
        <div class="models-list">
            <div class="list-header">
                <h3>Available Models ({models.length})</h3>
                {#if selectedModel}
                    <button class="btn-load" onclick={handleLoadModel}>
                        Load Selected Model
                    </button>
                {/if}
            </div>
            <div class="models-grid">
                {#each models as model}
                    <div
                        class="model-card"
                        class:selected={selectedModel?.full_identifier ===
                            model.full_identifier}
                        onclick={() => handleSelectModel(model)}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) =>
                            e.key === "Enter" && handleSelectModel(model)}
                    >
                        <div class="model-header">
                            <h4>{model.name}</h4>
                            <span class="version">{model.version}</span>
                        </div>
                        <div class="model-details">
                            <p><strong>Provider:</strong> {model.provider}</p>
                            <p><strong>Library:</strong> {model.library}</p>
                            <p>
                                <strong>Identifier:</strong>
                                {model.full_identifier}
                            </p>
                            {#if model.manifest.layers[0]}
                                <p>
                                    <strong>Size:</strong>
                                    {formatSize(model.manifest.layers[0].size)}
                                </p>
                            {/if}
                            {#if model.model_file_path}
                                <p class="file-path">
                                    <strong>File:</strong>
                                    {model.model_file_path}
                                </p>
                            {:else}
                                <p class="warning">⚠️ Model file not found</p>
                            {/if}
                        </div>
                        {#if selectedModel?.full_identifier === model.full_identifier}
                            <div class="selected-badge">✓ Selected</div>
                        {/if}
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    {#if selectedModel}
        <div class="selected-model">
            <div class="selected-header">
                <h3>Selected Model Details</h3>
                <button class="btn-primary" onclick={handleLoadModel}>
                    Load This Model
                </button>
            </div>
            <div class="model-info">
                <div class="info-row">
                    <span class="info-label">Name:</span>
                    <span class="info-value">{selectedModel.name}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">Version:</span>
                    <span class="info-value">{selectedModel.version}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">Provider:</span>
                    <span class="info-value">{selectedModel.provider}</span>
                </div>
                <div class="info-row">
                    <span class="info-label">Identifier:</span>
                    <span class="info-value"
                        >{selectedModel.full_identifier}</span
                    >
                </div>
                {#if selectedModel.model_file_path}
                    <div class="info-row">
                        <span class="info-label">File Path:</span>
                        <span class="info-value file-path"
                            >{selectedModel.model_file_path}</span
                        >
                    </div>
                {/if}
            </div>
        </div>
    {/if}
</div>

<style>
    .model-selector {
        padding: 20px;
        max-width: 1200px;
        margin: 0 auto;
    }

    .header h2 {
        margin: 0 0 20px 0;
        color: var(--color-text-primary, #333);
    }

    .directory-selection {
        margin-bottom: 20px;
    }

    .selected-path {
        margin-top: 10px;
        padding: 10px;
        background: var(--color-bg-secondary, #f5f5f5);
        border-radius: 4px;
        font-family: monospace;
        font-size: 14px;
        color: var(--color-text-primary, #333);
    }

    .actions {
        margin-bottom: 20px;
    }

    button {
        padding: 10px 20px;
        background: #007bff;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
        font-family: inherit;
    }

    button:hover:not(:disabled) {
        background: #0056b3;
    }

    button:disabled {
        background: #ccc;
        cursor: not-allowed;
    }

    .error {
        padding: 10px;
        background: #fee;
        border: 1px solid #fcc;
        border-radius: 4px;
        color: #c00;
        margin-bottom: 20px;
    }

    .success {
        padding: 10px;
        background: #efe;
        border: 1px solid #cfc;
        border-radius: 4px;
        color: #060;
        margin-bottom: 20px;
    }

    .list-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 15px;
    }

    .list-header h3 {
        margin: 0;
        color: var(--color-text-primary, #333);
    }

    .btn-load {
        background: #28a745;
    }

    .btn-load:hover:not(:disabled) {
        background: #218838;
    }

    .models-list h3 {
        margin-bottom: 15px;
    }

    .models-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
        gap: 15px;
    }

    .model-card {
        border: 2px solid #ddd;
        border-radius: 8px;
        padding: 15px;
        cursor: pointer;
        transition: all 0.2s;
        background: var(--color-bg-primary, white);
        position: relative;
    }

    .model-card:hover {
        border-color: #007bff;
        box-shadow: 0 2px 8px rgba(0, 123, 255, 0.2);
    }

    .model-card.selected {
        border-color: #007bff;
        background: #f0f8ff;
    }

    .model-card:focus {
        outline: 2px solid #007bff;
        outline-offset: 2px;
    }

    .selected-badge {
        position: absolute;
        top: 10px;
        right: 10px;
        background: #28a745;
        color: white;
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 12px;
        font-weight: bold;
    }

    .model-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 10px;
        padding-bottom: 10px;
        border-bottom: 1px solid #eee;
    }

    .model-header h4 {
        margin: 0;
        font-size: 18px;
        color: var(--color-text-primary, #333);
    }

    .version {
        background: #007bff;
        color: white;
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 12px;
    }

    .model-details p {
        margin: 5px 0;
        font-size: 14px;
        color: var(--color-text-primary, #333);
    }

    .file-path {
        font-family: monospace;
        font-size: 12px;
        word-break: break-all;
    }

    .warning {
        color: #ff6600;
        font-weight: bold;
    }

    .selected-model {
        margin-top: 30px;
        padding: 20px;
        background: var(--color-bg-secondary, #f9f9f9);
        border: 1px solid #ddd;
        border-radius: 8px;
    }

    .selected-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 20px;
    }

    .selected-header h3 {
        margin: 0;
        color: var(--color-text-primary, #333);
    }

    .btn-primary {
        background: #28a745;
        padding: 12px 24px;
        font-weight: 600;
    }

    .btn-primary:hover:not(:disabled) {
        background: #218838;
    }

    .model-info {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .info-row {
        display: flex;
        gap: 12px;
        padding: 8px 0;
        border-bottom: 1px solid #eee;
    }

    .info-row:last-child {
        border-bottom: none;
    }

    .info-label {
        font-weight: 600;
        min-width: 120px;
        color: var(--color-text-secondary, #666);
    }

    .info-value {
        flex: 1;
        color: var(--color-text-primary, #333);
    }
</style>
