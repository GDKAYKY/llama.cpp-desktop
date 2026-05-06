<script>
  import { unified } from "unified";
  import remarkParse from "remark-parse";
  import remarkBreaks from "remark-breaks";
  import remarkGfm from "remark-gfm";
  import remarkMath from "remark-math";
  import remarkRehype from "remark-rehype";
  import rehypeKatex from "rehype-katex";
  import rehypeHighlight from "rehype-highlight";
  import rehypeSanitize from "rehype-sanitize";
  import rehypeStringify from "rehype-stringify";
  import { onDestroy, tick } from "svelte";
  import { rehypeRestoreTableHtml } from "$lib/markdown/table-html-restorer.js";
  import { rehypeEnhanceLinks } from "$lib/markdown/enhance-links.js";
  import { rehypeEnhanceCodeBlocks } from "$lib/markdown/enhance-code-blocks.js";
  import { sanitizeSchema } from "$lib/markdown/sanitize-schema.js";
  import { remarkLiteralHtml } from "$lib/markdown/literal-html.js";
  import { preprocessLaTeX } from "$shared/latex-protection.js";
  import { copyCodeToClipboard } from "$shared/clipboard.js";
  import { mode } from "mode-watcher";
  import { cn } from "$shared/cn.js";
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
      .use(rehypeSanitize, sanitizeSchema)
      .use(rehypeStringify);
  };

  function cleanupEventListeners() {
    if (!containerRef) return;
    const copyButtons = containerRef.querySelectorAll(".copy-code-btn");
    for (const button of copyButtons) {
      button.removeEventListener("click", handleCopyClick);
    }
    const favicons = containerRef.querySelectorAll(".mcp-link-favicon");
    for (const icon of favicons) {
      icon.removeEventListener("error", handleFaviconError);
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

  function handleFaviconError(event) {
    const target = event.currentTarget;
    if (!(target instanceof HTMLImageElement)) return;
    const fallback = target.dataset.fallback;
    if (!fallback || target.src === fallback) return;
    target.src = fallback;
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

  function setupFaviconFallbacks() {
    if (!containerRef) return;
    const favicons = containerRef.querySelectorAll(".mcp-link-favicon");
    for (const icon of favicons) {
      if (icon.dataset.listenerBound === "true") continue;
      icon.dataset.listenerBound = "true";
      icon.addEventListener("error", handleFaviconError);
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
      setupFaviconFallbacks();
    }
  });

  onDestroy(() => {
    cleanupEventListeners();
  });
</script>

<div
  bind:this={containerRef}
  class={cn(
    "text-[15px] leading-6 md:text-base md:leading-relaxed break-words",
    "[&_p]:mb-4 [&_p:last-child]:mb-0",
    "[&_ul]:mb-4 [&_ul]:list-disc [&_ul]:pl-6",
    "[&_ol]:mb-4 [&_ol]:list-decimal [&_ol]:pl-6",
    "[&_li]:mb-1",
    "[&_code:not(pre_code)]:rounded [&_code:not(pre_code)]:bg-white/10 [&_code:not(pre_code)]:px-1.5 [&_code:not(pre_code)]:py-0.5 [&_code:not(pre_code)]:font-mono",
    "[&_table]:my-4 [&_table]:w-full [&_table]:border-collapse",
    "[&_th]:border [&_th]:border-border [&_th]:bg-white/5 [&_th]:p-2 [&_th]:text-left",
    "[&_td]:border [&_td]:border-border [&_td]:p-2 [&_td]:text-left",
    "[&_blockquote]:my-4 [&_blockquote]:border-l-4 [&_blockquote]:border-primary [&_blockquote]:pl-4 [&_blockquote]:italic [&_blockquote]:text-muted-foreground",
    "[&_h1]:mb-4 [&_h1]:mt-8 [&_h1]:text-2xl [&_h1]:font-bold",
    "[&_h2]:mb-3 [&_h2]:mt-6 [&_h2]:text-xl [&_h2]:font-bold",
    "[&_h3]:mb-2 [&_h3]:mt-4 [&_h3]:text-lg [&_h3]:font-bold",
    className,
  )}
>
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
