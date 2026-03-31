use crate::common;

use llama_desktop_lib::services::thinking_parser::{ThinkingStreamParser, ParsedChunk};

#[test]
fn test_parser_multiple_thinking_blocks() {
    let mut parser = ThinkingStreamParser::new();
    let mut all = Vec::new();
    
    all.extend(parser.push("Start <think>first</think> middle <think>second</think> end"));
    all.extend(parser.flush());
    
    assert_eq!(all, vec![
        ParsedChunk::Content("Start ".into()),
        ParsedChunk::Thinking("first".into()),
        ParsedChunk::Content(" middle ".into()),
        ParsedChunk::Thinking("second".into()),
        ParsedChunk::Content(" end".into()),
    ]);
}

#[test]
fn test_parser_analysis_tag() {
    let mut parser = ThinkingStreamParser::new();
    let mut all = Vec::new();
    
    all.extend(parser.push("<analysis>analyzing data</analysis>result"));
    all.extend(parser.flush());
    
    assert_eq!(all, vec![
        ParsedChunk::Thinking("analyzing data".into()),
        ParsedChunk::Content("result".into()),
    ]);
}

#[test]
fn test_parser_reasoning_tag() {
    let mut parser = ThinkingStreamParser::new();
    let mut all = Vec::new();
    
    all.extend(parser.push("<reasoning>logic here</reasoning>conclusion"));
    all.extend(parser.flush());
    
    assert_eq!(all, vec![
        ParsedChunk::Thinking("logic here".into()),
        ParsedChunk::Content("conclusion".into()),
    ]);
}

#[test]
fn test_parser_nested_tags_not_supported() {
    let mut parser = ThinkingStreamParser::new();
    let mut all = Vec::new();
    
    all.extend(parser.push("<think>outer <think>inner</think> outer</think>"));
    all.extend(parser.flush());
    
    // Parser treats first </think> as closing tag
    assert!(all.len() >= 2);
}

#[test]
fn test_parser_empty_thinking_block() {
    let mut parser = ThinkingStreamParser::new();
    let mut all = Vec::new();
    
    all.extend(parser.push("<think></think>content"));
    all.extend(parser.flush());
    
    assert_eq!(all, vec![ParsedChunk::Content("content".into())]);
}

#[test]
fn test_parser_only_thinking() {
    let mut parser = ThinkingStreamParser::new();
    let mut all = Vec::new();
    
    all.extend(parser.push("<think>only thinking</think>"));
    all.extend(parser.flush());
    
    assert_eq!(all, vec![ParsedChunk::Thinking("only thinking".into())]);
}

#[test]
fn test_parser_chunk_by_chunk() {
    let mut parser = ThinkingStreamParser::new();
    let mut all = Vec::new();
    
    all.extend(parser.push("H"));
    all.extend(parser.push("e"));
    all.extend(parser.push("l"));
    all.extend(parser.push("l"));
    all.extend(parser.push("o"));
    all.extend(parser.flush());
    
    assert_eq!(all, vec![
        ParsedChunk::Content("H".into()),
        ParsedChunk::Content("e".into()),
        ParsedChunk::Content("l".into()),
        ParsedChunk::Content("l".into()),
        ParsedChunk::Content("o".into()),
    ]);
}
