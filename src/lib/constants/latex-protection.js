/**
 * Matches common Markdown code blocks to exclude them from further processing (e.g. LaTeX).
 */
export const CODE_BLOCK_REGEXP = /(```[\s\S]*?```|`[^`\n]+`)/g;

/**
 * Matches LaTeX math delimiters \(...\) and \[...\] only when not preceded by a backslash,
 * while also capturing code blocks (```, `...`) so they can be skipped during processing.
 */
export const LATEX_MATH_AND_CODE_PATTERN =
  /(```[\S\s]*?```|`.*?`)|(?<!\\)\\\[([\S\s]*?[^\\])\\]|(?<!\\)\\\((.*?)\\\)/g;

/** Regex to capture the content of a $$...\\\\...$$ block (display-formula with line-break) */
export const LATEX_LINEBREAK_REGEXP = /\$\$([\s\S]*?\\\\[\s\S]*?)\$\$/;

/** map from mchem-regexp to replacement */
export const MHCHEM_PATTERN_MAP = [
  [/(\s)\$\\ce{/g, "$1$\\\\ce{"],
  [/(\s)\$\\pu{/g, "$1$\\\\pu{"],
];
