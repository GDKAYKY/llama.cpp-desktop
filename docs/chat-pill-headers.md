# Chat Pill Headers

## Purpose

The pill-header feature adds visual window-style framing to assistant messages. Each message gets a rounded header strip (`rounded-t-lg`) and an independently scrollable body, creating a card-like appearance.

## Architecture

```
ChatMessages (parent list, overflow-y-auto)
  └─ ChatMessage (per-message logic)
       └─ ChatMessageWindow (wrapper, when pill=true)
            ├─ Header (rounded-t-lg pill strip)
            └─ Body (overflow-hidden → overflow-y-auto, max-h-48)
```

### Component Responsibilities

| Component | Role |
|---|---|
| `ChatMessages.svelte` | Scrollable message list. Forwards `pill` prop (default: `true`). |
| `ChatMessage.svelte` | Per-message rendering, editing, actions. Accepts `pill` prop. |
| `ChatMessageWindow.svelte` | Generic wrapper. Applies pill styling, isolation, and scroll rules. |

## Props

### `ChatMessageWindow`

| Prop | Type | Default | Description |
|---|---|---|---|
| `pill` | `boolean` | `false` | Applies `rounded-t-lg` on the header element. |
| `header` | `Snippet` | required | Content rendered in the header strip. |
| `body` | `Snippet` | required | Content rendered in the scrollable body. |
| `class` | `string` | `undefined` | Additional CSS classes on the outer wrapper. |

### `ChatMessage`

| Prop | Type | Default | Description |
|---|---|---|---|
| `pill` | `boolean` | `false` | Forwarded to `ChatMessageWindow` for assistant messages. |

### `ChatMessages`

| Prop | Type | Default | Description |
|---|---|---|---|
| `pill` | `boolean` | `true` | Forwarded to every `ChatMessage` in the list. |

## CSS Rules (Mandatory)

These five rules prevent visual rendering issues:

1. **No `overflow-hidden` on the outer wrapper** — it clips `rounded-t-lg` during scroll.
2. **`rounded-t-lg` directly on the header element** — `border-radius` does not inherit from parent.
3. **`isolation: isolate`** on each child window — creates independent stacking context.
4. **`overflow-y-auto` + `max-h-48`** inside the body only — independent internal scroll.
5. **Parent container uses `overflow-y-auto` normally** — safe as long as rules 1–4 are followed.

## Usage

```svelte
<!-- Pill enabled (default in ChatMessages) -->
<ChatMessages messages={msgs} isLoading={false} pill={true} />

<!-- Pill disabled (falls back to original flat layout) -->
<ChatMessages messages={msgs} isLoading={false} pill={false} />
```
