import {
  CODE_BLOCK_REGEXP,
  LATEX_MATH_AND_CODE_PATTERN,
  LATEX_LINEBREAK_REGEXP,
  MHCHEM_PATTERN_MAP,
} from "../constants/latex-protection.js";

export function maskInlineLaTeX(content, latexExpressions) {
  if (!content.includes("$")) {
    return content;
  }
  return content
    .split("\n")
    .map((line) => {
      if (line.indexOf("$") == -1) {
        return line;
      }

      let processedLine = "";
      let currentPosition = 0;

      while (currentPosition < line.length) {
        const openDollarIndex = line.indexOf("$", currentPosition);

        if (openDollarIndex == -1) {
          processedLine += line.slice(currentPosition);
          break;
        }

        const closeDollarIndex = line.indexOf("$", openDollarIndex + 1);

        if (closeDollarIndex == -1) {
          processedLine += line.slice(currentPosition);
          break;
        }

        const charBeforeOpen =
          openDollarIndex > 0 ? line[openDollarIndex - 1] : "";
        const charAfterOpen = line[openDollarIndex + 1];
        const charBeforeClose =
          openDollarIndex + 1 < closeDollarIndex
            ? line[closeDollarIndex - 1]
            : "";
        const charAfterClose =
          closeDollarIndex + 1 < line.length ? line[closeDollarIndex + 1] : "";

        let shouldSkipAsNonLatex = false;

        if (closeDollarIndex == currentPosition + 1) {
          shouldSkipAsNonLatex = true;
        }

        if (/[A-Za-z0-9_$-]/.test(charBeforeOpen)) {
          shouldSkipAsNonLatex = true;
        }

        if (
          /[0-9]/.test(charAfterOpen) &&
          (/[A-Za-z0-9_$-]/.test(charAfterClose) || " " == charBeforeClose)
        ) {
          shouldSkipAsNonLatex = true;
        }

        if (shouldSkipAsNonLatex) {
          processedLine += line.slice(currentPosition, openDollarIndex + 1);
          currentPosition = openDollarIndex + 1;
          continue;
        }

        processedLine += line.slice(currentPosition, openDollarIndex);
        const latexContent = line.slice(openDollarIndex, closeDollarIndex + 1);
        latexExpressions.push(latexContent);
        processedLine += `<<LATEX_${latexExpressions.length - 1}>>`;
        currentPosition = closeDollarIndex + 1;
      }

      return processedLine;
    })
    .join("\n");
}

function escapeBrackets(text) {
  return text.replace(
    LATEX_MATH_AND_CODE_PATTERN,
    (match, codeBlock, squareBracket, roundBracket) => {
      if (codeBlock != null) {
        return codeBlock;
      } else if (squareBracket != null) {
        return `$$${squareBracket}$$`;
      } else if (roundBracket != null) {
        return `$${roundBracket}$`;
      }
      return match;
    },
  );
}

export function preprocessLaTeX(content) {
  const blockquoteMarkers = new Map();
  const lines = content.split("\n");
  const processedLines = lines.map((line, index) => {
    const match = line.match(/^(>\s*)/);
    if (match) {
      blockquoteMarkers.set(index, match[1]);
      return line.slice(match[1].length);
    }
    return line;
  });
  content = processedLines.join("\n");

  const codeBlocks = [];
  content = content.replace(CODE_BLOCK_REGEXP, (match) => {
    codeBlocks.push(match);
    return `<<CODE_BLOCK_${codeBlocks.length - 1}>>`;
  });

  const latexExpressions = [];
  content = content.replace(
    /([\S].*?)\\\[([\s\S]*?)\\\](.*)/g,
    (match, group1, group2, group3) => {
      if (group1.endsWith("\\")) {
        return match;
      }
      const hasSuffix = /\S/.test(group3);
      let optBreak;
      if (hasSuffix) {
        latexExpressions.push(`\\(${group2.trim()}\\)`);
        optBreak = "";
      } else {
        latexExpressions.push(`\\[${group2}\\]`);
        optBreak = "\n";
      }
      return `${group1}${optBreak}<<LATEX_${latexExpressions.length - 1}>>${optBreak}${group3}`;
    },
  );

  content = content.replace(
    /(\$\$[\s\S]*?\$\$|(?<!\\)\\\[[\s\S]*?\\\]|(?<!\\)\\\(.*?\\\))/g,
    (match) => {
      latexExpressions.push(match);
      return `<<LATEX_${latexExpressions.length - 1}>>`;
    },
  );

  content = maskInlineLaTeX(content, latexExpressions);
  content = content.replace(/\$(?=\d)/g, "\\$");

  content = content.replace(/<<LATEX_(\d+)>>/g, (_, index) => {
    let expr = latexExpressions[parseInt(index)];
    const match = expr.match(LATEX_LINEBREAK_REGEXP);
    if (match) {
      const formula = match[1];
      const prefix = formula.startsWith("\n") ? "" : "\n";
      const suffix = formula.endsWith("\n") ? "" : "\n";
      expr = "$$" + prefix + formula + suffix + "$$";
    }
    return expr;
  });

  content = escapeBrackets(content);
  content = content
    .replace(/(?<!\\)\\\((.+?)\\\)/g, "$$$1$")
    .replace(/(?<!\\)\\\[([\s\S]*?)\\\]/g, (_, content) => `$$${content}$$`);

  content = content.replace(/<<CODE_BLOCK_(\d+)>>/g, (_, index) => {
    return codeBlocks[parseInt(index)];
  });

  if (blockquoteMarkers.size > 0) {
    const finalLines = content.split("\n");
    const restoredLines = finalLines.map((line, index) => {
      const marker = blockquoteMarkers.get(index);
      return marker ? marker + line : line;
    });
    content = restoredLines.join("\n");
  }

  return content;
}
