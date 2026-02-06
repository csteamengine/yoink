import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface Settings {
  hotkey: string;
  launch_at_startup: boolean;
  history_limit: number;
  theme: 'light' | 'dark' | 'system';
  accent_color: 'blue' | 'purple' | 'green';
  font_size: number;
  show_timestamps: boolean;
  excluded_apps: string[];
  queue_mode_enabled: boolean;
  auto_paste: boolean;
  sticky_mode: boolean;
}

interface SettingsState {
  settings: Settings;
  isLoading: boolean;
  error: string | null;
  isSettingsOpen: boolean;
  activeTab: string;

  // Actions
  loadSettings: () => Promise<void>;
  updateSettings: (settings: Partial<Settings>) => Promise<void>;
  setHotkey: (hotkey: string) => Promise<void>;
  setTheme: (theme: Settings['theme']) => Promise<void>;
  setAccentColor: (color: Settings['accent_color']) => Promise<void>;
  addExcludedApp: (appId: string) => Promise<void>;
  removeExcludedApp: (appId: string) => Promise<void>;
  toggleQueueMode: () => Promise<void>;
  openSettings: (tab?: string) => void;
  closeSettings: () => void;
  setActiveTab: (tab: string) => void;
  applyTheme: () => void;
  setupListeners: () => Promise<() => void>;
}

const defaultSettings: Settings = {
  hotkey: 'CommandOrControl+Shift+V',
  launch_at_startup: false,
  history_limit: 100,
  theme: 'system',
  accent_color: 'blue',
  font_size: 14,
  show_timestamps: true,
  excluded_apps: [],
  queue_mode_enabled: false,
  auto_paste: true,
  sticky_mode: false,
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: defaultSettings,
  isLoading: false,
  error: null,
  isSettingsOpen: false,
  activeTab: 'general',

  loadSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await invoke<Settings>('get_settings');
      set({ settings, isLoading: false });
      get().applyTheme();
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  updateSettings: async (partial: Partial<Settings>) => {
    const { settings } = get();
    const newSettings = { ...settings, ...partial };
    try {
      await invoke('update_settings', { settings: newSettings });
      set({ settings: newSettings });
      get().applyTheme();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setHotkey: async (hotkey: string) => {
    try {
      // Validate hotkey first
      const isValid = await invoke<boolean>('validate_hotkey', { hotkey });
      if (!isValid) {
        set({ error: 'Invalid hotkey format' });
        return;
      }

      // Update settings
      const settings = await invoke<Settings>('set_hotkey', { hotkey });
      set({ settings });

      // Re-register the hotkey
      await invoke('register_hotkey', { hotkey });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setTheme: async (theme: Settings['theme']) => {
    try {
      const settings = await invoke<Settings>('set_theme', { theme });
      set({ settings });
      get().applyTheme();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setAccentColor: async (accentColor: Settings['accent_color']) => {
    try {
      const settings = await invoke<Settings>('set_accent_color', { accentColor });
      set({ settings });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  addExcludedApp: async (appId: string) => {
    try {
      const settings = await invoke<Settings>('add_excluded_app', { appId });
      set({ settings });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  removeExcludedApp: async (appId: string) => {
    try {
      const settings = await invoke<Settings>('remove_excluded_app', { appId });
      set({ settings });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  toggleQueueMode: async () => {
    try {
      const settings = await invoke<Settings>('toggle_queue_mode');
      set({ settings });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  openSettings: (tab?: string) => {
    set({ isSettingsOpen: true, activeTab: tab || 'general' });
  },

  closeSettings: () => {
    set({ isSettingsOpen: false });
  },

  setActiveTab: (tab: string) => {
    set({ activeTab: tab });
  },

  applyTheme: () => {
    const { settings } = get();
    const root = document.documentElement;

    let effectiveTheme = settings.theme;
    if (effectiveTheme === 'system') {
      effectiveTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light';
    }

    if (effectiveTheme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }

    // Apply accent color
    root.classList.remove('accent-blue', 'accent-purple', 'accent-green');
    if (settings.accent_color !== 'blue') {
      root.classList.add(`accent-${settings.accent_color}`);
    }

    // Apply font size
    root.style.fontSize = `${settings.font_size}px`;
  },

  setupListeners: async () => {
    const unlistenSettings = await listen('open-settings', () => {
      get().openSettings();
    });

    // Register global callback for backend to call via eval (tray menu)
    (window as any).__openSettings = () => {
      get().openSettings();
    };

    // Listen for system theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleThemeChange = () => {
      if (get().settings.theme === 'system') {
        get().applyTheme();
      }
    };
    mediaQuery.addEventListener('change', handleThemeChange);

    return () => {
      unlistenSettings();
      delete (window as any).__openSettings;
      mediaQuery.removeEventListener('change', handleThemeChange);
    };
  },
}));
