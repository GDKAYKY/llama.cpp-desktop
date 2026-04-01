import { describe, it, expect, vi } from 'vitest';
import { DEFAULT_CONFIG as defaultConfig } from '$lib/config/defaultConfig';

describe('defaultConfig', () => {
    it('has modelsDirectory as null', () => {
        expect(defaultConfig.modelsDirectory).toBeNull();
    });

    it('has llamaDirectory as null', () => {
        expect(defaultConfig.llamaDirectory).toBeNull();
    });

    it('is a valid config object', () => {
        expect(defaultConfig).toBeDefined();
        expect(typeof defaultConfig).toBe('object');
    });
});
