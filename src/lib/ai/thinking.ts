export type ParsedChunk =
  | { type: "content"; text: string }
  | { type: "thinking"; text: string };

const TAGS: Array<[string, string]> = [
  ["<think>", "</think>"],
  ["<analysis>", "</analysis>"],
  ["<reasoning>", "</reasoning>"],
];

export class ThinkingStreamParser {
  private inThinking = false;
  private buffer = "";
  private currentCloseTag = "</think>";

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
        const closeIndex = this.buffer.indexOf(this.currentCloseTag);
        if (closeIndex >= 0) {
          const text = this.buffer.slice(0, closeIndex);
          this.buffer = this.buffer.slice(closeIndex + this.currentCloseTag.length);
          this.inThinking = false;
          this.currentCloseTag = "</think>";
          if (text) results.push({ type: "thinking", text });
          continue;
        }

        if (couldBePartialTag(this.buffer, this.currentCloseTag)) break;
        const text = this.buffer;
        this.buffer = "";
        if (text) results.push({ type: "thinking", text });
        break;
      }

      const nextOpen = findNextOpenTag(this.buffer);
      if (nextOpen) {
        const { index, openTag, closeTag } = nextOpen;
        const text = this.buffer.slice(0, index);
        this.buffer = this.buffer.slice(index + openTag.length);
        this.inThinking = true;
        this.currentCloseTag = closeTag;
        if (text) results.push({ type: "content", text });
        continue;
      }

      if (TAGS.some(([openTag]) => couldBePartialTag(this.buffer, openTag))) break;
      const text = this.buffer;
      this.buffer = "";
      if (text) results.push({ type: "content", text });
      break;
    }

    return results;
  }

  flush(): ParsedChunk[] {
    if (!this.buffer) return [];
    const text = this.buffer;
    this.buffer = "";
    return [{ type: this.inThinking ? "thinking" : "content", text }];
  }
}

function findNextOpenTag(text: string) {
  let best: { index: number; openTag: string; closeTag: string } | null = null;
  for (const [openTag, closeTag] of TAGS) {
    const index = text.indexOf(openTag);
    if (index >= 0 && (!best || index < best.index)) {
      best = { index, openTag, closeTag };
    }
  }
  return best;
}

function couldBePartialTag(text: string, tag: string): boolean {
  const checkLength = Math.min(text.length, tag.length);
  for (let i = 1; i <= checkLength; i += 1) {
    if (tag.startsWith(text.slice(-i))) return true;
  }
  return false;
}
