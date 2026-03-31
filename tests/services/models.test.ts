import { describe, it, expect, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

describe('models service', () => {
    it('scanModelsDirectory calls backend', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        const { scanModelsDirectory } = await import('$lib/services/models');
        
        vi.mocked(invoke).mockResolvedValue([
            { name: 'model1', provider: 'test' },
        ]);
        
        const models = await scanModelsDirectory('/models');
        
        expect(invoke).toHaveBeenCalledWith('scan_models_directory', { 
            modelsRoot: '/models' 
        });
        expect(models).toHaveLength(1);
    });

    it('loadModelLibrary calls backend', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        const { loadModelLibrary } = await import('$lib/services/models');
        
        vi.mocked(invoke).mockResolvedValue([]);
        
        await loadModelLibrary('/models/library.json');
        
        expect(invoke).toHaveBeenCalledWith('load_model_library', {
            libraryPath: '/models/library.json'
        });
    });

    it('saveModelLibrary calls backend', async () => {
        const { invoke } = await import('@tauri-apps/api/core');
        const { saveModelLibrary } = await import('$lib/services/models');
        
        vi.mocked(invoke).mockResolvedValue(undefined);
        
        await saveModelLibrary('/models/library.json', []);
        
        expect(invoke).toHaveBeenCalledWith('save_model_library', {
            libraryPath: '/models/library.json',
            library: { models: [] }
        });
    });
});
