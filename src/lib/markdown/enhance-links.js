import { visit } from "unist-util-visit";

const SAFE_PROTOCOLS = new Set(["http:", "https:", "mailto:", "tel:"]);
const SCHEME_PATTERN = /^[a-zA-Z][a-zA-Z0-9+.-]*:/;

function isSafeUrl(rawHref) {
  if (!rawHref) return false;
  const href = String(rawHref).trim();

  if (
    href.startsWith("#") ||
    href.startsWith("/") ||
    href.startsWith("./") ||
    href.startsWith("../") ||
    href.startsWith("//")
  ) {
    return true;
  }

  if (!SCHEME_PATTERN.test(href)) {
    return true;
  }

  try {
    const { protocol } = new URL(href);
    return SAFE_PROTOCOLS.has(protocol);
  } catch {
    return false;
  }
}

function extractText(node) {
  if (!node || !node.children) return "";
  const parts = [];

  const walk = (child) => {
    if (!child) return;
    if (child.type === "text") {
      parts.push(child.value ?? "");
      return;
    }
    if (Array.isArray(child.children)) {
      for (const nested of child.children) {
        walk(nested);
      }
    }
  };

  for (const child of node.children) {
    walk(child);
  }

  return parts.join("").trim();
}

export const rehypeEnhanceLinks = () => {
  return (tree) => {
    visit(tree, "element", (node, index, parent) => {
      if (node.tagName !== "a") return;

      const props = node.properties ?? {};
      if (!props.href) return;

      if (!isSafeUrl(props.href)) {
        node.tagName = "span";
        node.properties = {};
        return;
      }

      props.target = "_blank";
      props.rel = "noopener noreferrer";
      props.className = ["mcp-link"];
      node.properties = props;

      if (!parent || typeof index !== "number") return;

      let hostname = String(props.href);
      try {
        hostname = new URL(String(props.href)).hostname;
      } catch {
        hostname = String(props.href);
      }

      const sourceNode = {
        type: "element",
        tagName: "span",
        properties: { className: ["mcp-link-source"] },
        children: [
          {
            type: "element",
            tagName: "img",
            properties: {
              className: ["mcp-link-favicon"],
              src: `https://www.google.com/s2/favicons?sz=64&domain_url=${hostname}`,
              "data-fallback": `https://icons.duckduckgo.com/ip3/${hostname}.ico`,
              alt: `${hostname} favicon`,
              width: "12",
              height: "12",
              loading: "lazy",
              decoding: "async",
              referrerpolicy: "no-referrer",
            },
            children: [],
          },
          { type: "text", value: hostname },
          {
            type: "element",
            tagName: "span",
            properties: { className: ["mcp-link-tooltip"] },
            children: [
              {
                type: "element",
                tagName: "span",
                properties: { className: ["mcp-link-tooltip-card"] },
                children: [
                  {
                    type: "element",
                    tagName: "span",
                    properties: { className: ["mcp-link-tooltip-title"] },
                    children: [
                      {
                        type: "element",
                        tagName: "span",
                        properties: { className: ["mcp-link-tooltip-icon"] },
                        children: [
                          {
                            type: "element",
                            tagName: "svg",
                            properties: {
                              xmlns: "http://www.w3.org/2000/svg",
                              width: "14",
                              height: "14",
                              viewBox: "0 0 24 24",
                              fill: "none",
                              stroke: "currentColor",
                              "stroke-width": "2",
                              "stroke-linecap": "round",
                              "stroke-linejoin": "round",
                              "aria-hidden": "true",
                              className: ["lucide", "lucide-link-2"],
                            },
                            children: [
                              {
                                type: "element",
                                tagName: "path",
                                properties: {
                                  d: "M15 7h3a5 5 0 0 1 0 10h-3",
                                },
                                children: [],
                              },
                              {
                                type: "element",
                                tagName: "path",
                                properties: {
                                  d: "M9 17H6a5 5 0 0 1 0-10h3",
                                },
                                children: [],
                              },
                              {
                                type: "element",
                                tagName: "line",
                                properties: { x1: "8", y1: "12", x2: "16", y2: "12" },
                                children: [],
                              },
                            ],
                          },
                        ],
                      },
                      {
                        type: "element",
                        tagName: "img",
                        properties: {
                          className: ["mcp-link-tooltip-favicon"],
                          src: `https://www.google.com/s2/favicons?sz=64&domain_url=${hostname}`,
                          "data-fallback": `https://icons.duckduckgo.com/ip3/${hostname}.ico`,
                          alt: `${hostname} favicon`,
                          width: "14",
                          height: "14",
                          loading: "lazy",
                          decoding: "async",
                          referrerpolicy: "no-referrer",
                        },
                        children: [],
                      },
                      { type: "text", value: hostname },
                    ],
                  },
                  {
                    type: "element",
                    tagName: "span",
                    properties: { className: ["mcp-link-tooltip-subtitle"] },
                    children: [
                      {
                        type: "text",
                        value: [
                          extractText(node) || hostname,
                          hostname,
                        ].join(" | "),
                      },
                    ],
                  },
                ],
              },
            ],
          },
        ],
      };

      const arrowNode = {
        type: "element",
        tagName: "svg",
        properties: {
          xmlns: "http://www.w3.org/2000/svg",
          width: "24",
          height: "24",
          viewBox: "0 0 24 24",
          fill: "none",
          stroke: "currentColor",
          "stroke-width": "2",
          "stroke-linecap": "round",
          "stroke-linejoin": "round",
          "aria-hidden": "true",
          className: ["mcp-link-arrow", "lucide", "lucide-external-link"],
        },
        children: [
          {
            type: "element",
            tagName: "path",
            properties: { d: "M7 7h10v10" },
            children: [],
          },
          {
            type: "element",
            tagName: "path",
            properties: { d: "M7 17 17 7" },
            children: [],
          },
        ],
      };

      node.children = [...(node.children ?? []), arrowNode];

      const wrapper = {
        type: "element",
        tagName: "span",
        properties: { className: ["mcp-link-wrap"] },
        children: [node, sourceNode],
      };

      parent.children[index] = wrapper;
    });
  };
};
