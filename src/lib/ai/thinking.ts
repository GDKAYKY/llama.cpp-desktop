export type ParsedChunk =
  | { type: "content"; text: string }
  | { type: "thinking"; text: string };

const TAGS: Array<[string, string]> = [
  ["<think>", "</think>"],
  ["<analysis>", "</analysis>"],
  ["<reasoning>", "</reasoning>"],
];

const THINKING_TAGS_REGEX = /<\/?(?:think|analysis|reasoning)>/gi;

/**
 * Case-insensitive indexOf helper.
 */
function indexOfIgnoreCase(
  haystack: string,
  needle: string,
  fromIndex = 0,
): number {
  return haystack.toLowerCase().indexOf(needle.toLowerCase(), fromIndex);
}

/**
 * Case-insensitive startsWith helper.
 */
function startsWithIgnoreCase(text: string, prefix: string): boolean {
  return text.toLowerCase().startsWith(prefix.toLowerCase());
}

/**
 * Case-insensitive endsWith helper.
 */
function endsWithIgnoreCase(text: string, suffix: string): boolean {
  return text.toLowerCase().endsWith(suffix.toLowerCase());
}

function sanitizeContent(text: string): string {
  if (!text) return text;
  return text.replace(THINKING_TAGS_REGEX, "");
}

function sanitizeThinkingText(text: string): string {
  if (!text) return text;
  return text.replace(THINKING_TAGS_REGEX, "").trim();
}

export class ThinkingStreamParser {
  private inThinking = false;
  private buffer = "";
  private currentOpenTag = "<think>";
  private currentCloseTag = "</think>";
  private thinkingBlockCount = 0;
  private maxThinkingBlocks = 10; // Sanity limit to prevent infinite loops

  constructor(options?: { startInThinking?: boolean }) {
    if (options?.startInThinking) {
      this.inThinking = true;
    }
  }

  push(chunk: string): ParsedChunk[] {
    this.buffer += chunk;
    const results: ParsedChunk[] = [];

    while (true) {
      if (this.inThinking) {
        const closeIndex = findMatchingCloseTag(
          this.buffer,
          this.currentOpenTag,
          this.currentCloseTag,
        );

        if (closeIndex >= 0) {
          const text = this.buffer.slice(0, closeIndex);
          this.buffer = this.buffer.slice(
            closeIndex + this.currentCloseTag.length,
          );
          this.inThinking = false;
          this.currentOpenTag = "<think>";
          this.currentCloseTag = "</think>";
          this.thinkingBlockCount++;
          const sanitized = sanitizeThinkingText(text);
          if (sanitized) results.push({ type: "thinking", text: sanitized });
          continue;
        }

        if (couldBePartialTag(this.buffer, this.currentCloseTag)) break;
        break;
      }

      const nextOpen = findNextOpenTag(this.buffer);
      if (nextOpen && this.thinkingBlockCount < this.maxThinkingBlocks) {
        const { index, openTag, closeTag } = nextOpen;
        const text = this.buffer.slice(0, index);
        this.buffer = this.buffer.slice(index + openTag.length);
        this.inThinking = true;
        this.currentOpenTag = openTag;
        this.currentCloseTag = closeTag;
        if (text) {
          const sanitized = sanitizeContent(text);
          if (sanitized) results.push({ type: "content", text: sanitized });
        }
        continue;
      }

      const partialSuffix = findPartialTagSuffix(this.buffer);
      if (partialSuffix) {
        const safePrefix = this.buffer.slice(
          0,
          this.buffer.length - partialSuffix.length,
        );
        this.buffer = partialSuffix;
        if (safePrefix) {
          const sanitized = sanitizeContent(safePrefix);
          if (sanitized) results.push({ type: "content", text: sanitized });
        }
        break;
      }

      const text = this.buffer;
      this.buffer = "";
      if (text) {
        const sanitized = sanitizeContent(text);
        if (sanitized) results.push({ type: "content", text: sanitized });
      }
      break;
    }

    return results;
  }

  flush(): ParsedChunk[] {
    if (!this.buffer) return [];
    const text = this.buffer;
    this.buffer = "";

    const partialSuffix = findPartialTagSuffix(text);
    const safeText = partialSuffix
      ? text.slice(0, -partialSuffix.length)
      : text;

    if (this.inThinking) {
      const sanitized = sanitizeThinkingText(safeText);
      return sanitized ? [{ type: "thinking", text: sanitized }] : [];
    }

    const sanitized = sanitizeContent(safeText);
    return sanitized ? [{ type: "content", text: sanitized }] : [];
  }

  /**
   * Get the current parsing state for debugging purposes.
   */
  getState() {
    return {
      inThinking: this.inThinking,
      bufferLength: this.buffer.length,
      thinkingBlockCount: this.thinkingBlockCount,
      currentCloseTag: this.currentCloseTag,
    };
  }
}

function findMatchingCloseTag(
  text: string,
  openTag: string,
  closeTag: string,
): number {
  let depth = 1;
  let index = 0;

  while (index < text.length) {
    const nextOpen = indexOfIgnoreCase(text, openTag, index);
    const nextClose = indexOfIgnoreCase(text, closeTag, index);

    if (nextClose === -1) {
      return -1;
    }

    if (nextOpen !== -1 && nextOpen < nextClose) {
      depth += 1;
      index = nextOpen + openTag.length;
      continue;
    }

    depth -= 1;
    if (depth === 0) {
      return nextClose;
    }

    index = nextClose + closeTag.length;
  }

  return -1;
}

function findNextOpenTag(text: string) {
  let best: { index: number; openTag: string; closeTag: string } | null = null;
  for (const [openTag, closeTag] of TAGS) {
    let searchStart = 0;
    while (true) {
      const index = indexOfIgnoreCase(text, openTag, searchStart);
      if (index === -1) break;
      const isInsideCloseTag = index > 0 && text[index - 1] === "/";
      if (!isInsideCloseTag) {
        if (best === null || index < best.index) {
          best = { index, openTag, closeTag };
        }
        break;
      }
      searchStart = index + 1;
    }
  }
  return best;
}

function findPartialTagSuffix(text: string): string {
  let best = "";
  const tags = TAGS.flatMap(([openTag, closeTag]) => [openTag, closeTag]);

  for (const tag of tags) {
    if (endsWithIgnoreCase(text, tag)) {
      return "";
    }
    const maxCheck = Math.min(text.length, tag.length - 1);
    for (let i = 1; i <= maxCheck; i += 1) {
      const suffix = text.slice(-i);
      if (startsWithIgnoreCase(tag, suffix) && suffix.length > best.length) {
        best = suffix;
      }
    }
  }

  return best;
}

function couldBePartialTag(text: string, tag: string): boolean {
  const checkLength = Math.min(text.length, tag.length);
  for (let i = 1; i <= checkLength; i += 1) {
    if (startsWithIgnoreCase(tag, text.slice(-i))) return true;
  }
  return false;
}
