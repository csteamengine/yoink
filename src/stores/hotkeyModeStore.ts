import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useClipboardStore } from './clipboardStore';

interface HotkeyModeState {
  isHotkeyMode: boolean;

  // Actions
  enterHotkeyMode: () => void;
  exitHotkeyMode: () => void;
  setupListeners: () => Promise<() => void>;
}

export const useHotkeyModeStore = create<HotkeyModeState>((set, get) => ({
  isHotkeyMode: false,

  enterHotkeyMode: () => {
    console.log('[HotkeyMode] Entering hotkey mode');
    set({ isHotkeyMode: true });
    // Sync current selected item to backend for modifier-release paste
    const { items, selectedIndex } = useClipboardStore.getState();
    const selectedItem = items[selectedIndex];
    if (selectedItem) {
      invoke('set_selected_item', { id: selectedItem.id });
    }
  },

  exitHotkeyMode: () => {
    console.log('[HotkeyMode] Exiting hotkey mode');
    set({ isHotkeyMode: false });
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

    // Listen for hotkey-cycle event (V pressed again while holding modifiers)
    const unlistenCycle = await listen('hotkey-cycle', () => {
      if (!get().isHotkeyMode) return;
      console.log('[HotkeyMode] Cycling to next item');
      const { selectNext } = useClipboardStore.getState();
      selectNext();
      // Sync new selection to backend
      const { items, selectedIndex } = useClipboardStore.getState();
      const newItem = items[selectedIndex];
      if (newItem) {
        invoke('set_selected_item', { id: newItem.id });
      }
    });

    return () => {
      unlistenHotkeyMode();
      unlistenPanelHidden();
      unlistenCycle();
    };
  },
}));
