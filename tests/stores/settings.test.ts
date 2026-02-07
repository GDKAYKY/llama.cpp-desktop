import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockConfig = {
  modelsDirectory: '/models',
  llamaDirectory: '/llama',
  theme: 'dark',
  language: 'en',
  maxTokens: 2048,
  temperature: 0.7,
  autoSaveChat: true,
  chatHistoryLimit: 50,
  serverPort: 8080,
};

describe('settings store', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
  });

  it('loads settings on init', async () => {
    vi.doMock('$lib/config/index', () => ({
      DEFAULT_CONFIG: mockConfig,
      loadConfig: vi.fn().mockResolvedValue({ theme: 'light' }),
      saveConfig: vi.fn(),
      resetConfig: vi.fn(),
    }));

    const { settingsStore } = await import('../../src/lib/stores/settings.svelte');
    await settingsStore.init();
    expect(settingsStore.settings.theme).toBe('light');
    expect(settingsStore.error).toBeNull();
  });

  it('sets error when loadConfig fails', async () => {
    vi.doMock('$lib/config/index', () => ({
      DEFAULT_CONFIG: mockConfig,
      loadConfig: vi.fn().mockRejectedValue(new Error('fail')),
      saveConfig: vi.fn(),
      resetConfig: vi.fn(),
    }));

    const { settingsStore } = await import('../../src/lib/stores/settings.svelte');
    await settingsStore.init();
    expect(settingsStore.error).toBe('Failed to load settings');
  });

  it('updates settings and handles errors', async () => {
    const saveConfig = vi.fn().mockResolvedValue(undefined);
    vi.doMock('$lib/config/index', () => ({
      DEFAULT_CONFIG: mockConfig,
      loadConfig: vi.fn().mockResolvedValue({}),
      saveConfig,
      resetConfig: vi.fn(),
    }));

    const { settingsStore } = await import('../../src/lib/stores/settings.svelte');
    await settingsStore.update({ theme: 'light' });
    expect(settingsStore.settings.theme).toBe('light');
    expect(saveConfig).toHaveBeenCalled();

    saveConfig.mockRejectedValueOnce(new Error('fail'));
    await settingsStore.update({ theme: 'dark' });
    expect(settingsStore.error).toBe('Failed to update settings');
  });

  it('resets settings with defaults', async () => {
    vi.doMock('$lib/config/index', () => ({
      DEFAULT_CONFIG: mockConfig,
      loadConfig: vi.fn().mockResolvedValue({}),
      saveConfig: vi.fn(),
      resetConfig: vi.fn().mockResolvedValue({ theme: 'light' }),
    }));

    const { settingsStore } = await import('../../src/lib/stores/settings.svelte');
    await settingsStore.reset();
    expect(settingsStore.settings.theme).toBe('light');
  });
});
