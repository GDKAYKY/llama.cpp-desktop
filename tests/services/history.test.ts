import { describe, it, expect, beforeEach } from 'vitest';
import { extractKeywords, estimateTokens } from '$lib/services/history';
import 'fake-indexeddb/auto';

describe('history service', () => {
    describe('extractKeywords', () => {
        it('extracts keywords from text', () => {
            const text = 'The quick brown fox jumps over the lazy dog';
            const keywords = extractKeywords(text);
            
            expect(keywords).toContain('quick');
            expect(keywords).toContain('brown');
            expect(keywords).toContain('fox');
            expect(keywords).not.toContain('the');
            expect(keywords).not.toContain('a');
        });

        it('removes punctuation', () => {
            const text = 'Hello, world! How are you?';
            const keywords = extractKeywords(text);
            
            expect(keywords).toContain('hello');
            expect(keywords).toContain('world');
        });

        it('filters short words', () => {
            const text = 'I am a developer';
            const keywords = extractKeywords(text);
            
            expect(keywords).not.toContain('i');
            expect(keywords).not.toContain('am');
            expect(keywords).toContain('developer');
        });

        it('handles empty text', () => {
            const keywords = extractKeywords('');
            expect(keywords).toEqual([]);
        });
    });

    describe('estimateTokens', () => {
        it('estimates tokens from text length', () => {
            const text = 'This is a test message';
            const tokens = estimateTokens(text);
            
            expect(tokens).toBeGreaterThan(0);
            expect(tokens).toBe(Math.ceil(text.length / 4));
        });

        it('handles empty text', () => {
            const tokens = estimateTokens('');
            expect(tokens).toBe(0);
        });

        it('handles long text', () => {
            const text = 'a'.repeat(1000);
            const tokens = estimateTokens(text);
            expect(tokens).toBe(250);
        });
    });
});
