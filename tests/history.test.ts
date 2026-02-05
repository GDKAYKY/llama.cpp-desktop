import { describe, it, expect, beforeEach } from 'vitest';
import { db, createConversation, saveMessage, getConversationHistory, type ChatMessage } from '../src/lib/services/history';

describe('Chat History Service (IndexedDB)', () => {
  // Clear the database before each test to ensure a clean state
  beforeEach(async () => {
    await db.delete();
    await db.open();
  });

  it('should create a conversation and save messages correctly', async () => {
    // 1. Create a new conversation
    const conversationId = await createConversation('Test Chat');
    expect(conversationId).toBeDefined();
    expect(typeof conversationId).toBe('number');

    // 2. Save a user message
    const userContent = 'Hello, AI!';
    await saveMessage(conversationId, 'user', userContent);

    // 3. Save an assistant message
    const assistantContent = 'Hello! How can I help you today?';
    await saveMessage(conversationId, 'assistant', assistantContent, 'llama-3');

    // 4. Retrieve history
    const history = await getConversationHistory(conversationId);

    // 5. Verify conversation history
    expect(history).toHaveLength(2);

    const [msg1, msg2] = history;

    // Verify first message (User)
    expect(msg1.role).toBe('user');
    expect(msg1.content).toBe(userContent);
    expect(msg1.conversationId).toBe(conversationId);
    expect(msg1.tokens).toBeGreaterThan(0);
    expect(msg1.timestamp).toBeDefined();

    // Verify second message (Assistant)
    expect(msg2.role).toBe('assistant');
    expect(msg2.content).toBe(assistantContent);
    expect(msg2.model).toBe('llama-3');
    expect(msg2.conversationId).toBe(conversationId);
  });

  it('should maintain message order based on timestamp', async () => {
    const conversationId = await createConversation('Ordered Chat');

    await saveMessage(conversationId, 'user', 'First');
    // Small delay to ensure timestamp difference if necessary, 
    // though Date.now() usually changes or IndexedDB auto-increment helps stability?
    // The implementation uses sorting by timestamp.
    await new Promise(r => setTimeout(r, 10)); 
    await saveMessage(conversationId, 'assistant', 'Second');

    const history = await getConversationHistory(conversationId);
    
    expect(history[0].content).toBe('First');
    expect(history[1].content).toBe('Second');
  });
});
