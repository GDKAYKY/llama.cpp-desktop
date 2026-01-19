import { visit } from "unist-util-visit";

const COPY_ICON_SVG = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-copy-icon lucide-copy"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>`;

function createRawHtmlElement(html) {
  return {
    type: "element",
    tagName: "span",
    properties: {},
    children: [{ type: "raw", value: html }],
  };
}

function createCopyButton(codeId) {
  return {
    type: "element",
    tagName: "button",
    properties: {
      className: ["copy-code-btn"],
      "data-code-id": codeId,
      title: "Copy code",
      type: "button",
    },
    children: [createRawHtmlElement(COPY_ICON_SVG)],
  };
}

function createHeader(language, codeId) {
  const actions = [createCopyButton(codeId)];

  return {
    type: "element",
    tagName: "div",
    properties: { className: ["code-block-header"] },
    children: [
      {
        type: "element",
        tagName: "span",
        properties: { className: ["code-language"] },
        children: [{ type: "text", value: language }],
      },
      {
        type: "element",
        tagName: "div",
        properties: { className: ["code-block-actions"] },
        children: actions,
      },
    ],
  };
}

function createWrapper(header, preElement) {
  return {
    type: "element",
    tagName: "div",
    properties: { className: ["code-block-wrapper"] },
    children: [header, preElement],
  };
}

function extractLanguage(codeElement) {
  const className = codeElement.properties?.className;
  if (!Array.isArray(className)) return "text";

  for (const cls of className) {
    if (typeof cls === "string" && cls.startsWith("language-")) {
      return cls.replace("language-", "");
    }
  }

  return "text";
}

function generateCodeId() {
  if (typeof window !== "undefined") {
    return `code-${(window.idxCodeBlock = (window.idxCodeBlock ?? 0) + 1)}`;
  }
  return `code-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
}

export const rehypeEnhanceCodeBlocks = () => {
  return (tree) => {
    visit(tree, "element", (node, index, parent) => {
      if (node.tagName !== "pre" || !parent || index === undefined) return;

      const codeElement = node.children.find(
        (child) => child.type === "element" && child.tagName === "code",
      );

      if (!codeElement) return;

      const language = extractLanguage(codeElement);
      const codeId = generateCodeId();

      codeElement.properties = {
        ...codeElement.properties,
        "data-code-id": codeId,
      };

      const header = createHeader(language, codeId);
      const wrapper = createWrapper(header, node);

      parent.children[index] = wrapper;
    });
  };
};
