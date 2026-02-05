import { describe, it, expect, beforeEach } from 'vitest';
import { db, createConversation, saveMessage, getConversationHistory, updateConversationTitle, deleteConversation, getRecentConversations, extractKeywords, estimateTokens, findRelevantContext, type ChatMessage } from '../src/lib/services/history';

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
    await new Promise(r => setTimeout(r, 10)); 
    await saveMessage(conversationId, 'assistant', 'Second');

    const history = await getConversationHistory(conversationId);
    
    expect(history[0].content).toBe('First');
    expect(history[1].content).toBe('Second');
  });

  it('should update conversation title', async () => {
    const conversationId = await createConversation('Old Title');
    await updateConversationTitle(conversationId, 'New Title');
    
    const conversation = await db.conversations.get(conversationId);
    expect(conversation?.title).toBe('New Title');
  });

  it('should delete conversation and its messages', async () => {
    const conversationId = await createConversation('To Delete');
    await saveMessage(conversationId, 'user', 'Test message');
    
    await deleteConversation(conversationId);
    
    const conversation = await db.conversations.get(conversationId);
    const messages = await db.messages.where('conversationId').equals(conversationId).toArray();
    
    expect(conversation).toBeUndefined();
    expect(messages).toHaveLength(0);
  });

  it('should get recent conversations ordered by updatedAt', async () => {
    const id1 = await createConversation('First');
    await new Promise(r => setTimeout(r, 10));
    const id2 = await createConversation('Second');
    await new Promise(r => setTimeout(r, 10));
    const id3 = await createConversation('Third');
    
    const recent = await getRecentConversations(2);
    
    expect(recent).toHaveLength(2);
    expect(recent[0].id).toBe(id3);
    expect(recent[1].id).toBe(id2);
  });

  it('should extract keywords correctly', () => {
    const text = 'How can I train a machine learning model?';
    const keywords = extractKeywords(text);
    
    expect(keywords).toContain('train');
    expect(keywords).toContain('machine');
    expect(keywords).toContain('learning');
    expect(keywords).toContain('model');
    expect(keywords).not.toContain('can');
    expect(keywords).not.toContain('the');
  });

  it('should estimate tokens correctly', () => {
    const text = 'Hello world';
    const tokens = estimateTokens(text);
    expect(tokens).toBe(Math.ceil(text.length / 4));
  });

  it('should find relevant context from other conversations', async () => {
    const conv1 = await createConversation('Conv 1');
    await saveMessage(conv1, 'user', 'How do I train machine learning models?');
    await saveMessage(conv1, 'assistant', 'You can use Python libraries like TensorFlow.');
    
    const conv2 = await createConversation('Conv 2');
    await saveMessage(conv2, 'user', 'What is deep learning?');
    
    const context = await findRelevantContext('machine learning training', conv2, 2000);
    
    expect(context).toContain('train');
    expect(context).toContain('machine');
  });

  it('should return empty context when no relevant messages found', async () => {
    const conv1 = await createConversation('Conv 1');
    await saveMessage(conv1, 'user', 'Hello');
    
    const conv2 = await createConversation('Conv 2');
    const context = await findRelevantContext('completely unrelated query xyz', conv2, 2000);
    
    expect(context).toBe('');
  });
});
