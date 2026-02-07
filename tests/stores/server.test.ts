import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('server store', () => {
  beforeEach(() => {
    vi.resetModules();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it('starts server and updates config', async () => {
    const invokeCommand = vi.fn()
      .mockResolvedValueOnce(false) // is_server_running during init
      .mockResolvedValueOnce('1234'); // start_llama_server

    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');

    await serverStore.startServer('/bin', '/model', 8000, 4096, 33, 1);
    expect(serverStore.isRunning).toBe(true);
    expect(serverStore.currentConfig?.model_path).toBe('/model');
  });

  it('stops server and clears state', async () => {
    const invokeCommand = vi.fn()
      .mockResolvedValueOnce(false) // init check
      .mockResolvedValueOnce(undefined); // stop_llama_server

    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');

    serverStore.isRunning = true;
    serverStore.currentConfig = {
      llama_cpp_path: '/bin',
      model_path: '/model',
      port: 8000,
      ctx_size: 4096,
      parallel: 1,
      n_gpu_layers: 0,
    };
    await serverStore.stopServer();
    expect(serverStore.isRunning).toBe(false);
    expect(serverStore.currentConfig).toBeNull();
  });

  it('startServer returns early when already starting', async () => {
    const invokeCommand = vi.fn().mockResolvedValueOnce(false);
    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');
    serverStore.isStarting = true;
    await serverStore.startServer('/bin', '/model');
    expect(invokeCommand).toHaveBeenCalledTimes(1); // only init check
  });

  it('startServer returns early when config matches', async () => {
    const invokeCommand = vi.fn().mockResolvedValueOnce(false);
    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');
    serverStore.isRunning = true;
    serverStore.currentConfig = {
      llama_cpp_path: '/bin',
      model_path: '/model',
      port: 8000,
      ctx_size: 4096,
      parallel: 1,
      n_gpu_layers: 33,
    };
    await serverStore.startServer('/bin', '/model', 8000, 4096, 33, 1);
    expect(invokeCommand).toHaveBeenCalledTimes(1); // only init check
  });

  it('checkHealth handles not running and unhealthy', async () => {
    const invokeCommand = vi.fn()
      .mockResolvedValueOnce(false) // init check
      .mockResolvedValueOnce(false); // check_server_health

    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');

    serverStore.isRunning = false;
    await serverStore.checkHealth();
    expect(serverStore.isHealthy).toBe(false);

    serverStore.isRunning = true;
    await serverStore.checkHealth();
    expect(serverStore.error).toBe('Server health check failed');
  });

  it('checkRunning updates config', async () => {
    const invokeCommand = vi.fn()
      .mockResolvedValueOnce(false) // init check
      .mockResolvedValueOnce(true) // is_server_running
      .mockResolvedValueOnce({
        llama_cpp_path: '/bin',
        model_path: '/model',
        port: 8000,
        ctx_size: 4096,
        parallel: 1,
        n_gpu_layers: 0,
      }); // get_llama_config

    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');

    await serverStore.checkRunning();
    expect(serverStore.isRunning).toBe(true);
    expect(serverStore.currentConfig?.model_path).toBe('/model');
  });

  it('fetchMetrics no-ops when not running', async () => {
    const invokeCommand = vi.fn().mockResolvedValueOnce(false);
    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');
    serverStore.isRunning = false;
    await serverStore.fetchMetrics();
    expect(invokeCommand).toHaveBeenCalledTimes(1); // only init check
  });

  it('health monitoring triggers metrics on models page', async () => {
    const invokeCommand = vi.fn()
      .mockResolvedValueOnce(false) // init check
      .mockResolvedValueOnce('1234') // start_llama_server
      .mockResolvedValueOnce(true) // check_server_health
      .mockResolvedValueOnce({ cpu_usage: 1 }); // get_server_metrics

    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');

    Object.defineProperty(window, 'location', {
      value: { pathname: '/models' },
      writable: true,
    });

    await serverStore.startServer('/bin', '/model', 8000, 4096, 33, 1);
    await vi.advanceTimersByTimeAsync(2000);
    expect(invokeCommand).toHaveBeenCalledWith('check_server_health');
    expect(invokeCommand).toHaveBeenCalledWith('get_server_metrics');
  });

  it('startHealthMonitoring clears existing interval', async () => {
    const invokeCommand = vi.fn().mockResolvedValueOnce(false);
    vi.doMock('$infrastructure/ipc', () => ({ invokeCommand }));
    const { serverStore } = await import('../../src/lib/stores/server.svelte');

    serverStore.isRunning = true;
    serverStore.startHealthMonitoring();
    const first = (serverStore as any).healthInterval;
    serverStore.startHealthMonitoring();
    const second = (serverStore as any).healthInterval;
    expect(first).not.toBeNull();
    expect(second).not.toBeNull();
    expect(second).not.toBe(first);
  });
});
