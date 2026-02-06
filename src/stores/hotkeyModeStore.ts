import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface HotkeyModeState {
  isHotkeyMode: boolean;

  // Actions
  enterHotkeyMode: () => void;
  exitHotkeyMode: () => void;
  pasteAndSimulate: (id: string) => Promise<void>;
  setupListeners: () => Promise<() => void>;
}

export const useHotkeyModeStore = create<HotkeyModeState>((set, get) => ({
  isHotkeyMode: false,

  enterHotkeyMode: () => {
    console.log('[HotkeyMode] Entering hotkey mode');
    set({ isHotkeyMode: true });
  },

  exitHotkeyMode: () => {
    console.log('[HotkeyMode] Exiting hotkey mode');
    set({ isHotkeyMode: false });
  },

  pasteAndSimulate: async (id: string) => {
    try {
      // Exit hotkey mode first
      set({ isHotkeyMode: false });
      // Call backend to paste and simulate Cmd+V
      await invoke('paste_and_simulate', { id });
    } catch (error) {
      console.error('Failed to paste and simulate:', error);
    }
  },

  setupListeners: async () => {
    // Listen for hotkey-mode-started event from backend
    const unlistenHotkeyMode = await listen('hotkey-mode-started', () => {
      get().enterHotkeyMode();
    });

    // Listen for panel-hidden event to exit hotkey mode when window loses focus
    const unlistenPanelHidden = await listen('panel-hidden', () => {
      get().exitHotkeyMode();
    });

    return () => {
      unlistenHotkeyMode();
      unlistenPanelHidden();
    };
  },
}));
