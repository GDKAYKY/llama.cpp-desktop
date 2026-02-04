import { describe, expect, test } from "vitest";
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
import { rehypeRestoreTableHtml } from "../table-html-restorer.js";
import { rehypeEnhanceLinks } from "../enhance-links.js";
import { rehypeEnhanceCodeBlocks } from "../enhance-code-blocks.js";
import { remarkLiteralHtml } from "../literal-html.js";
import { sanitizeSchema } from "../sanitize-schema.js";
import { preprocessLaTeX } from "../../shared/latex-protection.js";

async function renderMarkdown(markdown) {
  const processor = unified()
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

  const normalized = preprocessLaTeX(markdown);
  const file = await processor.process(normalized);
  return String(file);
}

describe("markdown sanitization", () => {
  test("blocks javascript: links", async () => {
    const html = await renderMarkdown("[x](javascript:alert(1))");
    expect(html).not.toContain("javascript:");
    expect(html).not.toContain("<a");
  });

  test("blocks data: links", async () => {
    const html = await renderMarkdown("[x](data:text/html;base64,aaaa)");
    expect(html).not.toContain("data:text/html");
    expect(html).not.toContain("<a");
  });

  test("keeps https links with target/rel", async () => {
    const html = await renderMarkdown("[x](https://example.com)");
    expect(html).toContain('href="https://example.com"');
    expect(html).toContain('target="_blank"');
    expect(html).toContain('rel="noopener noreferrer"');
  });

  test("renders code blocks with copy icon", async () => {
    const html = await renderMarkdown("```rust\nfn main() {}\n```");
    expect(html).toContain('class="my-6');
    expect(html).toContain("lucide-copy-icon");
    expect(html).toContain("<svg");
  });

  test("keeps KaTeX output with MathML", async () => {
    const html = await renderMarkdown("$a^2$");
    expect(html).toContain("katex");
    expect(html).toContain("<math");
  });
});
