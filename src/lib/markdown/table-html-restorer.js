import { visit } from "unist-util-visit";
import { visitParents } from "unist-util-visit-parents";
import {
  BR_PATTERN,
  LIST_PATTERN,
  LI_PATTERN,
} from "../constants/table-html-restorer.js";

function expandBrTags(value) {
  const matches = [...value.matchAll(BR_PATTERN)];
  if (!matches.length) return [{ type: "text", value }];

  const result = [];
  let cursor = 0;

  for (const m of matches) {
    if (m.index > cursor) {
      result.push({ type: "text", value: value.slice(cursor, m.index) });
    }
    result.push({
      type: "element",
      tagName: "br",
      properties: {},
      children: [],
    });
    cursor = m.index + m[0].length;
  }

  if (cursor < value.length) {
    result.push({ type: "text", value: value.slice(cursor) });
  }

  return result;
}

function parseList(value) {
  const match = value.trim().match(LIST_PATTERN);
  if (!match) return null;

  const body = match[1];
  const items = [];
  let cursor = 0;

  for (const liMatch of body.matchAll(LI_PATTERN)) {
    if (body.slice(cursor, liMatch.index).trim()) return null;
    items.push({
      type: "element",
      tagName: "li",
      properties: {},
      children: expandBrTags(liMatch[1] ?? ""),
    });
    cursor = liMatch.index + liMatch[0].length;
  }

  if (!items.length || body.slice(cursor).trim()) return null;
  return { type: "element", tagName: "ul", properties: {}, children: items };
}

function processCell(cell) {
  visitParents(cell, "text", (textNode, ancestors) => {
    const parent = ancestors[ancestors.length - 1];
    if (!parent || parent.type !== "element") return;

    const parentEl = parent;
    const siblings = parentEl.children;
    const startIndex = siblings.indexOf(textNode);
    if (startIndex === -1) return;

    let combined = "";
    let endIndex = startIndex;

    for (let i = startIndex; i < siblings.length; i++) {
      const sib = siblings[i];
      if (sib.type === "text") {
        combined += sib.value;
        endIndex = i;
      } else if (sib.type === "element" && sib.tagName === "br") {
        combined += "\n";
        endIndex = i;
      } else {
        break;
      }
    }

    const list = parseList(combined);
    if (list) {
      siblings.splice(startIndex, endIndex - startIndex + 1, list);
      return;
    }

    const expanded = expandBrTags(textNode.value);
    if (expanded.length !== 1 || expanded[0] !== textNode) {
      siblings.splice(startIndex, 1, ...expanded);
    }
  });
}

export const rehypeRestoreTableHtml = () => (tree) => {
  visit(tree, "element", (node) => {
    if (node.tagName === "td" || node.tagName === "th") {
      processCell(node);
    }
  });
};
