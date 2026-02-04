import { visit } from "unist-util-visit";

function createCopyIcon() {
  return {
    type: "element",
    tagName: "svg",
    properties: {
      xmlns: "http://www.w3.org/2000/svg",
      width: "16",
      height: "16",
      viewBox: "0 0 24 24",
      fill: "none",
      stroke: "currentColor",
      "stroke-width": "2",
      "stroke-linecap": "round",
      "stroke-linejoin": "round",
      className: ["lucide", "lucide-copy-icon", "lucide-copy"],
    },
    children: [
      {
        type: "element",
        tagName: "rect",
        properties: { width: "14", height: "14", x: "8", y: "8", rx: "2", ry: "2" },
        children: [],
      },
      {
        type: "element",
        tagName: "path",
        properties: {
          d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2",
        },
        children: [],
      },
    ],
  };
}

function createCopyButton(codeId) {
  return {
    type: "element",
    tagName: "button",
    properties: {
      className: [
        "copy-code-btn group/copy flex cursor-pointer items-center rounded p-1 transition-all",
      ],
      "data-code-id": codeId,
      title: "Copy code",
      type: "button",
    },
    children: [createCopyIcon()],
  };
}

function createHeader(language, codeId) {
  const actions = [createCopyButton(codeId)];

  return {
    type: "element",
    tagName: "div",
    properties: {
      className: [
        "code-block-header flex items-center justify-between px-4 py-2 text-[0.875rem]",
      ],
    },
    children: [
      {
        type: "element",
        tagName: "span",
        properties: {
          className: ["code-block-language text-[0.75rem] font-medium uppercase"],
        },
        children: [{ type: "text", value: language }],
      },
      {
        type: "element",
        tagName: "div",
        properties: { className: ["code-block-actions flex items-center"] },
        children: actions,
      },
    ],
  };
}

function createWrapper(header, preElement) {
  // Add styling to pre element
  if (preElement.properties) {
    preElement.properties.className = [
      "code-block-pre m-0 overflow-x-auto",
    ];
  }

  return {
    type: "element",
    tagName: "div",
    properties: {
      className: ["code-block-wrapper my-6 overflow-hidden rounded-xl shadow-xl"],
    },
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
