import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$infrastructure/ipc', () => ({
  invokeCommand: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

const { invokeCommand } = await import('$infrastructure/ipc');
const { open } = await import('@tauri-apps/plugin-dialog');
const {
  parseModelManifest,
  scanModelsDirectory,
  saveModelLibrary,
  loadModelLibrary,
  selectModelsDirectory,
  selectLlamaDirectory,
  formatModelIdentifier,
  parseModelIdentifier,
  getModelBlobPath,
} = await import('../../src/lib/services/models');

describe('models service', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('formats and parses model identifiers', () => {
    const identifier = formatModelIdentifier('provider', 'name', 'v1');
    expect(identifier).toBe('provider:name:v1');
    const parsed = parseModelIdentifier(identifier);
    expect(parsed).toEqual({ provider: 'provider', name: 'name', version: 'v1' });
  });

  it('throws for invalid identifier format', () => {
    expect(() => parseModelIdentifier('bad')).toThrow('Invalid model identifier');
  });

  it('builds blob path from digest', () => {
    const path = getModelBlobPath('/models', 'sha256:abc');
    expect(path).toBe('/models/blobs/sha256-abc');
  });

  it('invokes backend for parsing and scanning', async () => {
    (invokeCommand as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ name: 'model' })
      .mockResolvedValueOnce([{ name: 'model' }]);

    const parsed = await parseModelManifest('/m/manifest', '/models');
    expect(parsed).toEqual({ name: 'model' });

    const scanned = await scanModelsDirectory('/models');
    expect(scanned).toEqual([{ name: 'model' }]);
  });

  it('invokes backend for save/load library', async () => {
    (invokeCommand as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([{ name: 'one' }]);

    await saveModelLibrary('/models/modelLibrary.json', [{ name: 'one' } as any]);
    const loaded = await loadModelLibrary('/models/modelLibrary.json');
    expect(loaded).toEqual([{ name: 'one' }]);
  });

  it('selects directories via dialog', async () => {
    (open as ReturnType<typeof vi.fn>).mockResolvedValueOnce('/models');
    const selected = await selectModelsDirectory();
    expect(selected).toBe('/models');
  });

  it('wraps dialog errors', async () => {
    (open as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('fail'));
    await expect(selectLlamaDirectory()).rejects.toThrow('Failed to open directory dialog');
  });
});
