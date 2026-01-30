import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock SvelteKit modules if needed
vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

vi.mock('$app/environment', () => ({
  browser: true,
}));
