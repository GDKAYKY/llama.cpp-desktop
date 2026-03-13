# Subagent Implementation Checklist

## ✅ Implementation Steps

### Phase 1: Add Subagent Module

- [ ] Copy `subagent.rs` into your `src/services/` directory
- [ ] Add `pub mod subagent;` to `src/services/mod.rs`
- [ ] Verify imports compile

### Phase 2: Update Models

- [ ] Add two new fields to `IntentClassification` struct:
  ```rust
  pub needs_multi_step: bool,
  pub multi_step_reasoning: Option<String>,
  ```
- [ ] Update intent parsing to handle new fields (defaults to `false` / `None`)

### Phase 3: Modify ChatOrchestrator

- [ ] Import subagent types at top of `orchestrator.rs`
- [ ] Add `subagent: Subagent` field to `ChatOrchestrator` struct
- [ ] Initialize subagent in constructor (see integration_guide.rs)
- [ ] Update `classify_intent()` system prompt to include multi-step detection
- [ ] Add multi-step check in `process()` method
- [ ] Implement `answer_with_formatted_data()` method

### Phase 4: Test

- [ ] Test single-tool queries (should work as before)
- [ ] Test multi-step queries (weather comparison, etc.)
- [ ] Test edge cases:
  - Max iterations exceeded
  - Subagent fails midway (fallback works?)
  - Invalid tool names
  - Empty results

### Phase 5: Optional Enhancements

- [ ] Add subagent progress events to `on_event` channel
- [ ] Tune `MAX_SUBAGENT_ITERATIONS` for your use case
- [ ] Add metrics/logging for subagent performance
- [ ] Implement caching for repeated tool calls
- [ ] Add user-facing "thinking" indicators

## 🔧 Configuration Options

```rust
// In subagent.rs, tune these constants:

const MAX_SUBAGENT_ITERATIONS: usize = 5;  // Increase for complex tasks
const SUBAGENT_MAX_TOKENS: i32 = 512;      // Increase for verbose LLMs
```

## 🧪 Test Queries

### Should trigger single-tool:

- "What's the weather in Tokyo?"
- "Search for AI news"
- "Get my calendar events"

### Should trigger subagent:

- "Compare weather in Tokyo, NYC, and London"
- "Find news about AI, then papers on those topics"
- "Get weather and calendar, then suggest activities"
- "Search for restaurants near me, then get reviews for top 3"

## 🚀 Performance Tips

1. **Token Budget**: Subagent uses ephemeral sessions (no KV-cache pollution)
2. **Parallel Tools**: For truly independent calls, consider adding parallel execution
3. **Caching**: Cache tool results within a session to avoid redundant calls
4. **Streaming**: Current impl doesn't stream subagent thinking (by design)

## 🐛 Common Issues

### Issue: Subagent always returns "finish" on first iteration

**Fix**: Check your system prompt. Make sure it's clear that the subagent should gather ALL needed data before finishing.

### Issue: Intent classifier never sets `needs_multi_step: true`

**Fix**: Your LLM might be too conservative. Try:

- Adding examples to the system prompt
- Lowering temperature for classification
- Using a more capable model for intent classification

### Issue: Context window overflow

**Fix**: The `truncate_to_token_budget()` function should handle this, but you may need to:

- Increase context size in llama.cpp config
- Reduce `SUBAGENT_MAX_TOKENS`
- Truncate tool results more aggressively

### Issue: Subagent gets stuck in loops

**Fix**: This shouldn't happen due to `MAX_SUBAGENT_ITERATIONS`, but if the LLM keeps calling the same tool:

- Improve the system prompt to encourage finishing
- Add explicit loop detection in `execute()`
- Log tool call history and detect patterns

## 📊 Monitoring

Add these events to track subagent behavior:

```rust
// In subagent.execute():
on_event.send(json!({
    "subagent": {
        "iteration": iteration,
        "action": action.action_type,
        "tool": action.tool_name,
        "reasoning": action.reasoning
    }
}));
```

## 🎯 Success Criteria

Your implementation is working well if:

- ✅ Single-tool queries complete in <3 seconds
- ✅ Multi-step queries complete in <10 seconds
- ✅ No context window overflows
- ✅ Subagent uses ≤ 3 iterations for typical queries
- ✅ Fallback to plain LLM works when tools unavailable
- ✅ Results are coherent and well-synthesized

## 📝 Next Steps

After basic implementation works:

1. Add user preferences for subagent behavior
2. Implement parallel tool execution for independent calls
3. Add cost/latency tracking
4. Consider streaming subagent progress to user
5. Build a UI for visualizing tool call chains

FILE 1: ./subagent.rs
