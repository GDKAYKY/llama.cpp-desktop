# Chat History & Context Retrieval

## Overview

The Llama Desktop application features a persistent chat history and hybrid context retrieval system. All conversations are stored locally in the browser's IndexedDB, enabling users to save conversations, search through past messages, and provide the AI with relevant long-term memory context.

## Architecture

```
┌─────────────────┐
│   Chat UI       │
│  (Svelte)       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  history.ts     │ ← Main service layer
│  (Frontend)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Dexie.js      │ ← IndexedDB wrapper
│  (IndexedDB)    │
└─────────────────┘
```

## 1. Persistent Storage (IndexedDB)

**Technology**: IndexedDB via **Dexie.js** library  
**Location**: `src/lib/services/history.ts`  
**Database**: `LlamaDesktopDB`

### Database Schema

#### `conversations` Table
| Field | Type | Description |
|-------|------|-------------|
| `id` | number (PK) | Auto-incrementing primary key |
| `title` | string | Conversation title (auto-generated or user-defined) |
| `updatedAt` | number | Timestamp of last message or title change |

**Indices**: `++id`, `title`, `updatedAt`

#### `messages` Table
| Field | Type | Description |
|-------|------|-------------|
| `id` | number (PK) | Auto-incrementing primary key |
| `conversationId` | number (FK) | References conversation |
| `role` | string | `'user'`, `'assistant'`, or `'system'` |
| `content` | string | Message text content |
| `tokens` | number | Estimated token count |
| `keywords` | string[] | Extracted keywords for search |
| `timestamp` | number | Creation timestamp (Date.now()) |
| `model` | string? | Optional model ID that generated response |

**Indices**: `++id`, `conversationId`, `role`, `*keywords` (multi-entry), `timestamp`, `model`

## 2. Hybrid Context Retrieval

The hybrid context retrieval system provides the AI with relevant long-term memory beyond the immediate conversation window.

### Keyword Extraction Pipeline

When a message is saved, keywords are extracted using this process:

1. **Normalize**: Convert to lowercase
2. **Clean**: Remove punctuation
3. **Tokenize**: Split into words
4. **Filter Stop Words**: Remove common words ("the", "is", "at", "and", etc.)
5. **Length Filter**: Keep only words > 2 characters
6. **Store**: Save keywords array in `messages.keywords` field

**Implementation**: `extractKeywords(content: string): string[]`

### Search & Scoring Algorithm

When a new query is sent, the system searches for relevant context:

#### 1. Initial Search
```typescript
db.messages
  .where('keywords')
  .anyOf(queryKeywords)
  .distinct()
  .toArray()
```
Uses Dexie's multi-entry index on `keywords` for efficient lookup.

#### 2. Scoring System
Each candidate message receives a score based on:

| Factor | Weight | Description |
|--------|--------|-------------|
| **Keyword Match** | +1.0 per match | Number of matching keywords |
| **Role Preference** | +0.5 | Bonus for user messages (contain questions/definitions) |
| **Conversation Filter** | Exclude | Excludes current conversation (handled by backend session) |

**Minimum Score Threshold**: 1.0

#### 3. Selection Criteria
- **Token Budget**: Default 2000 tokens
- **Count Limit**: Maximum 5 messages
- **Formatting Cost**: 10 tokens per message overhead
- **Sort Order**: By timestamp (ascending) for chronological context

#### 4. Output Format
```
[USER]: <message content>
[ASSISTANT]: <message content>
...
```

## 3. Key Functions

### `saveMessage()`
```typescript
async function saveMessage(
  conversationId: number,
  role: 'user' | 'assistant' | 'system',
  content: string,
  model?: string
)
```
- Persists message to IndexedDB
- Extracts and stores keywords
- Estimates token count
- Updates conversation `updatedAt` timestamp

### `findRelevantContext()`
```typescript
async function findRelevantContext(
  query: string,
  currentConversationId: number,
  limitTokens: number = 2000
): Promise<string>
```
- Performs hybrid search across all conversations
- Scores and ranks messages by relevance
- Returns formatted context string within token budget
- Returns empty string if no relevant context found

### `deleteConversation()`
```typescript
async function deleteConversation(conversationId: number)
```
- Deletes conversation and all associated messages
- Uses Dexie transaction for atomicity
- Cascades deletion to prevent orphaned messages

### Other Functions
- `createConversation(title?: string)` - Create new conversation
- `getConversation(id: number)` - Fetch conversation by ID
- `getMessages(conversationId: number, limit?: number)` - Load messages (default: last 50)
- `updateConversationTitle(id: number, title: string)` - Update conversation title
- `estimateTokens(text: string)` - Rough token estimation (words * 1.3)

## 4. UI Integration

### Chat Sidebar (`ChatSidebar.svelte`)
- Displays list of recent conversations
- Sorted by `updatedAt` (most recent first)
- Shows conversation title and last update time
- Click to load conversation

### Chat Interface (`Home.svelte`)
- Automatically loads last 50 messages of selected conversation
- Streams new messages in real-time
- Auto-saves messages to history
- Updates conversation timestamp on each message

### Auto-Title Generation
- Generates title from first user message
- Truncates to 50 characters
- Falls back to "New Chat" if empty

## 5. Performance Considerations

### Indexing Strategy
- Multi-entry index on `keywords` enables fast keyword lookups
- Compound indices on `conversationId` + `timestamp` for efficient message loading
- `updatedAt` index for fast conversation list sorting

### Optimization Techniques
- **Distinct Results**: Prevents duplicate messages in search results
- **Token Budget**: Limits context size to prevent prompt overflow
- **Count Limit**: Hard cap of 5 messages prevents excessive context
- **Lazy Loading**: Loads only last 50 messages per conversation

### Scalability
- IndexedDB handles thousands of messages efficiently
- Keyword-based search scales well with proper indexing
- Consider pagination for conversations with 1000+ messages

## 6. Future Enhancements

- [ ] **Recency Decay**: Add time-based score decay for older messages
- [ ] **Semantic Search**: Integrate vector embeddings for better relevance
- [ ] **Conversation Folders**: Organize conversations into categories
- [ ] **Export/Import**: Backup and restore conversation history
- [ ] **Full-Text Search**: Add search UI for finding specific messages
- [ ] **Message Editing**: Allow editing past messages
- [ ] **Conversation Merging**: Combine related conversations

---

*Last updated: 2026-03-28*
