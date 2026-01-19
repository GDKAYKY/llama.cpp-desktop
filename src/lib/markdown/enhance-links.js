import { visit } from "unist-util-visit";

export const rehypeEnhanceLinks = () => {
  return (tree) => {
    visit(tree, "element", (node) => {
      if (node.tagName !== "a") return;

      const props = node.properties ?? {};
      if (!props.href) return;

      props.target = "_blank";
      props.rel = "noopener noreferrer";
      node.properties = props;
    });
  };
};
