import { describe, it, expect, vi, beforeEach } from 'vitest';

describe('models store', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
  });

  it('computes libraryPath from settings', async () => {
    vi.doMock('$lib/stores/settings.svelte', () => ({
      settingsStore: {
        settings: { modelsDirectory: '/models' },
        update: vi.fn(),
      },
    }));
    vi.doMock('$lib/services/models', () => ({
      scanModelsDirectory: vi.fn(),
      loadModelLibrary: vi.fn(),
      saveModelLibrary: vi.fn(),
      selectModelsDirectory: vi.fn(),
    }));

    const { modelsStore } = await import('../../src/lib/stores/models.svelte');
    expect(modelsStore.libraryPath).toBe('/models/modelLibrary.json');
  });

  it('scans models and saves library', async () => {
    const scanModelsDirectory = vi.fn().mockResolvedValue([{ name: 'model' }]);
    const saveModelLibrary = vi.fn().mockResolvedValue(undefined);

    vi.doMock('$lib/stores/settings.svelte', () => ({
      settingsStore: {
        settings: { modelsDirectory: '/models' },
        update: vi.fn(),
      },
    }));
    vi.doMock('$lib/services/models', () => ({
      scanModelsDirectory,
      loadModelLibrary: vi.fn(),
      saveModelLibrary,
      selectModelsDirectory: vi.fn(),
    }));

    const { modelsStore } = await import('../../src/lib/stores/models.svelte');
    await modelsStore.scan();
    expect(modelsStore.models.length).toBe(1);
    expect(modelsStore.successMessage).toContain('Found and saved');
    expect(saveModelLibrary).toHaveBeenCalled();
  });

  it('sets error when no models root', async () => {
    vi.doMock('$lib/stores/settings.svelte', () => ({
      settingsStore: {
        settings: { modelsDirectory: null },
        update: vi.fn(),
      },
    }));
    vi.doMock('$lib/services/models', () => ({
      scanModelsDirectory: vi.fn(),
      loadModelLibrary: vi.fn(),
      saveModelLibrary: vi.fn(),
      selectModelsDirectory: vi.fn(),
    }));

    const { modelsStore } = await import('../../src/lib/stores/models.svelte');
    await modelsStore.scan();
    expect(modelsStore.error).toBe('Please select a models directory first');
  });

  it('selects directory and loads library', async () => {
    const selectModelsDirectory = vi.fn().mockResolvedValue('/models');
    const loadModelLibrary = vi.fn().mockResolvedValue([{ name: 'one' }]);
    const update = vi.fn().mockResolvedValue(undefined);

    vi.doMock('$lib/stores/settings.svelte', () => ({
      settingsStore: {
        settings: { modelsDirectory: '' },
        update,
      },
    }));
    vi.doMock('$lib/services/models', () => ({
      scanModelsDirectory: vi.fn(),
      loadModelLibrary,
      saveModelLibrary: vi.fn(),
      selectModelsDirectory,
    }));

    const { modelsStore } = await import('../../src/lib/stores/models.svelte');
    await modelsStore.selectDirectory();
    expect(update).toHaveBeenCalled();
    expect(modelsStore.models.length).toBe(1);
  });

  it('selectModel updates selection and clears messages', async () => {
    vi.doMock('$lib/stores/settings.svelte', () => ({
      settingsStore: {
        settings: { modelsDirectory: '/models' },
        update: vi.fn(),
      },
    }));
    vi.doMock('$lib/services/models', () => ({
      scanModelsDirectory: vi.fn(),
      loadModelLibrary: vi.fn(),
      saveModelLibrary: vi.fn(),
      selectModelsDirectory: vi.fn(),
    }));

    const { modelsStore } = await import('../../src/lib/stores/models.svelte');
    modelsStore.error = 'err';
    modelsStore.successMessage = 'ok';
    modelsStore.selectModel({ name: 'm1' } as any);
    expect(modelsStore.selectedModel?.name).toBe('m1');
    modelsStore.clearMessages();
    expect(modelsStore.error).toBeNull();
    expect(modelsStore.successMessage).toBe('');
  });
});
