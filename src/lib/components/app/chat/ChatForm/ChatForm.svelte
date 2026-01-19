<script>
  import { modelsStore } from "$lib/stores/models.svelte";

  /** @type {{
   *   userInput: string,
   *   modelLoaded: boolean,
   *   isLoading: boolean,
   *   onKeydown: (e: KeyboardEvent) => void,
   *   onInput: () => void,
   *   onSend: () => void,
   *   textarea: HTMLTextAreaElement
   * }} */
  let {
    userInput = $bindable(),
    modelLoaded,
    isLoading,
    onKeydown,
    onInput,
    onSend,
    textarea = $bindable(),
  } = $props();

  let modelHash = $derived(
    modelsStore.selectedModel
      ? modelsStore.selectedModel.full_identifier
      : "No model selected",
  );
</script>

<div class="chat-form-container">
  <form
    onsubmit={(e) => {
      e.preventDefault();
      onSend();
    }}
    class="chat-form"
  >
    <div class="form-inner">
      <button type="button" class="action-btn" aria-label="Attach files">
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
          ><path
            d="m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.51a2 2 0 0 1-2.83-2.83l8.49-8.48"
          /></svg
        >
      </button>

      <div class="textarea-wrapper">
        <textarea
          placeholder="Ask anything..."
          bind:value={userInput}
          bind:this={textarea}
          onkeydown={onKeydown}
          oninput={onInput}
          rows="1"
        ></textarea>
      </div>

      <div class="bottom-row">
        <div class="model-pill">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><path
              d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
            ></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"
            ></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg
          >
          <span class="hash">{modelHash}</span>
        </div>

        <button
          type="submit"
          class="send-submit-btn"
          disabled={!userInput.trim() || !modelLoaded || isLoading}
          aria-label="Send"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><path d="M5 12h14" /><path d="m12 5 7 7-7 7" /></svg
          >
        </button>
      </div>
    </div>
  </form>
  <div class="keyboard-hint">
    Press <span class="key">Enter</span> to send,
    <span class="key">Shift + Enter</span> for new line
  </div>
</div>

<style>
  .chat-form-container {
    width: 100%;
    max-width: 48rem;
    margin: 0 auto;
    padding: 0 1rem 1.5rem;
  }

  .chat-form {
    background-color: var(--secondary);
    border: 1px solid var(--border);
    border-radius: 1.5rem;
    padding: 1rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: box-shadow 0.2s;
  }

  .chat-form:focus-within {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .form-inner {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    position: relative;
  }

  .action-btn {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    background: transparent;
    border: none;
    color: var(--muted-foreground);
    cursor: pointer;
    padding: 8px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      background 0.2s,
      color 0.2s;
    z-index: 5;
  }

  .action-btn:hover {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--foreground);
  }

  .textarea-wrapper {
    padding-left: 2.5rem; /* Space for attach button */
    padding-right: 0.5rem;
  }

  textarea {
    width: 100%;
    background: transparent;
    border: none;
    color: var(--foreground);
    font-family: inherit;
    font-size: 1rem;
    line-height: 1.5;
    outline: none;
    resize: none;
    min-height: 24px;
    max-height: 200px;
    padding: 0;
  }

  textarea::placeholder {
    color: var(--muted-foreground);
    opacity: 0.6;
  }

  .bottom-row {
    display: flex;
    justify-content: center;
    align-items: center;
    margin-top: 0.5rem;
    position: relative;
  }

  .model-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border);
    border-radius: 9999px;
    padding: 2px 10px;
    font-size: 0.75rem;
    color: var(--muted-foreground);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
      monospace;
    max-width: 300px;
  }

  .model-pill .hash {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .send-submit-btn {
    position: absolute;
    right: 0;
    bottom: -4px;
    background-color: var(--muted-foreground);
    color: var(--background);
    border: none;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      transform 0.2s,
      background-color 0.2s;
  }

  .send-submit-btn:hover:not(:disabled) {
    background-color: var(--foreground);
    transform: scale(1.05);
  }

  .send-submit-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .keyboard-hint {
    text-align: center;
    font-size: 0.75rem;
    color: var(--muted-foreground);
    margin-top: 0.75rem;
    opacity: 0.8;
  }

  .key {
    background-color: var(--secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 4px;
    font-family: inherit;
  }
</style>
