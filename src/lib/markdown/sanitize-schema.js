import { defaultSchema } from "rehype-sanitize";

const EXTRA_TAGS = [
  "div",
  "span",
  "pre",
  "code",
  "table",
  "thead",
  "tbody",
  "tr",
  "th",
  "td",
  "ul",
  "ol",
  "li",
  "blockquote",
  "hr",
  "br",
  "button",
  "svg",
  "path",
  "rect",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "math",
  "semantics",
  "annotation",
  "mrow",
  "mi",
  "mo",
  "mn",
  "msup",
  "msub",
  "mfrac",
  "mtext",
  "mspace",
  "munder",
  "mover",
  "munderover",
  "mtable",
  "mtr",
  "mtd",
  "mlabeledtr",
];

const GLOBAL_ATTRS = ["className", "style", "aria-hidden", "aria-label", "role"];

const EXTRA_ATTRS = {
  a: ["href", "title", "target", "rel"],
  code: ["className", "data-code-id"],
  div: ["data-block-id"],
  button: ["type", "title", "data-code-id"],
  svg: [
    "xmlns",
    "width",
    "height",
    "viewBox",
    "fill",
    "stroke",
    "stroke-width",
    "stroke-linecap",
    "stroke-linejoin",
  ],
  rect: ["width", "height", "x", "y", "rx", "ry"],
  path: ["d"],
  math: ["xmlns"],
  annotation: ["encoding"],
  mi: ["mathvariant"],
  mo: ["mathvariant"],
  mn: ["mathvariant"],
  mtext: ["mathvariant"],
};

const clone = (value) => {
  if (typeof structuredClone === "function") {
    return structuredClone(value);
  }
  return JSON.parse(JSON.stringify(value));
};

const mergeUnique = (existing = [], additions = []) => {
  const merged = new Set(existing);
  for (const item of additions) {
    merged.add(item);
  }
  return Array.from(merged);
};

const baseSchema = clone(defaultSchema);

baseSchema.tagNames = mergeUnique(baseSchema.tagNames, EXTRA_TAGS);

baseSchema.attributes = {
  ...baseSchema.attributes,
  "*": mergeUnique(baseSchema.attributes?.["*"], GLOBAL_ATTRS),
};

for (const [tagName, attrs] of Object.entries(EXTRA_ATTRS)) {
  baseSchema.attributes[tagName] = mergeUnique(
    baseSchema.attributes?.[tagName],
    attrs,
  );
}

export const sanitizeSchema = baseSchema;
