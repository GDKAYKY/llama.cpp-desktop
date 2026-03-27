import { describe, it, expect, vi, beforeEach } from 'vitest';
import { removeModelByManifestPath, removeModelByIdentifier } from '../../src/lib/services/models';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(true),
}));

const { invoke } = await import('@tauri-apps/api/core');

describe('models service - removal', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('removeModelByManifestPath calls the correct command', async () => {
    (invoke as any)
      .mockResolvedValueOnce(true);

    const result = await removeModelByManifestPath('/path/to/manifest.json', '/models/root');

    expect(invoke).toHaveBeenCalledWith('remove_model_by_manifest_path', {
      manifestPath: '/path/to/manifest.json',
      modelsRoot: '/models/root'
    });
    expect(result).toBe(true);
  });

  it('removeModelByIdentifier finds the model and removes it', async () => {
    const mockModel = {
      name: 'test-model',
      version: 'latest',
      provider: 'ollama',
      library: 'library',
      full_identifier: 'ollama:test-model:latest',
      manifest: {
        schema_version: 2,
        media_type: 'application/vnd.ollama.manifest.v1+json',
        config: {
          mediaType: 'application/vnd.ollama.image.config',
          digest: 'sha256:config',
          size: 100
        },
        layers: [{
          mediaType: 'application/vnd.ollama.image.model',
          digest: 'sha256:model',
          size: 1000
        }]
      },
      model_file_path: '/models/blobs/sha256-model'
    };

    // Mock the scanModelsDirectory to return our test model
    vi.mock('../../src/lib/services/models', async (importOriginal) => {
      const actual = await importOriginal();
      return {
        ...actual,
        scanModelsDirectory: vi.fn().mockResolvedValue([mockModel])
      };
    });

    const { scanModelsDirectory } = await import('../../src/lib/services/models');
    vi.mocked(scanModelsDirectory).mockResolvedValue([mockModel]);

    (invoke as any)
      .mockResolvedValueOnce(true);

    const result = await removeModelByIdentifier('ollama:test-model:latest', '/models/root');

    expect(scanModelsDirectory).toHaveBeenCalledWith('/models/root');
    expect(invoke).toHaveBeenCalledWith('remove_model_by_manifest_path', {
      manifestPath: '/models/root/manifests/ollama/library/test-model/latest/manifest.json',
      modelsRoot: '/models/root'
    });
    expect(result).toBe(true);
  });

  it('removeModelByIdentifier throws error if model not found', async () => {
    vi.mock('../../src/lib/services/models', async (importOriginal) => {
      const actual = await importOriginal();
      return {
        ...actual,
        scanModelsDirectory: vi.fn().mockResolvedValue([])
      };
    });

    const { scanModelsDirectory } = await import('../../src/lib/services/models');
    vi.mocked(scanModelsDirectory).mockResolvedValue([]);

    await expect(removeModelByIdentifier('nonexistent:model:latest', '/models/root'))
      .rejects
      .toThrow('Model with identifier nonexistent:model:latest not found');
  });
});
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
