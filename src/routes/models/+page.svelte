<script>
    import ModelSelector from "$lib/components/ModelSelector.svelte";

    let currentModel = $state(null);

    function handleModelSelected(event) {
        currentModel = event.detail.model;
        console.log("Model selected:", currentModel);
    }
</script>

<svelte:head>
    <title>Model Management - Llama Desktop</title>
</svelte:head>

<div class="container">
    {#if currentModel}
        <div class="current-model-banner">
            <div class="banner-content">
                <span class="banner-label">Current Model:</span>
                <span class="banner-model"
                    >{currentModel.name}:{currentModel.version}</span
                >
                <span class="banner-status">✓ Loaded</span>
            </div>
            <a href="/" class="btn-exit" title="Exit Models">
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
    {:else}
        <div class="header-bar">
            <h2>Model Selection</h2>
            <a href="/" class="btn-exit" title="Exit Models">
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
    {/if}

    <div class="content">
        <ModelSelector on:modelSelected={handleModelSelected} />
    </div>
</div>

<style>
    .container {
        width: 100%;
        height: 100vh;
        background: #fafafa;
        display: flex;
        flex-direction: column;
    }

    .current-model-banner,
    .header-bar {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: white;
        padding: 16px 20px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        flex-shrink: 0;
    }

    .header-bar {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .header-bar h2 {
        margin: 0;
        font-size: 24px;
        font-weight: 600;
    }

    .current-model-banner {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .banner-content {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .banner-label {
        font-weight: 600;
        font-size: 14px;
    }

    .banner-model {
        font-family: monospace;
        font-size: 16px;
        font-weight: 700;
    }

    .banner-status {
        background: rgba(255, 255, 255, 0.2);
        padding: 4px 12px;
        border-radius: 12px;
        font-size: 13px;
        font-weight: 600;
    }

    .btn-exit {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 40px;
        height: 40px;
        background-color: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.3);
        border-radius: 6px;
        color: white;
        text-decoration: none;
        cursor: pointer;
        transition: all 0.2s;
        flex-shrink: 0;
    }

    .btn-exit:hover {
        background-color: rgba(255, 255, 255, 0.2);
        border-color: #ff6b6b;
        color: #ff6b6b;
    }

    .content {
        flex: 1;
        overflow-y: auto;
    }
</style>
