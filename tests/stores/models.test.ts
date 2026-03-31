import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

vi.mock('$lib/services/models', () => ({
    scanModelsDirectory: vi.fn().mockResolvedValue([]),
    loadModelLibrary: vi.fn().mockResolvedValue([]),
    saveModelLibrary: vi.fn().mockResolvedValue(undefined),
    selectModelsDirectory: vi.fn().mockResolvedValue('/models'),
}));

vi.mock('$lib/stores/settings.svelte', () => ({
    settingsStore: {
        settings: { modelsDirectory: '/models' },
        update: vi.fn().mockResolvedValue(undefined),
    },
}));

describe('modelsStore', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('computes libraryPath from modelsRoot', async () => {
        const { modelsStore } = await import('$lib/stores/models.svelte');
        expect(modelsStore.libraryPath).toBe('/models/modelLibrary.json');
    });

    it('returns empty libraryPath when no modelsRoot', async () => {
        vi.doMock('$lib/stores/settings.svelte', () => ({
            settingsStore: {
                settings: { modelsDirectory: null },
                update: vi.fn(),
            },
        }));
        
        const { modelsStore } = await import('$lib/stores/models.svelte');
        expect(modelsStore.libraryPath).toBe('');
    });

    it('sets error when scanning without modelsRoot', async () => {
        vi.doMock('$lib/stores/settings.svelte', () => ({
            settingsStore: {
                settings: { modelsDirectory: null },
                update: vi.fn(),
            },
        }));
        
        const { modelsStore } = await import('$lib/stores/models.svelte');
        await modelsStore.scan();
        expect(modelsStore.error).toBe('Please select a models directory first');
    });
});
