import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$infrastructure/ipc', () => ({
  invokeCommand: vi.fn(),
}));

const { invokeCommand } = await import('$infrastructure/ipc');
const { loadConfig, saveConfig, resetConfig, getConfigPath } = await import('../../src/lib/config/index');

describe('config helpers', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('loads config via IPC', async () => {
    (invokeCommand as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ theme: 'dark' });
    const config = await loadConfig();
    expect(config).toEqual({ theme: 'dark' });
  });

  it('saves config via IPC', async () => {
    (invokeCommand as ReturnType<typeof vi.fn>).mockResolvedValueOnce(undefined);
    await saveConfig({ theme: 'light' } as any);
    expect(invokeCommand).toHaveBeenCalledWith('save_config', { config: { theme: 'light' } });
  });

  it('resets config via IPC', async () => {
    (invokeCommand as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ theme: 'dark' });
    const config = await resetConfig();
    expect(config).toEqual({ theme: 'dark' });
  });

  it('gets config path via IPC', async () => {
    (invokeCommand as ReturnType<typeof vi.fn>).mockResolvedValueOnce('/path/config.json');
    const path = await getConfigPath();
    expect(path).toBe('/path/config.json');
  });
});
