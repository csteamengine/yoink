import { useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useClipboardStore } from '@/stores/clipboardStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useQueueStore } from '@/stores/queueStore';
import { useHotkeyModeStore } from '@/stores/hotkeyModeStore';

export function useKeyboardNav() {
  const {
    items,
    pinnedItems,
    selectNext,
    selectPrevious,
    pasteSelected,
    pasteItem,
    deleteSelected,
    togglePinSelected,
    setSelectedIndex,
  } = useClipboardStore();

  const { isSettingsOpen, closeSettings, openSettings } = useSettingsStore();
  const { isActive: queueModeActive, pasteNext } = useQueueStore();
  const { isHotkeyMode, exitHotkeyMode, cycleNext } = useHotkeyModeStore();

  const handleKeyDown = useCallback(
    async (e: KeyboardEvent) => {
      // Don't handle if in input field (except for specific keys)
      const target = e.target as HTMLElement;
      const isInput =
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable;

      // Escape always works - also exits hotkey mode without pasting
      if (e.key === 'Escape') {
        if (isHotkeyMode) {
          exitHotkeyMode();
        }
        // Always exit backend hotkey mode to prevent modifier-release paste,
        // even if frontend state is out of sync.
        invoke('exit_hotkey_mode');
        if (isSettingsOpen) {
          closeSettings();
        } else {
          await invoke('hide_window');
        }
        return;
      }

      // Cmd+, to open settings
      if (e.metaKey && e.key === ',') {
        e.preventDefault();
        openSettings();
        return;
      }

      // V key cycles to next item in hotkey mode.
      // Uses e.code for consistency regardless of modifiers held.
      // cycleNext() reads isHotkeyMode from store (not stale closure) and deduplicates
      // with backend hotkey-cycle events to prevent double-cycling.
      if (e.code === 'KeyV' && useHotkeyModeStore.getState().isHotkeyMode) {
        e.preventDefault();
        cycleNext();
        return;
      }

      // If settings is open, don't handle other keys
      if (isSettingsOpen) return;

      // Navigation keys work even in search input
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectNext();
        // Sync selection to backend for hotkey mode paste on modifier release
        if (isHotkeyMode) {
          const state = useClipboardStore.getState();
          const item = state.items[state.selectedIndex];
          if (item) invoke('set_selected_item', { id: item.id });
        }
        return;
      }

      if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectPrevious();
        // Sync selection to backend for hotkey mode paste on modifier release
        if (isHotkeyMode) {
          const state = useClipboardStore.getState();
          const item = state.items[state.selectedIndex];
          if (item) invoke('set_selected_item', { id: item.id });
        }
        return;
      }

      // Enter to paste
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        if (queueModeActive) {
          await pasteNext();
        } else {
          await pasteSelected();
        }
        return;
      }

      // Don't handle other shortcuts when in input
      if (isInput) return;

      // Delete/Backspace to delete selected
      if (e.key === 'Delete' || e.key === 'Backspace') {
        e.preventDefault();
        await deleteSelected();
        return;
      }

      // Cmd/Ctrl+P to toggle pin
      if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
        e.preventDefault();
        await togglePinSelected();
        return;
      }

      // Cmd/Ctrl+Shift+[1-9,0] to paste pinned item at slot
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && /^[0-9]$/.test(e.key)) {
        e.preventDefault();
        const slot = e.key === '0' ? 9 : parseInt(e.key) - 1;
        if (pinnedItems[slot]) {
          await pasteItem(pinnedItems[slot].id);
        }
        return;
      }

      // Number keys 1-9 to select item
      if (/^[1-9]$/.test(e.key) && !e.metaKey && !e.ctrlKey && !e.shiftKey) {
        const index = parseInt(e.key) - 1;
        if (index < items.length) {
          setSelectedIndex(index);
        }
        return;
      }
    },
    [
      items,
      pinnedItems,
      selectNext,
      selectPrevious,
      pasteSelected,
      pasteItem,
      deleteSelected,
      togglePinSelected,
      setSelectedIndex,
      isSettingsOpen,
      closeSettings,
      openSettings,
      queueModeActive,
      pasteNext,
      isHotkeyMode,
      exitHotkeyMode,
      cycleNext,
    ]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);
}
