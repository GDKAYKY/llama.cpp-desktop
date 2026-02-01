import Dexie, { type Table } from 'dexie';

export interface Conversation {
  id?: number;
  title: string;
  updatedAt: number;
}

export interface ChatMessage {
  id?: number;
  conversationId: number;
  role: 'user' | 'assistant' | 'system';
  content: string;
  tokens: number;
  keywords: string[];
  timestamp: number;
}

export class ChatDatabase extends Dexie {
  conversations!: Table<Conversation>;
  messages!: Table<ChatMessage>;

  constructor() {
    super('LlamaDesktopDB');
    this.version(1).stores({
      conversations: '++id, title, updatedAt',
      messages: '++id, conversationId, role, *keywords, timestamp'
    });
  }
}

export const db = new ChatDatabase();

// --- Helper: Keyword Extraction (Simple Stopword removal) ---
const STOP_WORDS = new Set([
  'a', 'an', 'the', 'in', 'on', 'at', 'to', 'for', 'of', 'with', 'by',
  'is', 'are', 'was', 'were', 'be', 'been', 'being',
  'it', 'this', 'that', 'these', 'those',
  'i', 'you', 'he', 'she', 'we', 'they', 'me', 'him', 'her', 'us', 'them',
  'what', 'which', 'who', 'whom', 'whose',
  'can', 'could', 'will', 'would', 'shall', 'should',
  'have', 'has', 'had', 'do', 'does', 'did',
  'but', 'and', 'or', 'so', 'if', 'when', 'where', 'how', 'why'
]);

export function extractKeywords(text: string): string[] {
  return text
    .toLowerCase()
    .replace(/[^\w\s]/g, '') // Remove punctuation
    .split(/\s+/)
    .filter(word => word.length > 2 && !STOP_WORDS.has(word));
}

export function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

// --- Service Logic ---

export async function saveMessage(conversationId: number, role: 'user' | 'assistant' | 'system', content: string) {
  await db.messages.add({
    conversationId,
    role,
    content,
    tokens: estimateTokens(content),
    keywords: extractKeywords(content),
    timestamp: Date.now()
  });
  
  await db.conversations.update(conversationId, { updatedAt: Date.now() });
}

export async function createConversation(title: string = 'New Chat'): Promise<number> {
  return await db.conversations.add({
    title,
    updatedAt: Date.now()
  });
}

export async function getConversationHistory(conversationId: number): Promise<ChatMessage[]> {
    return await db.messages.where('conversationId').equals(conversationId).sortBy('timestamp');
}

export async function updateConversationTitle(conversationId: number, title: string) {
    await db.conversations.update(conversationId, { title });
}

export async function deleteConversation(conversationId: number) {
    await db.transaction('rw', db.conversations, db.messages, async () => {
        await db.messages.where('conversationId').equals(conversationId).delete();
        await db.conversations.delete(conversationId);
    });
}

export async function getRecentConversations(limit: number = 20): Promise<Conversation[]> {
    // Dexie doesn't support complex sorting easily without hooks or compound indices for everything.
    // 'conversations' is indexed by ++id, title, updatedAt.
    // We want desc order of updatedAt.
    return await db.conversations.orderBy('updatedAt').reverse().limit(limit).toArray();
}

export interface ContextResult {
    message: ChatMessage;
    score: number;
}

export async function findRelevantContext(query: string, currentConversationId: number, limitTokens: number = 2000): Promise<string> {
   const queryKeywords = extractKeywords(query);
   if (queryKeywords.length === 0) return '';

   // 1. Fetch all candidate messages (optimization: limit to recent 1000 messages or use a query limit if DB grows large)
   // For now, we fetch all user/assistant messages to scan. Indexing 'keywords' helps if we use multi-entry, 
   // but purely scoring in JS is more flexible for "hybrid" logic.
   // We can use db.messages.where('keywords').anyOf(queryKeywords) to filter first.
   
   const candidates = await db.messages
    .where('keywords')
    .anyOf(queryKeywords)
    .distinct() 
    .toArray();
    
   // 2. Score candidates
   const scored: ContextResult[] = candidates
    .filter(msg => {
        // Exclude current conversation to avoid repeating immediate context (optional, but usually immediate context is already in the prompt)
        // Let's exclude ONLY the very last few messages if they are from the SAME conversation to avoid duplication?
        // Actually, preventing the *current* conversation from being in "long term memory" context is safe if the backend handles immediate session history.
        return msg.conversationId !== currentConversationId;
    })
    .map(msg => {
       let score = 0;
       
       // Keyword Match Score
       const matchCount = msg.keywords.filter(k => queryKeywords.includes(k)).length;
       score += matchCount * 1.0;
       
       // Recency Score (Global, based on ID or timestamp)
       // This is rough. Assume higher ID = newer.
       // Normalize ID score? Let's just give a small bump for very recent messages.
       // Simple approach: Decay score based on age? 
       // For now: Just keyword density.
       
       // Role Score
       if (msg.role === 'user') score += 0.5; // Slightly prefer user queries
       
       return { message: msg, score };
    });
    
   // 3. Sort by score desc
   scored.sort((a, b) => b.score - a.score);
   
   // 4. Select top unique messages fitting token budget
   const selected: ChatMessage[] = [];
   let currentTokens = 0;
   
   // formatting cost
   const formatCost = 10; 
   
   for (const item of scored) {
       if (item.score < 1) continue; // Minimum relevance threshold
       
       if (currentTokens + item.message.tokens + formatCost > limitTokens) break;
       
       selected.push(item.message);
       currentTokens += item.message.tokens + formatCost;
       
       if (selected.length >= 5) break; // Hard limit on count
   }
   
   // 5. Format output
   if (selected.length === 0) return '';
   
   // Sort selected by timestamp asc to make sense reading them
   selected.sort((a, b) => a.timestamp - b.timestamp);
   
   return selected.map(m => `[${m.role.toUpperCase()}]: ${m.content}`).join('\n');
}
