import { describe, it, expect } from "vitest";
import { ThinkingStreamParser } from "./thinking";

/**
 * End-to-end integration test: Verify reasoning never leaks into assistant output.
 * Simulates the full pipeline from streamed chunks → parser → content callbacks.
 */
describe("Qwen3.5 Thinking Mode Integration", () => {
  it("should prevent reasoning leak through full streaming pipeline", () => {
    const parser = new ThinkingStreamParser({ startInThinking: true });
    let assistantContent = "";
    let thinkingContent = "";

    // When startInThinking: true, parser is already in thinking mode.
    // All content flows as thinking until first </think>.
    const stream = [
      "Let me analyze this problem",
      "\nStep 1: Check if the input is valid",
      "\nStep 2: Process the data",
      "</think>", // Exits thinking mode
      "\nBased on my analysis, the answer is: 42",
      "</think>", // Orphaned close (stripped)
      "\nThis is additional content.",
    ];

    for (const chunk of stream) {
      for (const parsed of parser.push(chunk)) {
        if (parsed.type === "thinking") {
          thinkingContent += parsed.text;
        } else {
          assistantContent += parsed.text;
        }
      }
    }

    for (const parsed of parser.flush()) {
      if (parsed.type === "thinking") {
        thinkingContent += parsed.text;
      } else {
        assistantContent += parsed.text;
      }
    }

    // Assertions: thinking is separated from content
    expect(thinkingContent).toContain("Let me analyze this problem");
    expect(thinkingContent).toContain("Step 1");
    expect(assistantContent).toContain(
      "Based on my analysis, the answer is: 42",
    );
    expect(assistantContent).toContain("This is additional content");

    // Critical: No reasoning or tags in assistant output
    expect(assistantContent).not.toContain("<think>");
    expect(assistantContent).not.toContain("</think>");
    expect(assistantContent).not.toContain("Let me analyze");
    expect(assistantContent).not.toContain("Step 1");
    expect(assistantContent).not.toContain("Step 2");
  });

  it("should handle MCP tool integration with thinking mode", () => {
    const parser = new ThinkingStreamParser({ startInThinking: true });
    let assistantContent = "";
    let thinkingContent = "";

    // When thinking mode identifies tool needs, those thoughts stay hidden
    const stream = [
      "I need to search for information",
      "\nCalling search tool...",
      "</think>", // Exits thinking mode
      "\nThe search results show: [data]",
      "\nHere's my response based on the search.",
    ];

    for (const chunk of stream) {
      for (const parsed of parser.push(chunk)) {
        if (parsed.type === "thinking") {
          thinkingContent += parsed.text;
        } else if (parsed.type === "content") {
          assistantContent += parsed.text;
        }
      }
    }

    for (const parsed of parser.flush()) {
      if (parsed.type === "thinking") {
        thinkingContent += parsed.text;
      } else if (parsed.type === "content") {
        assistantContent += parsed.text;
      }
    }

    // Tool planning stays in thinking
    expect(thinkingContent).toContain("I need to search");
    expect(thinkingContent).toContain("Calling search tool");

    // Assistant response is clean
    expect(assistantContent).toContain("The search results show");
    expect(assistantContent).toContain("Here's my response");
    expect(assistantContent).not.toContain("I need to search");
    expect(assistantContent).not.toContain("Calling search tool");
  });

  it("should recover from deeply malformed streaming with duplicate tags", () => {
    const parser = new ThinkingStreamParser({ startInThinking: true });
    let assistantContent = "";

    // Worst-case: malformed, duplicated, orphaned tags
    const stream = [
      "reasoning",
      "</think>", // First close exits thinking mode
      "</think>", // Orphaned
      "Valid content",
      "</think>", // Orphaned
      "</think>", // Orphaned
      " more content",
      "<</think>>", // Malformed
    ];

    for (const chunk of stream) {
      for (const parsed of parser.push(chunk)) {
        if (parsed.type === "content") {
          assistantContent += parsed.text;
        }
      }
    }

    for (const parsed of parser.flush()) {
      if (parsed.type === "content") {
        assistantContent += parsed.text;
      }
    }

    // All closing tags stripped, only valid content remains
    expect(assistantContent).toContain("Valid content");
    expect(assistantContent).toContain("more content");
    expect(assistantContent).not.toContain("</think>");
    expect(assistantContent).not.toContain("<</think>>");
  });

  it("should handle normal mode with explicit thinking tags", () => {
    const parser = new ThinkingStreamParser();
    let assistantContent = "";
    let thinkingContent = "";

    // Normal mode (startInThinking: false): tags are explicit in stream
    const stream = [
      "Start: ",
      "<think>internal reasoning here</think>",
      "Answer: 42",
    ];

    for (const chunk of stream) {
      for (const parsed of parser.push(chunk)) {
        if (parsed.type === "thinking") {
          thinkingContent += parsed.text;
        } else {
          assistantContent += parsed.text;
        }
      }
    }

    for (const parsed of parser.flush()) {
      if (parsed.type === "thinking") {
        thinkingContent += parsed.text;
      } else {
        assistantContent += parsed.text;
      }
    }

    expect(thinkingContent).toBe("internal reasoning here");
    expect(assistantContent).toContain("Start:");
    expect(assistantContent).toContain("Answer: 42");
    expect(assistantContent).not.toContain("internal reasoning");
    expect(assistantContent).not.toContain("<think>");
  });

  it("should handle case-insensitive tags in real stream", () => {
    const parser = new ThinkingStreamParser({ startInThinking: true });
    let assistantContent = "";
    let thinkingContent = "";

    // Real-world scenario: model outputs mixed-case tags
    const stream = [
      "Analyzing the problem",
      "\nConsidering options",
      "</THINK>", // Uppercase close
      "\nThe answer is 42.",
    ];

    for (const chunk of stream) {
      for (const parsed of parser.push(chunk)) {
        if (parsed.type === "thinking") {
          thinkingContent += parsed.text;
        } else {
          assistantContent += parsed.text;
        }
      }
    }

    for (const parsed of parser.flush()) {
      if (parsed.type === "thinking") {
        thinkingContent += parsed.text;
      } else {
        assistantContent += parsed.text;
      }
    }

    expect(thinkingContent).toContain("Analyzing");
    expect(thinkingContent).toContain("Considering options");
    expect(assistantContent).toContain("The answer is 42");
    expect(assistantContent).not.toContain("Analyzing");
    expect(assistantContent).not.toContain("</THINK>");
  });
});
