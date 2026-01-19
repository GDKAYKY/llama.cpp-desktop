import { loadConfig, saveConfig, resetConfig, DEFAULT_CONFIG } from '$lib/config.js';

class SettingsStore {
  settings = $state({ ...DEFAULT_CONFIG });
  isLoading = $state(true);
  error = $state<string | null>(null);

  constructor() {
    this.init();
  }

  async init() {
    try {
      this.isLoading = true;
      const config = await loadConfig();
      this.settings = { ...DEFAULT_CONFIG, ...config };
      this.error = null;
    } catch (err) {
      console.error('Failed to load settings:', err);
      this.error = 'Failed to load settings';
    } finally {
      this.isLoading = false;
    }
  }

  async update(newSettings: Partial<typeof DEFAULT_CONFIG>) {
    try {
      this.settings = { ...this.settings, ...newSettings };
      await saveConfig(this.settings);
    } catch (err) {
      console.error('Failed to update settings:', err);
      this.error = 'Failed to update settings';
    }
  }

  async reset() {
    try {
      const config = await resetConfig();
      this.settings = { ...DEFAULT_CONFIG, ...config };
    } catch (err) {
      console.error('Failed to reset settings:', err);
      this.error = 'Failed to reset settings';
    }
  }
}

export const settingsStore = new SettingsStore();
