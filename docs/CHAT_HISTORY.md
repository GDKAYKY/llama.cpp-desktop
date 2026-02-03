# Chat History & Context Retrieval

## Overview

The Llama Desktop application features a persistent chat history and a hybrid context retrieval system. This allows users to save conversations, search through past messages, and provide the AI with relevant long-term memory context.

## 1. Persistent Storage (IndexedDB)

All chat conversations and messages are stored locally in the user's browser (webview) using **IndexedDB**, managed via the **Dexie.js** library.

### Database Schema

The database `LlamaDesktopDB` currently has two main tables:

- **`conversations`**:
  - `id`: Auto-incrementing primary key.
  - `title`: The title of the conversation (auto-generated or user-defined).
  - `updatedAt`: Timestamp of the last message or title change.
  - *Indices*: `++id`, `title`, `updatedAt`.

- **`messages`**:
  - `id`: Auto-incrementing primary key.
  - `conversationId`: Foreign key to the conversation.
  - `role`: `'user'`, `'assistant'`, or `'system'`.
  - `content`: The text content of the message.
  - `tokens`: Estimated token count.
  - `keywords`: Array of extracted keywords for search.
  - `timestamp`: Creation timestamp.
  - `model`: (Optional) The model ID that generated the response.
  - *Indices*: `++id`, `conversationId`, `role`, `*keywords` (multi-entry), `timestamp`, `model`.

## 2. Hybrid Context Retrieval

To enhance the AI's memory beyond the immediate conversation window, the app implements a **Hybrid Context Retrieval** system.

### Keyword Extraction
When a message is saved, it is processed to extract keywords:
1. Converting to lowercase.
2. Removing punctuation.
3. Splitting into words.
4. Removing common "stop words" (e.g., "the", "is", "at").
5. Filtering for words longer than 2 characters.

### Search & Scoring Logic
When a new query is sent, the system searches for relevant context:
1. **Initial Search**: Finds messages containing any of the query's keywords using Dexie's multi-entry index.
2. **Scoring**:
   - **Keyword Match**: +1.0 per matching keyword.
   - **Role Preference**: +0.5 for user messages (more likely to contain questions/definitions).
   - **Recency**: Future versions may include a decay factor for older messages.
3. **Filtering**:
   - Minimum score threshold (currently 1.0).
   - Excludes the current conversation to avoid immediate redundancy (which is handled by the backend session).
4. **Selection**: Top 5 best matches fitting within a token budget (default 2000 tokens).

## 3. Implementation Details

- **Location**: `src/lib/services/history.ts`
- **Key Functions**:
  - `saveMessage(...)`: Persists a message and updates the conversation timestamp.
  - `findRelevantContext(...)`: Performs the hybrid search.
  - `deleteConversation(...)`: Deletes a conversation and all its associated messages in a single transaction.

## 4. UI Integration

- **Sidebar**: Displays a list of recent conversations sorted by `updatedAt`.
- **Chat Interface**: Automatically loads the last 50 messages of a selected conversation.
- **Auto-Title**: (Planned/Partial) Generates a title based on the first few exchanges.

---
*Last updated: 2026-02-03*
