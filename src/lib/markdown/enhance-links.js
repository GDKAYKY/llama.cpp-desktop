import { visit } from "unist-util-visit";

export const rehypeEnhanceLinks = () => {
  return (tree) => {
    visit(tree, "element", (node, index, parent) => {
      if (node.tagName !== "a") return;

      const props = node.properties ?? {};
      if (!props.href) return;

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
        children: [{ type: "text", value: hostname }],
      };

      const arrowNode = {
        type: "element",
        tagName: "span",
        properties: { className: ["mcp-link-arrow"] },
        children: [{ type: "text", value: "↗" }],
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
