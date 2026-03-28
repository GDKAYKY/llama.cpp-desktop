# llama.cpp — **All Possible JSON Parameters** for `/v1/chat/completions`

**Updated: March 2026**  
**llama-server** (OpenAI-compatible endpoint) + full llama.cpp extensions.

The endpoint accepts **every** parameter from the OpenAI Chat Completions spec **plus** all llama.cpp sampling params, reasoning/thinking controls, Jinja template options, and more.

Copy-paste this file as `llama-cpp-chat-completions-parameters.md`.

---

## 1. Required Parameters

| Parameter  | Type   | Description                                                                                                                                                                                                                                                 |
| ---------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `model`    | string | Model identifier (GGUF path, alias set with `--alias`, or any string)                                                                                                                                                                                       |
| `messages` | array  | Array of message objects: `{"role": "system\|user\|assistant\|tool", "content": string or array}`<br>• Multimodal: `content` can be array with `{"type": "text", "text": "..."}` + `{"type": "image_url", "image_url": {"url": "data:... or https://..."}}` |

---

## 2. Standard OpenAI-compatible Parameters

| Parameter                  | Type             | Default  | Description                                                                |
| -------------------------- | ---------------- | -------- | -------------------------------------------------------------------------- |
| `max_tokens` / `n_predict` | integer          | `-1`     | Max tokens to generate (`-1` = unlimited)                                  |
| `temperature`              | number           | `0.8`    | 0.0–2.0                                                                    |
| `top_p`                    | number           | `0.95`   | Nucleus sampling                                                           |
| `top_k`                    | integer          | `40`     | 0 = disabled                                                               |
| `min_p`                    | number           | `0.05`   | Min-P sampling                                                             |
| `stream`                   | boolean          | `false`  | Server-Sent Events streaming                                               |
| `stop`                     | string or array  | `null`   | Stop sequences                                                             |
| `presence_penalty`         | number           | `0.0`    | -2.0…2.0                                                                   |
| `frequency_penalty`        | number           | `0.0`    | -2.0…2.0                                                                   |
| `repeat_penalty`           | number           | `1.1`    | Repetition penalty                                                         |
| `seed`                     | integer          | `-1`     | `-1` = random                                                              |
| `response_format`          | object           | —        | `{"type": "json_object"}` or `{"type": "json_schema", "schema": {...}}`    |
| `tools`                    | array            | —        | Tool/function definitions (requires `--jinja`)                             |
| `tool_choice`              | string or object | `"auto"` | `"none"`, `"auto"`, or `{"type": "function", "function": {"name": "..."}}` |
| `n`                        | integer          | `1`      | Number of completions (rarely used in chat)                                |

---

## 3. llama.cpp Sampling Parameters (all work in chat/completions)

| Parameter               | Type             | Default         | Description                                                                           |
| ----------------------- | ---------------- | --------------- | ------------------------------------------------------------------------------------- |
| `typical_p`             | number           | `1.0`           | Typical sampling (1.0 = disabled)                                                     |
| `tfs_z`                 | number           | `1.0`           | Tail-free sampling                                                                    |
| `mirostat`              | integer          | `0`             | 0=off, 1=Mirostat, 2=Mirostat 2.0                                                     |
| `mirostat_tau`          | number           | `5.0`           | Mirostat target entropy                                                               |
| `mirostat_eta`          | number           | `0.1`           | Mirostat learning rate                                                                |
| `dynatemp_range`        | number           | `0.0`           | Dynamic temperature range                                                             |
| `dynatemp_exponent`     | number           | `1.0`           | Dynamic temperature exponent                                                          |
| `xtc_probability`       | number           | `0.0`           | XTC probability                                                                       |
| `xtc_threshold`         | number           | `0.1`           | XTC threshold                                                                         |
| `top_n_sigma`           | number           | `-1.0`          | Top-n sigma                                                                           |
| `dry_multiplier`        | number           | `0.0`           | DRY sampler multiplier                                                                |
| `dry_base`              | number           | `1.75`          | DRY base                                                                              |
| `dry_allowed_length`    | integer          | `2`             | DRY allowed length                                                                    |
| `dry_penalty_last_n`    | integer          | `-1`            | DRY penalty last n (`-1` = all)                                                       |
| `dry_sequence_breakers` | array of strings | `[]`            | DRY sequence breakers                                                                 |
| `samplers`              | string           | (default order) | Sampler order string (e.g. `"penalties;dry;top_k;typ_p;top_p;min_p;xtc;temperature"`) |
| `grammar`               | string           | —               | Full GBNF grammar                                                                     |
| `json_schema`           | object           | —               | Alternative to `response_format` (auto-generates grammar)                             |
| `repeat_last_n`         | integer          | `64`            | Tokens considered for repeat penalty                                                  |
| `logprobs` / `n_probs`  | integer          | —               | Return logprobs for top N tokens                                                      |

---

## 4. **Reasoning / Thinking Parameters** (new in 2025–2026)

| Parameter                  | Type    | Default  | Description                                                                                                                                                  |
| -------------------------- | ------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `reasoning_format`         | string  | `"auto"` | Controls thought tags extraction:<br>• `"none"` → thoughts stay in `content`<br>• `"deepseek"` → thoughts go to `reasoning_content`<br>• `"deepseek-legacy"` |
| `reasoning_budget`         | integer | `-1`     | **Token budget for thinking**<br>• `-1` = unrestricted (default)<br>• `0` = disable thinking completely<br>• `N > 0` = max N thinking tokens                 |
| `reasoning_budget_message` | string  | `null`   | Message injected **before** the end-of-thinking tag when budget is exhausted                                                                                 |
| `thinking_forced_open`     | boolean | `false`  | Force reasoning models to always output thinking tags                                                                                                        |
| `chat_template_kwargs`     | object  | `{}`     | Extra kwargs passed to Jinja template, e.g. `{"enable_thinking": false}`                                                                                     |

> **Note**: When `reasoning_format: "deepseek"`, the response object will contain `reasoning_content` (separate from `content`).

---

## 5. Chat-Specific & Advanced Parameters

| Parameter             | Type    | Default | Description                                                 |
| --------------------- | ------- | ------- | ----------------------------------------------------------- |
| `cache_prompt`        | boolean | `true`  | Reuse KV cache (huge speed-up for multi-turn)               |
| `generation_prompt`   | string  | —       | Prefill the generation prompt after template                |
| `parse_tool_calls`    | boolean | `true`  | Automatically parse tool calls from output                  |
| `parallel_tool_calls` | boolean | `false` | Allow multiple tool calls in one response                   |
| `id_slot`             | integer | —       | Force request into a specific model slot (multi-model mode) |

---

## 6. Rarely Used / Legacy Parameters (still accepted)

- `echo` (boolean)
- `suffix` (string)
- `penalize_nl` (boolean)
- `logit_bias` (object)

---

## Complete Real-World Example JSON

```json
{
  "model": "deepseek-r1-distill-llama-8b",
  "messages": [
    { "role": "system", "content": "You are a helpful assistant." },
    { "role": "user", "content": "Explain how a black hole works" }
  ],
  "temperature": 0.7,
  "top_p": 0.95,
  "min_p": 0.05,
  "repeat_penalty": 1.1,
  "max_tokens": 2048,
  "seed": 12345,
  "cache_prompt": true,
  "reasoning_format": "deepseek",
  "reasoning_budget": 1200,
  "reasoning_budget_message": "Budget exhausted - final answer follows.",
  "chat_template_kwargs": { "enable_thinking": true },
  "stop": ["<|end|>"],
  "stream": false
}
```
