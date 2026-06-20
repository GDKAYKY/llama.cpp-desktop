import { describe, it, expect } from "vitest";
import { ThinkingStreamParser, type ParsedChunk } from "./thinking";

describe("ThinkingStreamParser", () => {
  describe("Basic thinking block extraction", () => {
    it("should extract simple thinking block", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<think>reasoning here</think>final answer");

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "reasoning here" });
      expect(result[1]).toEqual({ type: "content", text: "final answer" });
    });

    it("should handle thinking block at start", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<think>thinking</think>");

      expect(result).toHaveLength(1);
      expect(result[0]).toEqual({ type: "thinking", text: "thinking" });
    });

    it("should handle content before thinking", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("prefix<think>thinking</think>suffix");

      expect(result).toHaveLength(3);
      expect(result[0]).toEqual({ type: "content", text: "prefix" });
      expect(result[1]).toEqual({ type: "thinking", text: "thinking" });
      expect(result[2]).toEqual({ type: "content", text: "suffix" });
    });
  });

  describe("Orphaned closing tag handling (PRIMARY FIX)", () => {
    it("should strip orphaned </think> tag", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("Final answer with orphaned </think> tag");

      expect(result).toHaveLength(1);
      expect(result[0]).toEqual({
        type: "content",
        text: "Final answer with orphaned  tag",
      });
    });

    it("should strip multiple orphaned closing tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("Content </think> more content </think> end");

      expect(result).toHaveLength(1);
      expect(result[0]).toEqual({
        type: "content",
        text: "Content  more content  end",
      });
    });

    it("should handle orphaned tags after flush", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("Some text");
      expect(result).toHaveLength(1);
      expect(result[0]).toEqual({ type: "content", text: "Some text" });

      const flushed = parser.flush();
      expect(flushed).toHaveLength(0);
    });

    it("should handle orphaned tag in flushed content", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("Text with orphaned </think>");

      expect(result).toHaveLength(1);
      expect(result[0].type).toBe("content");
      expect(result[0].text).not.toContain("</think>");

      const flushed = parser.flush();
      expect(flushed).toHaveLength(0);
    });
  });

  describe("Empty and malformed thinking blocks", () => {
    it("should remove empty thinking blocks", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<think></think>Content");

      expect(result).toHaveLength(1);
      expect(result[0]).toEqual({ type: "content", text: "Content" });
    });

    it("should remove empty thinking with whitespace", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<think>   </think>Real content");

      expect(result).toHaveLength(1);
      expect(result[0]).toEqual({ type: "content", text: "Real content" });
    });

    it("should handle duplicate closing tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push(
        "<think>reasoning</think></think>Final answer",
      );

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "reasoning" });
      expect(result[1]).toEqual({ type: "content", text: "Final answer" });
    });
  });

  describe("Streaming simulation (partial chunks)", () => {
    it("should handle thinking block split across chunks", () => {
      const parser = new ThinkingStreamParser();
      const chunk1 = parser.push("<think>part");
      const chunk2 = parser.push(" 1");
      const chunk3 = parser.push("</think>Done");

      expect(chunk1).toHaveLength(0); // Buffered
      expect(chunk2).toHaveLength(0); // Buffered
      expect(chunk3).toHaveLength(2);
      expect(chunk3[0]).toEqual({ type: "thinking", text: "part 1" });
      expect(chunk3[1]).toEqual({ type: "content", text: "Done" });
    });

    it("should handle orphaned tag split across chunks", () => {
      const parser = new ThinkingStreamParser();
      const chunk1 = parser.push("Content <");
      const chunk2 = parser.push("/think>");

      expect(chunk1).toHaveLength(1);
      expect(chunk1[0]).toEqual({ type: "content", text: "Content " });
      expect(chunk2).toHaveLength(0); // Orphaned tag stripped in first chunk
    });

    it("should buffer partial tags", () => {
      const parser = new ThinkingStreamParser();
      const result1 = parser.push("Content with <");
      const result2 = parser.push("think>reasoning</think>");

      expect(result1).toHaveLength(1);
      expect(result1[0]).toEqual({ type: "content", text: "Content with " });
      expect(result2).toHaveLength(1);
      expect(result2[0]).toEqual({ type: "thinking", text: "reasoning" });
    });
  });

  describe("Multiple thinking blocks", () => {
    it("should extract multiple thinking blocks", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push(
        "<think>First</think>Middle<think>Second</think>End",
      );

      expect(result).toHaveLength(4);
      expect(result[0]).toEqual({ type: "thinking", text: "First" });
      expect(result[1]).toEqual({ type: "content", text: "Middle" });
      expect(result[2]).toEqual({ type: "thinking", text: "Second" });
      expect(result[3]).toEqual({ type: "content", text: "End" });
    });

    it("should limit thinking blocks to prevent infinite loops", () => {
      const parser = new ThinkingStreamParser();
      // Create a stream with many thinking blocks
      let input = "";
      for (let i = 0; i < 20; i++) {
        input += `<think>Block${i}</think>`;
      }
      const result = parser.push(input);

      // Should stop after max blocks
      const thinkingChunks = result.filter((c) => c.type === "thinking");
      expect(thinkingChunks.length).toBeLessThanOrEqual(10);
    });
  });

  describe("Other tag types", () => {
    it("should handle <analysis> tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<analysis>analyzing</analysis>conclusion");

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "analyzing" });
      expect(result[1]).toEqual({ type: "content", text: "conclusion" });
    });

    it("should handle <reasoning> tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<reasoning>logical steps</reasoning>result");

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "logical steps" });
      expect(result[1]).toEqual({ type: "content", text: "result" });
    });

    it("should strip orphaned tags of all types", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("Text</analysis>more</reasoning>end</think>");

      expect(result).toHaveLength(1);
      expect(result[0].type).toBe("content");
      expect(result[0].text).not.toContain("</analysis>");
      expect(result[0].text).not.toContain("</reasoning>");
      expect(result[0].text).not.toContain("</think>");
    });
  });

  describe("Real-world scenarios", () => {
    it("should handle tool result with orphaned tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push(
        "<think>I need to search for temperature</think>Based on search: 78°F</think>",
      );

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({
        type: "thinking",
        text: "I need to search for temperature",
      });
      expect(result[1].type).toBe("content");
      expect(result[1].text).toBe("Based on search: 78°F");
    });

    it("should recover from duplicate closing and trailing thinking text", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push(
        "<think>reasoning</think></think>Final answer",
      );

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "reasoning" });
      expect(result[1]).toEqual({ type: "content", text: "Final answer" });
    });

    it("should handle nested thinking tags without leaking tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push(
        "<think>outer <think>inner</think> still outer</think>done",
      );

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({
        type: "thinking",
        text: "outer inner still outer",
      });
      expect(result[1]).toEqual({ type: "content", text: "done" });
    });

    it("should handle startInThinking mode", () => {
      const parser = new ThinkingStreamParser({ startInThinking: true });
      const result = parser.push("Initial thinking");
      const result2 = parser.push("</think>Final answer");

      expect(result).toHaveLength(0); // Buffered
      expect(result2).toHaveLength(2);
      expect(result2[0]).toEqual({
        type: "thinking",
        text: "Initial thinking",
      });
      expect(result2[1]).toEqual({ type: "content", text: "Final answer" });
    });

    it("should provide debug state", () => {
      const parser = new ThinkingStreamParser();
      parser.push("<think>test</think>");

      const state = parser.getState();
      expect(state).toHaveProperty("inThinking");
      expect(state).toHaveProperty("bufferLength");
      expect(state).toHaveProperty("thinkingBlockCount");
    });
  });

  describe("Edge cases and recovery", () => {
    it("should recover from malformed nesting", () => {
      const parser = new ThinkingStreamParser();
      // Nested thinking tags (should treat as content after first close)
      const result = parser.push(
        "<think>outer <think>inner</think> still outer</think>done",
      );

      expect(result.length).toBeGreaterThan(0);
      const finalContent = result
        .filter((c) => c.type === "content")
        .map((c) => c.text)
        .join("");
      expect(finalContent).not.toContain("<think>");
      expect(finalContent).not.toContain("</think>");
    });

    it("should handle unclosed thinking block at end", () => {
      const parser = new ThinkingStreamParser();
      parser.push("<think>Unclosed thinking");
      const flushed = parser.flush();

      expect(flushed).toHaveLength(1);
      expect(flushed[0].type).toBe("thinking");
    });

    it("should clean up on flush with orphaned tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("Content</think>");

      expect(result).toHaveLength(1);
      expect(result[0].text).toBe("Content");
      expect(result[0].text).not.toContain("</think>");

      const flushed = parser.flush();
      expect(flushed).toHaveLength(0);
    });

    it("should handle very long content gracefully", () => {
      const parser = new ThinkingStreamParser();
      const longThinking = "x".repeat(10000);
      const result = parser.push(`<think>${longThinking}</think>Short answer`);

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: longThinking });
      expect(result[1]).toEqual({ type: "content", text: "Short answer" });
    });
  });

  describe("Case-insensitive tag handling", () => {
    it("should handle uppercase thinking tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<THINK>reasoning</THINK>answer");

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "reasoning" });
      expect(result[1]).toEqual({ type: "content", text: "answer" });
    });

    it("should handle mixed-case thinking tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<Think>reasoning</Think>answer");

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "reasoning" });
      expect(result[1]).toEqual({ type: "content", text: "answer" });
    });

    it("should handle uppercase analysis tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("<ANALYSIS>analyzing</ANALYSIS>result");

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "analyzing" });
      expect(result[1]).toEqual({ type: "content", text: "result" });
    });

    it("should strip orphaned uppercase closing tags", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push("Content</THINK>more</Think>");

      expect(result).toHaveLength(1);
      expect(result[0].type).toBe("content");
      expect(result[0].text).not.toContain("</THINK>");
      expect(result[0].text).not.toContain("</Think>");
    });

    it("should handle nested tags with mixed case", () => {
      const parser = new ThinkingStreamParser();
      const result = parser.push(
        "<think>outer <THINK>inner</THINK> still</think>done",
      );

      expect(result).toHaveLength(2);
      expect(result[0].type).toBe("thinking");
      expect(result[0].text).toContain("outer");
      expect(result[0].text).toContain("inner");
      expect(result[1]).toEqual({ type: "content", text: "done" });
    });

    it("should handle startInThinking with uppercase closing tag", () => {
      const parser = new ThinkingStreamParser({ startInThinking: true });
      const result = parser.push("thinking content</THINK>visible answer");

      expect(result).toHaveLength(2);
      expect(result[0]).toEqual({ type: "thinking", text: "thinking content" });
      expect(result[1]).toEqual({ type: "content", text: "visible answer" });
    });

    it("should buffer partial uppercase tags correctly", () => {
      const parser = new ThinkingStreamParser();
      const chunk1 = parser.push("Start <TH");
      const chunk2 = parser.push("INK>reasoning</THINK>answer");

      expect(chunk1).toHaveLength(1);
      expect(chunk1[0]).toEqual({ type: "content", text: "Start " });
      expect(chunk2).toHaveLength(2);
      expect(chunk2[0]).toEqual({ type: "thinking", text: "reasoning" });
      expect(chunk2[1]).toEqual({ type: "content", text: "answer" });
    });
  });
});
