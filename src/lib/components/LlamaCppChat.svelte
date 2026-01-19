<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let messages = [];
    let inputMessage = "";
    let isLoading = false;
    let serverRunning = false;
    let showSettings = false;

    // Configuration
    let config = {
        llama_cpp_path: "E:\\src\\llama_cpp",
        model_path: "C:\\Models\\mistral-7b.gguf",
        port: 8080,
        ctx_size: 4096,
        parallel: 4,
        n_gpu_layers: 33,
    };

    let chatParams = {
        temperature: 0.7,
        top_p: 0.95,
        top_k: 40,
        max_tokens: 512,
    };

    onMount(async () => {
        await checkServerStatus();
    });

    async function checkServerStatus() {
        try {
            serverRunning = await invoke("is_llama_running");
            if (serverRunning) {
                const currentConfig = await invoke("get_llama_config");
                if (currentConfig) {
                    config = currentConfig;
                }
            }
        } catch (error) {
            console.error("Error checking server status:", error);
        }
    }

    async function startServer() {
        try {
            isLoading = true;
            await invoke("start_llama_server", config);
            serverRunning = true;
            messages.push({
                role: "system",
                content: "Server started successfully",
            });
        } catch (error) {
            messages.push({
                role: "system",
                content: `Error starting server: ${error}`,
            });
        } finally {
            isLoading = false;
        }
    }

    async function stopServer() {
        try {
            isLoading = true;
            await invoke("stop_llama_server");
            serverRunning = false;
            messages.push({
                role: "system",
                content: "Server stopped",
            });
        } catch (error) {
            messages.push({
                role: "system",
                content: `Error stopping server: ${error}`,
            });
        } finally {
            isLoading = false;
        }
    }

    async function sendMessage() {
        if (!inputMessage.trim() || !serverRunning) return;

        const userMessage = inputMessage;
        inputMessage = "";

        // Add user message to chat
        messages.push({
            role: "user",
            content: userMessage,
        });
        messages = messages;

        try {
            isLoading = true;

            // Prepare message history for context
            const messageHistory = messages
                .filter((m) => m.role !== "system")
                .map((m) => [m.role, m.content]);

            // Send to server
            const response = await invoke("send_chat_with_history", {
                messages: messageHistory,
                ...chatParams,
            });

            // Add assistant response
            messages.push({
                role: "assistant",
                content: response,
            });
            messages = messages;
        } catch (error) {
            messages.push({
                role: "system",
                content: `Error: ${error}`,
            });
            messages = messages;
        } finally {
            isLoading = false;
        }
    }

    function clearChat() {
        messages = messages.filter((m) => m.role === "system");
    }
</script>

<div class="chat-container">
    <div class="header">
        <h1>Llama.cpp Chat</h1>
        <div class="status">
            <span class="status-indicator" class:running={serverRunning} />
            <span>{serverRunning ? "Server Running" : "Server Offline"}</span>
        </div>
    </div>

    {#if !serverRunning}
        <div class="setup-panel">
            <h2>Server Configuration</h2>

            <div class="form-group">
                <label>llama.cpp Path</label>
                <input
                    type="text"
                    bind:value={config.llama_cpp_path}
                    placeholder="E:\src\llama_cpp"
                />
            </div>

            <div class="form-group">
                <label>Model Path</label>
                <input
                    type="text"
                    bind:value={config.model_path}
                    placeholder="C:\Models\mistral-7b.gguf"
                />
            </div>

            <div class="form-row">
                <div class="form-group">
                    <label>Port</label>
                    <input type="number" bind:value={config.port} />
                </div>
                <div class="form-group">
                    <label>Context Size</label>
                    <input type="number" bind:value={config.ctx_size} />
                </div>
            </div>

            <div class="form-row">
                <div class="form-group">
                    <label>Parallel Slots</label>
                    <input type="number" bind:value={config.parallel} />
                </div>
                <div class="form-group">
                    <label>GPU Layers</label>
                    <input type="number" bind:value={config.n_gpu_layers} />
                </div>
            </div>

            <button
                on:click={startServer}
                disabled={isLoading}
                class="btn-primary"
            >
                {isLoading ? "Starting..." : "Start Server"}
            </button>
        </div>
    {:else}
        <div class="chat-panel">
            <div class="messages">
                {#each messages as message (message)}
                    <div class="message" class:user={message.role === "user"}>
                        <div class="message-role">{message.role}</div>
                        <div class="message-content">{message.content}</div>
                    </div>
                {/each}
                {#if isLoading}
                    <div class="message assistant">
                        <div class="message-role">assistant</div>
                        <div class="message-content">
                            <span class="typing">Thinking...</span>
                        </div>
                    </div>
                {/if}
            </div>

            <div class="input-area">
                <textarea
                    bind:value={inputMessage}
                    placeholder="Type your message..."
                    disabled={isLoading}
                    on:keydown={(e) => {
                        if (e.key === "Enter" && !e.shiftKey) {
                            e.preventDefault();
                            sendMessage();
                        }
                    }}
                />
                <button
                    on:click={sendMessage}
                    disabled={isLoading || !inputMessage.trim()}
                >
                    Send
                </button>
            </div>

            <div class="controls">
                <button
                    on:click={() => (showSettings = !showSettings)}
                    class="btn-secondary"
                >
                    {showSettings ? "Hide" : "Show"} Settings
                </button>
                <button on:click={clearChat} class="btn-secondary"
                    >Clear Chat</button
                >
                <button
                    on:click={stopServer}
                    disabled={isLoading}
                    class="btn-danger"
                >
                    Stop Server
                </button>
            </div>

            {#if showSettings}
                <div class="settings-panel">
                    <h3>Chat Parameters</h3>
                    <div class="form-row">
                        <div class="form-group">
                            <label
                                >Temperature ({chatParams.temperature.toFixed(
                                    2,
                                )})</label
                            >
                            <input
                                type="range"
                                min="0"
                                max="2"
                                step="0.1"
                                bind:value={chatParams.temperature}
                            />
                        </div>
                        <div class="form-group">
                            <label>Top P ({chatParams.top_p.toFixed(2)})</label>
                            <input
                                type="range"
                                min="0"
                                max="1"
                                step="0.05"
                                bind:value={chatParams.top_p}
                            />
                        </div>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label>Top K</label>
                            <input
                                type="number"
                                bind:value={chatParams.top_k}
                            />
                        </div>
                        <div class="form-group">
                            <label>Max Tokens</label>
                            <input
                                type="number"
                                bind:value={chatParams.max_tokens}
                            />
                        </div>
                    </div>
                </div>
            {/if}
        </div>
    {/if}
</div>

<style>
    .chat-container {
        display: flex;
        flex-direction: column;
        height: 100vh;
        background: #f5f5f5;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        background: #2c3e50;
        color: white;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }

    .header h1 {
        margin: 0;
        font-size: 1.5rem;
    }

    .status {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .status-indicator {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: #e74c3c;
        transition: background 0.3s;
    }

    .status-indicator.running {
        background: #27ae60;
    }

    .setup-panel,
    .chat-panel {
        flex: 1;
        display: flex;
        flex-direction: column;
        padding: 1rem;
        overflow: hidden;
    }

    .setup-panel {
        justify-content: center;
        align-items: center;
        max-width: 500px;
        margin: auto;
    }

    .setup-panel h2 {
        margin-top: 0;
    }

    .form-group {
        margin-bottom: 1rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
        color: #2c3e50;
    }

    .form-group input {
        width: 100%;
        padding: 0.5rem;
        border: 1px solid #bdc3c7;
        border-radius: 4px;
        font-size: 1rem;
    }

    .form-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .messages {
        flex: 1;
        overflow-y: auto;
        margin-bottom: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .message {
        padding: 0.75rem;
        border-radius: 8px;
        background: white;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .message.user {
        background: #3498db;
        color: white;
        margin-left: 2rem;
    }

    .message.assistant {
        background: white;
        margin-right: 2rem;
    }

    .message-role {
        font-size: 0.75rem;
        font-weight: 600;
        opacity: 0.7;
        margin-bottom: 0.25rem;
        text-transform: uppercase;
    }

    .message-content {
        line-height: 1.5;
        word-wrap: break-word;
    }

    .typing {
        animation: blink 1.4s infinite;
    }

    @keyframes blink {
        0%,
        20%,
        50%,
        80%,
        100% {
            opacity: 1;
        }
        40% {
            opacity: 0.5;
        }
        60% {
            opacity: 0.7;
        }
    }

    .input-area {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1rem;
    }

    .input-area textarea {
        flex: 1;
        padding: 0.75rem;
        border: 1px solid #bdc3c7;
        border-radius: 4px;
        font-family: inherit;
        font-size: 1rem;
        resize: none;
        max-height: 100px;
    }

    .input-area button {
        padding: 0.75rem 1.5rem;
        background: #3498db;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 500;
        transition: background 0.3s;
    }

    .input-area button:hover:not(:disabled) {
        background: #2980b9;
    }

    .input-area button:disabled {
        background: #95a5a6;
        cursor: not-allowed;
    }

    .controls {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1rem;
    }

    .btn-primary,
    .btn-secondary,
    .btn-danger {
        padding: 0.5rem 1rem;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 500;
        transition: background 0.3s;
    }

    .btn-primary {
        background: #27ae60;
        color: white;
    }

    .btn-primary:hover:not(:disabled) {
        background: #229954;
    }

    .btn-secondary {
        background: #95a5a6;
        color: white;
    }

    .btn-secondary:hover:not(:disabled) {
        background: #7f8c8d;
    }

    .btn-danger {
        background: #e74c3c;
        color: white;
    }

    .btn-danger:hover:not(:disabled) {
        background: #c0392b;
    }

    .btn-primary:disabled,
    .btn-secondary:disabled,
    .btn-danger:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .settings-panel {
        background: white;
        padding: 1rem;
        border-radius: 8px;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    }

    .settings-panel h3 {
        margin-top: 0;
        color: #2c3e50;
    }
</style>
