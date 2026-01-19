<script>
  import { cn } from "$lib/utils/cn.js";

  /** @type {{
   *   isSidebarOpen: boolean,
   *   toggleSidebar: () => void,
   *   isLoading: boolean,
   *   toggleDropdown: () => void,
   *   selectedModel: any,
   *   isDropdownOpen: boolean,
   *   models: any[],
   *   selectModel: (model: any) => void,
   *   handleClickOutside: (e: MouseEvent) => void,
   *   modelLoaded: boolean,
   *   llama_cpp_path: string
   * }} */
  let {
    isSidebarOpen,
    toggleSidebar,
    isLoading,
    toggleDropdown,
    selectedModel,
    isDropdownOpen,
    models,
    selectModel,
    handleClickOutside,
    modelLoaded,
    llama_cpp_path,
  } = $props();
</script>

<svelte:window onclick={handleClickOutside} />

<header class="chat-header">
  <div class="left-actions">
    <button
      class="icon-btn"
      onclick={toggleSidebar}
      aria-label="Toggle sidebar"
    >
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
        ><line x1="3" y1="12" x2="21" y2="12"></line><line
          x1="3"
          y1="6"
          x2="21"
          y2="6"
        ></line><line x1="3" y1="18" x2="21" y2="18"></line></svg
      >
    </button>
  </div>

  <div class="header-center">
    <div class="model-dropdown-container">
      <button class="model-selector-btn" onclick={(e) => toggleDropdown(e)}>
        <span class="model-name">
          {selectedModel ? selectedModel.name : "Select a model"}
        </span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class={cn("chevron", isDropdownOpen && "open")}
          ><polyline points="6 9 12 15 18 9"></polyline></svg
        >
      </button>

      {#if isDropdownOpen}
        <div class="model-dropdown">
          {#if models.length === 0}
            <div class="dropdown-item empty">No models found</div>
          {:else}
            {#each models as model}
              <button
                class={cn(
                  "dropdown-item",
                  selectedModel?.full_identifier === model.full_identifier &&
                    "active",
                )}
                onclick={() => selectModel(model)}
              >
                <div class="model-info">
                  <span class="name">{model.name}</span>
                  <span class="details">{model.full_identifier}</span>
                </div>
                {#if selectedModel?.full_identifier === model.full_identifier}
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><polyline points="20 6 9 17 4 12"></polyline></svg
                  >
                {/if}
              </button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <div class="right-actions">
    {#if isLoading}
      <div class="loading-indicator">
        <svg
          class="spinner"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            class="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            stroke-width="4"
          ></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
      </div>
    {/if}
    <button class="icon-btn" aria-label="Settings">
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
        ><circle cx="12" cy="12" r="3"></circle><path
          d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
        ></path></svg
      >
    </button>
  </div>
</header>

<style>
  .chat-header {
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    background: var(--background);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 50;
  }

  .left-actions,
  .right-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .header-center {
    flex-grow: 1;
    display: flex;
    justify-content: center;
    position: relative;
  }

  .model-dropdown-container {
    position: relative;
    max-width: 400px;
    width: 100%;
    display: flex;
    justify-content: center;
  }

  .model-selector-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background-color: var(--secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--foreground);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
    max-width: 100%;
  }

  .model-selector-btn:hover {
    background-color: rgba(255, 255, 255, 0.05);
  }

  .model-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .chevron {
    transition: transform 0.2s;
    flex-shrink: 0;
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .model-dropdown {
    position: absolute;
    top: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    width: 320px;
    background-color: var(--secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3);
    overflow: hidden;
    z-index: 100;
  }

  .dropdown-item {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: transparent;
    border: none;
    color: var(--foreground);
    cursor: pointer;
    text-align: left;
    transition: background 0.2s;
  }

  .dropdown-item:hover {
    background-color: rgba(255, 255, 255, 0.05);
  }

  .dropdown-item.active {
    background-color: rgba(255, 255, 255, 0.1);
  }

  .dropdown-item.empty {
    color: var(--muted-foreground);
    justify-content: center;
    font-style: italic;
    pointer-events: none;
  }

  .model-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .model-info .name {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .model-info .details {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--muted-foreground);
    cursor: pointer;
    padding: 8px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      background 0.2s,
      color 0.2s;
  }

  .icon-btn:hover {
    background-color: var(--secondary);
    color: var(--foreground);
  }

  .loading-indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--muted-foreground);
  }

  .spinner {
    animation: spin 1s linear infinite;
    width: 16px;
    height: 16px;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .opacity-25 {
    opacity: 0.25;
  }
  .opacity-75 {
    opacity: 0.75;
  }
</style>
