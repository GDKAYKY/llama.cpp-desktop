import { describe, it, expect, vi } from 'vitest';
import { defaultConfig } from '$lib/config/defaultConfig';

describe('defaultConfig', () => {
    it('has modelsDirectory as null', () => {
        expect(defaultConfig.modelsDirectory).toBeNull();
    });

    it('has llamaCppPath as null', () => {
        expect(defaultConfig.llamaCppPath).toBeNull();
    });

    it('is a valid config object', () => {
        expect(defaultConfig).toBeDefined();
        expect(typeof defaultConfig).toBe('object');
    });
});
