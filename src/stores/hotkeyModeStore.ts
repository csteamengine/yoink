import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useClipboardStore } from './clipboardStore';

interface HotkeyModeState {
  isHotkeyMode: boolean;

  // Actions
  enterHotkeyMode: () => void;
  exitHotkeyMode: () => void;
  cycleNext: () => void;
  setupListeners: () => Promise<() => void>;
}

// Dedup timestamp shared by all cycle sources (global shortcut, backend polling, frontend keydown)
let lastCycleTime = 0;

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

  // Unified cycle handler with dedup - called from hotkey-cycle events AND frontend keydown
  cycleNext: () => {
    if (!get().isHotkeyMode) return;
    const now = Date.now();
    if (now - lastCycleTime < 100) return;
    lastCycleTime = now;
    console.log('[HotkeyMode] Cycling to next item');
    const { selectNext } = useClipboardStore.getState();
    selectNext();
    // Sync new selection to backend
    const { items, selectedIndex } = useClipboardStore.getState();
    const newItem = items[selectedIndex];
    if (newItem) {
      invoke('set_selected_item', { id: newItem.id });
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

    // Listen for hotkey-cycle event from backend (global shortcut handler or polling thread)
    const unlistenCycle = await listen('hotkey-cycle', () => {
      get().cycleNext();
    });

    // If the frontend loaded after hotkey mode started, sync once on setup.
    try {
      const active = await invoke<boolean>('is_hotkey_mode_active');
      if (active) {
        get().enterHotkeyMode();
      }
    } catch {
      // Ignore sync failures; normal events will still handle mode changes.
    }

    return () => {
      unlistenHotkeyMode();
      unlistenPanelHidden();
      unlistenCycle();
    };
  },
}));
