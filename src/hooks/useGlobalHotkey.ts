import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '@/stores/settingsStore';

export function useGlobalHotkey() {
  const { settings } = useSettingsStore();

  useEffect(() => {
    // Register the hotkey when the component mounts or hotkey changes
    const registerHotkey = async () => {
      try {
        await invoke('register_hotkey', { hotkey: settings.hotkey });
      } catch (error) {
        console.error('Failed to register hotkey:', error);
      }
    };

    registerHotkey();
  }, [settings.hotkey]);
}
