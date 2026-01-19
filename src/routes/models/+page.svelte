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
        </div>
    {/if}

    <ModelSelector on:modelSelected={handleModelSelected} />
</div>

<style>
    .container {
        width: 100%;
        min-height: 100vh;
        background: #fafafa;
    }

    .current-model-banner {
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        color: white;
        padding: 16px 20px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .banner-content {
        max-width: 1200px;
        margin: 0 auto;
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
        margin-left: auto;
        background: rgba(255, 255, 255, 0.2);
        padding: 4px 12px;
        border-radius: 12px;
        font-size: 13px;
        font-weight: 600;
    }
</style>
