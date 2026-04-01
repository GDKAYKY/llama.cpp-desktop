function extractQuotedSteps(text) {
  const raw = String(text || "");
  if (!raw.trim()) return [];
  return [...raw.matchAll(/^\s*>\s*(.+)$/gm)].map((match) => match[1].trim());
}

function normalizeTagList(tags) {
  if (!Array.isArray(tags)) return [];
  return tags
    .map((tag) =>
      String(tag || "")
        .toLowerCase()
        .trim(),
    )
    .filter(Boolean);
}

function detectTagsFromText(text) {
  const raw = String(text || "");
  if (!raw.trim()) return [];
  const tags = new Set();
  const regex = /<\/?([a-zA-Z][a-zA-Z0-9_-]{0,32})>/g;
  let match = regex.exec(raw);
  while (match) {
    tags.add(match[1].toLowerCase());
    match = regex.exec(raw);
  }
  return [...tags];
}

function replaceThinkingTags(text, tags) {
  let result = String(text || "");
  const list = normalizeTagList(tags);
  for (const tag of list) {
    const safe = tag.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const open = new RegExp(`<${safe}>`, "gi");
    const close = new RegExp(`</${safe}>`, "gi");
    result = result.replace(close, "\n").replace(open, "");
  }
  return result;
}

function extractTaggedBlocks(text, tags) {
  const raw = String(text || "");
  if (!raw.trim()) return [];
  const list = normalizeTagList(tags);
  const blocks = [];
  for (const tag of list) {
    const safe = tag.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(`<${safe}>([\s\S]*?)</${safe}>`, "gi");
    let match = regex.exec(raw);
    while (match) {
      const chunk = String(match[1] || "")
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean);
      blocks.push(...chunk);
      match = regex.exec(raw);
    }
  }
  return blocks;
}

function mergeUniqueSteps(base, extra) {
  const result = [...base];
  for (const item of extra) {
    const trimmed = String(item || "").trim();
    if (!trimmed) continue;
    if (result[result.length - 1] === trimmed) continue;
    result.push(trimmed);
  }
  return result;
}

function extractThinkingSteps(entries, modelThinkingText, messageContent, tags) {
  const steps = Array.isArray(entries) ? [...entries] : [];
  const raw = String(modelThinkingText || "");
  const contentRaw = modelThinkingText ? "" : String(messageContent || "");

  const quotedThinking = extractQuotedSteps(raw);
  const quotedContent = extractQuotedSteps(contentRaw);
  const taggedThinking = extractTaggedBlocks(raw, tags);
  const taggedContent = extractTaggedBlocks(contentRaw, tags);
  const normalized = replaceThinkingTags(raw, tags)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  let parsed = [];
  if (quotedThinking.length > 0) {
    parsed = quotedThinking;
  } else if (taggedThinking.length > 0) {
    parsed = taggedThinking;
  } else if (normalized.length > 0) {
    parsed = normalized;
  } else if (taggedContent.length > 0) {
    parsed = taggedContent;
  } else if (quotedContent.length > 0) {
    parsed = quotedContent;
  }

  if (parsed.length === 0) return steps;
  if (raw.trim()) return parsed;
  return mergeUniqueSteps(steps, parsed);
}

function resolveThinkingTags(tagCandidates, modelThinkingText, messageContent) {
  const provided = normalizeTagList(tagCandidates);
  if (provided.length > 0) return provided;
  const detected = detectTagsFromText(`${modelThinkingText}\n${messageContent}`);
  if (detected.length > 0) return detected;
  return ["think", "analysis", "reasoning"];
}

function formatTagLabel(tag) {
  const normalized = String(tag || "")
    .replace(/[_-]+/g, " ")
    .trim();
  if (!normalized) return "Thinking";
  return normalized.replace(/\b\w/g, (c) => c.toUpperCase());
}

function detectTagLabel(modelThinkingText, messageContent, tags) {
  const raw = `${modelThinkingText || ""}\n${messageContent || ""}`.toLowerCase();
  for (const tag of normalizeTagList(tags)) {
    if (raw.includes(`<${tag}>`) || raw.includes(`</${tag}>`)) {
      return formatTagLabel(tag);
    }
  }
  return "";
}

function formatToolStep(toolName) {
  const raw = String(toolName || "").trim();
  if (!raw) return "Called tool";
  const normalized = raw.replace(/[_-]+/g, " ").trim();
  const lower = normalized.toLowerCase();
  if (lower.startsWith("list ")) return `Listed ${normalized.slice(5)}`;
  if (lower.startsWith("get ")) return `Got ${normalized.slice(4)}`;
  if (lower.startsWith("search ")) return `Searched ${normalized.slice(7)}`;
  if (lower.startsWith("find ")) return `Found ${normalized.slice(5)}`;
  if (lower.startsWith("read ")) return `Read ${normalized.slice(5)}`;
  return `Ran ${normalized}`;
}

export function summarizeThinking(
  entries,
  modelThinkingText,
  toolContextItems,
  labelFromTemplate,
  messageContent,
  tagCandidates,
) {
  const ignoredPatterns = [
    "tool loop",
    "no tool calls",
    "streaming final",
    "exceeded max iterations",
    "iteration",
  ];
  const detectedTags = resolveThinkingTags(
    tagCandidates,
    modelThinkingText,
    messageContent,
  );
  let cleaned = extractThinkingSteps(
    entries,
    modelThinkingText,
    messageContent,
    detectedTags,
  )
    .map((entry) => String(entry || "").trim())
    .filter(Boolean)
    .filter((entry) => {
      const text = entry.toLowerCase();
      return !ignoredPatterns.some((p) => text.includes(p));
    });

  let label = labelFromTemplate || "Thinking";
  if (toolContextItems.length > 0) {
    label = "Listed MCP tools";
  } else {
    const tagLabel = detectTagLabel(
      modelThinkingText,
      messageContent,
      detectedTags,
    );
    if (tagLabel) {
      label = tagLabel;
    } else {
      const lowered = cleaned.map((entry) => entry.toLowerCase());
      if (lowered.some((text) => text.includes("listing mcp"))) {
        label = "List MCP tools";
      } else if (lowered.some((text) => text.includes("calling mcp tool"))) {
        label = "Listed MCP tools";
      } else if (lowered.some((text) => text.includes("calling tool"))) {
        label = "Calling tool";
      }
    }
  }

  if (cleaned.length === 0 && toolContextItems.length > 0) {
    cleaned = toolContextItems.map((ctx) => formatToolStep(ctx?.toolName));
  }

  return { label, steps: cleaned };
}

export function groupThinkingSteps(steps) {
  const groups = [];
  let current = null;
  for (const rawStep of steps) {
    const step = String(rawStep || "").trim();
    if (!step) continue;
    const numbered = step.match(/^\d+\.\s+(.*)$/);
    const bullet = step.match(/^(?:\*|-|•)\s+(.*)$/);
    if (numbered) {
      if (current) groups.push(current);
      current = { title: numbered[1].trim(), items: [] };
      continue;
    }
    if (bullet) {
      if (!current) {
        groups.push({ title: bullet[1].trim(), items: [] });
      } else {
        current.items.push(bullet[1].trim());
      }
      continue;
    }
    if (current) {
      current.items.push(step);
    } else {
      groups.push({ title: step, items: [] });
    }
  }
  if (current) groups.push(current);
  return groups;
}

export function isToolStep(text) {
  const raw = String(text || "")
    .trim()
    .toLowerCase();
  return raw.startsWith("calling mcp tool") || raw.startsWith("called mcp tool");
}
