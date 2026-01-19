<script>
  import { unified } from "unified";
  import remarkParse from "remark-parse";
  import remarkBreaks from "remark-breaks";
  import remarkGfm from "remark-gfm";
  import remarkMath from "remark-math";
  import remarkRehype from "remark-rehype";
  import rehypeKatex from "rehype-katex";
  import rehypeHighlight from "rehype-highlight";
  import rehypeStringify from "rehype-stringify";
  import { onDestroy, tick } from "svelte";
  import { rehypeRestoreTableHtml } from "$lib/markdown/table-html-restorer.js";
  import { rehypeEnhanceLinks } from "$lib/markdown/enhance-links.js";
  import { rehypeEnhanceCodeBlocks } from "$lib/markdown/enhance-code-blocks.js";
  import { remarkLiteralHtml } from "$lib/markdown/literal-html.js";
  import { preprocessLaTeX } from "$lib/utils/latex-protection.js";
  import { copyCodeToClipboard } from "$lib/utils/clipboard.js";
  import { mode } from "mode-watcher";
  import "highlight.js/styles/github-dark.css";
  import "katex/dist/katex.min.css";

  /** @type {{ content: string, class?: string }} */
  let { content, class: className = "" } = $props();

  let containerRef = $state();
  let renderedBlocks = $state([]);
  let unstableBlockHtml = $state("");

  let pendingMarkdown = null;
  let isProcessing = false;

  const processor = () => {
    return unified()
      .use(remarkParse)
      .use(remarkGfm)
      .use(remarkMath)
      .use(remarkBreaks)
      .use(remarkLiteralHtml)
      .use(remarkRehype)
      .use(rehypeKatex)
      .use(rehypeHighlight)
      .use(rehypeRestoreTableHtml)
      .use(rehypeEnhanceLinks)
      .use(rehypeEnhanceCodeBlocks)
      .use(rehypeStringify, { allowDangerousHtml: true });
  };

  function cleanupEventListeners() {
    if (!containerRef) return;
    const copyButtons = containerRef.querySelectorAll(".copy-code-btn");
    for (const button of copyButtons) {
      button.removeEventListener("click", handleCopyClick);
    }
  }

  async function handleCopyClick(event) {
    event.preventDefault();
    event.stopPropagation();
    const target = event.currentTarget;
    if (!target) return;

    const wrapper = target.closest(".code-block-wrapper");
    if (!wrapper) return;

    const codeElement = wrapper.querySelector("code[data-code-id]");
    if (!codeElement) return;

    const rawCode = codeElement.textContent ?? "";
    await copyCodeToClipboard(rawCode);
  }

  function getHastNodeId(node, indexFallback) {
    const position = node.position;
    if (position?.start?.offset != null && position?.end?.offset != null) {
      return `hast-${position.start.offset}-${position.end.offset}`;
    }
    return `${node.type}-${indexFallback}`;
  }

  async function processMarkdown(markdown) {
    if (!markdown) {
      renderedBlocks = [];
      unstableBlockHtml = "";
      return;
    }

    const normalized = preprocessLaTeX(markdown);
    const processorInstance = processor();
    const ast = processorInstance.parse(normalized);
    const processedRoot = await processorInstance.run(ast);
    const processedChildren = processedRoot.children ?? [];
    const stableCount = Math.max(processedChildren.length - 1, 0);
    const nextBlocks = [];

    for (let index = 0; index < stableCount; index++) {
      const hastChild = processedChildren[index];
      const id = getHastNodeId(hastChild, index);
      const existing = renderedBlocks[index];

      if (existing && existing.id === id) {
        nextBlocks.push(existing);
        continue;
      }

      const html = processorInstance.stringify({
        type: "root",
        children: [hastChild],
      });

      nextBlocks.push({ id, html });
    }

    let unstableHtml = "";
    if (processedChildren.length > stableCount) {
      const unstableChild = processedChildren[stableCount];
      unstableHtml = processorInstance.stringify({
        type: "root",
        children: [unstableChild],
      });
    }

    renderedBlocks = nextBlocks;
    await tick();
    unstableBlockHtml = unstableHtml;
  }

  function setupCodeBlockActions() {
    if (!containerRef) return;
    const wrappers = containerRef.querySelectorAll(".code-block-wrapper");
    for (const wrapper of wrappers) {
      const copyButton = wrapper.querySelector(".copy-code-btn");
      if (copyButton && copyButton.dataset.listenerBound !== "true") {
        copyButton.dataset.listenerBound = "true";
        copyButton.addEventListener("click", handleCopyClick);
      }
    }
  }

  async function updateRenderedBlocks(markdown) {
    pendingMarkdown = markdown;
    if (isProcessing) return;
    isProcessing = true;

    try {
      while (pendingMarkdown !== null) {
        const nextMarkdown = pendingMarkdown;
        pendingMarkdown = null;
        await processMarkdown(nextMarkdown);
      }
    } catch (error) {
      console.error("Failed to process markdown:", error);
      renderedBlocks = [];
      unstableBlockHtml = markdown.replace(/\n/g, "<br>");
    } finally {
      isProcessing = false;
    }
  }

  $effect(() => {
    updateRenderedBlocks(content);
  });

  $effect(() => {
    if ((renderedBlocks.length > 0 || unstableBlockHtml) && containerRef) {
      setupCodeBlockActions();
    }
  });

  onDestroy(() => {
    cleanupEventListeners();
  });
</script>

<div bind:this={containerRef} class="markdown-content {className}">
  {#each renderedBlocks as block (block.id)}
    <div class="markdown-block" data-block-id={block.id}>
      {@html block.html}
    </div>
  {/each}

  {#if unstableBlockHtml}
    <div
      class="markdown-block markdown-block--unstable"
      data-block-id="unstable"
    >
      {@html unstableBlockHtml}
    </div>
  {/if}
</div>

<style>
  .markdown-content {
    line-height: 1.6;
    font-size: 16px;
  }

  .markdown-block {
    display: contents;
  }

  .markdown-content :global(p) {
    margin-bottom: 1rem;
  }

  .markdown-content :global(p:last-child) {
    margin-bottom: 0;
  }

  .markdown-content :global(pre) {
    background: #1e1e1e;
    color: #d4d4d4;
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
    margin: 1rem 0;
  }

  .markdown-content :global(code:not(pre code)) {
    background: rgba(255, 255, 255, 0.1);
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-family: monospace;
  }

  .markdown-content :global(.code-block-wrapper) {
    margin: 1.5rem 0;
    border-radius: 0.75rem;
    overflow: hidden;
    border: 1px solid var(--color-border);
    background: #1e1e1e;
  }

  .markdown-content :global(.code-block-header) {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.875rem;
  }

  .markdown-content :global(.code-language) {
    color: #888;
    font-weight: 500;
    text-transform: uppercase;
    font-size: 0.75rem;
  }

  .markdown-content :global(.copy-code-btn) {
    background: transparent;
    border: none;
    color: #888;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    transition: all 0.2s;
  }

  .markdown-content :global(.copy-code-btn:hover) {
    color: white;
    background: rgba(255, 255, 255, 0.1);
  }

  .markdown-content :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
  }

  .markdown-content :global(th),
  .markdown-content :global(td) {
    border: 1px solid var(--color-border);
    padding: 0.5rem;
    text-align: left;
  }

  .markdown-content :global(th) {
    background: rgba(255, 255, 255, 0.05);
  }

  .markdown-content :global(ul),
  .markdown-content :global(ol) {
    margin-bottom: 1rem;
    padding-left: 1.5rem;
  }

  .markdown-content :global(blockquote) {
    border-left: 4px solid var(--color-accent);
    padding-left: 1rem;
    color: var(--color-text-secondary);
    margin: 1rem 0;
    font-style: italic;
  }
</style>
