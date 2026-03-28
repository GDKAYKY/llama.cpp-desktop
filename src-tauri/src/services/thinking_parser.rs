/// Incremental parser that separates `<think>…</think>` reasoning blocks
/// from the main response content while streaming token-by-token.
///
/// Handles the critical edge-case of tags being split across multiple chunks
/// (e.g. `<thi` in one chunk, `nk>` in the next) by buffering potential
/// partial tags.

const TAGS: [(&str, &str); 3] = [
    ("<think>", "</think>"),
    ("<analysis>", "</analysis>"),
    ("<reasoning>", "</reasoning>"),
];

/// Emitted events from the parser on each `push` call.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedChunk {
    /// Regular response content — display immediately.
    Content(String),
    /// Reasoning content — route to the thinking accordion.
    Thinking(String),
}

pub struct ThinkingStreamParser {
    in_think: bool,
    buffer: String,
    current_close_tag: Option<&'static str>,
}

impl ThinkingStreamParser {
    pub fn new() -> Self {
        Self {
            in_think: false,
            buffer: String::new(),
            current_close_tag: None,
        }
    }

    /// Feed a raw chunk from the LLM stream.
    /// Returns zero or more `ParsedChunk` items to emit to the frontend.
    pub fn push(&mut self, chunk: &str) -> Vec<ParsedChunk> {
        self.buffer.push_str(chunk);
        let mut results = Vec::new();

        loop {
            if self.in_think {
                let close_tag = self.current_close_tag.unwrap_or("</think>");
                match self.buffer.find(close_tag) {
                    Some(pos) => {
                        let thinking_text = self.buffer[..pos].to_string();
                        self.buffer = self.buffer[pos + close_tag.len()..].to_string();
                        self.in_think = false;
                        self.current_close_tag = None;
                        if !thinking_text.is_empty() {
                            results.push(ParsedChunk::Thinking(thinking_text));
                        }
                    }
                    None => {
                        // Check if buffer ends with a partial close tag
                        if self.could_be_partial_tag(&self.buffer.clone(), close_tag) {
                            break; // Wait for more data
                        }
                        // Safe to emit everything as thinking
                        let text = std::mem::take(&mut self.buffer);
                        if !text.is_empty() {
                            results.push(ParsedChunk::Thinking(text));
                        }
                        break;
                    }
                }
            } else {
                match self.find_next_open_tag(&self.buffer) {
                    Some((pos, open_len, close_tag)) => {
                        let content_text = self.buffer[..pos].to_string();
                        self.buffer = self.buffer[pos + open_len..].to_string();
                        self.in_think = true;
                        self.current_close_tag = Some(close_tag);
                        if !content_text.is_empty() {
                            results.push(ParsedChunk::Content(content_text));
                        }
                    }
                    None => {
                        // Check if buffer ends with a partial open tag
                        if TAGS
                            .iter()
                            .any(|(open, _)| self.could_be_partial_tag(&self.buffer.clone(), open))
                        {
                            break; // Wait for more data
                        }
                        // Safe to emit everything as content
                        let text = std::mem::take(&mut self.buffer);
                        if !text.is_empty() {
                            results.push(ParsedChunk::Content(text));
                        }
                        break;
                    }
                }
            }
        }

        results
    }

    /// Flush any remaining buffer content at end-of-stream.
    pub fn flush(&mut self) -> Vec<ParsedChunk> {
        let mut results = Vec::new();
        if !self.buffer.is_empty() {
            let text = std::mem::take(&mut self.buffer);
            if self.in_think {
                results.push(ParsedChunk::Thinking(text));
            } else {
                results.push(ParsedChunk::Content(text));
            }
        }
        results
    }

    /// Check if the end of `text` could be the start of a partial `tag`.
    /// Example: text ending with `<thi` could be start of `<think>`.
    fn could_be_partial_tag(&self, text: &str, tag: &str) -> bool {
        let check_len = tag.len().min(text.len());
        for i in 1..=check_len {
            if let Some(start) = text.len().checked_sub(i) {
                if text.is_char_boundary(start) {
                    let suffix = &text[start..];
                    if tag.starts_with(suffix) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn find_next_open_tag(&self, text: &str) -> Option<(usize, usize, &'static str)> {
        let mut best: Option<(usize, usize, &'static str)> = None;
        for (open, close) in TAGS.iter() {
            if let Some(pos) = text.find(open) {
                match best {
                    Some((best_pos, _, _)) if pos >= best_pos => {}
                    _ => best = Some((pos, open.len(), *close)),
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_no_thinking() {
        let mut parser = ThinkingStreamParser::new();
        let chunks = parser.push("Hello, world!");
        let flushed = parser.flush();
        let all: Vec<_> = chunks.into_iter().chain(flushed).collect();
        assert_eq!(all, vec![ParsedChunk::Content("Hello, world!".into())]);
    }

    #[test]
    fn complete_thinking_block() {
        let mut parser = ThinkingStreamParser::new();
        let mut all = Vec::new();
        all.extend(parser.push("<think>reasoning here</think>actual response"));
        all.extend(parser.flush());
        assert_eq!(
            all,
            vec![
                ParsedChunk::Thinking("reasoning here".into()),
                ParsedChunk::Content("actual response".into()),
            ]
        );
    }

    #[test]
    fn split_open_tag() {
        let mut parser = ThinkingStreamParser::new();
        let mut all = Vec::new();
        // Tag split across two chunks
        all.extend(parser.push("Hello <thi"));
        all.extend(parser.push("nk>deep thought</think>answer"));
        all.extend(parser.flush());
        assert_eq!(
            all,
            vec![
                ParsedChunk::Content("Hello ".into()),
                ParsedChunk::Thinking("deep thought".into()),
                ParsedChunk::Content("answer".into()),
            ]
        );
    }

    #[test]
    fn split_close_tag() {
        let mut parser = ThinkingStreamParser::new();
        let mut all = Vec::new();
        all.extend(parser.push("<think>reasoning</thi"));
        all.extend(parser.push("nk>response"));
        all.extend(parser.flush());
        assert_eq!(
            all,
            vec![
                ParsedChunk::Thinking("reasoning".into()),
                ParsedChunk::Content("response".into()),
            ]
        );
    }

    #[test]
    fn streaming_thinking_incomplete() {
        let mut parser = ThinkingStreamParser::new();
        let mut all = Vec::new();
        all.extend(parser.push("<think>step 1\n"));
        all.extend(parser.push("step 2\n"));
        // Stream ends without closing tag
        all.extend(parser.flush());
        assert_eq!(
            all,
            vec![
                ParsedChunk::Thinking("step 1\n".into()),
                ParsedChunk::Thinking("step 2\n".into()),
            ]
        );
    }

    #[test]
    fn angle_bracket_not_a_tag() {
        let mut parser = ThinkingStreamParser::new();
        let mut all = Vec::new();
        all.extend(parser.push("a < b and c > d"));
        all.extend(parser.flush());
        assert_eq!(all, vec![ParsedChunk::Content("a < b and c > d".into())]);
    }
}
