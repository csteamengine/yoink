import { useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useClipboardStore } from '@/stores/clipboardStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useQueueStore } from '@/stores/queueStore';

export function useKeyboardNav() {
  const {
    items,
    pinnedItems,
    selectedIndex,
    selectNext,
    selectPrevious,
    pasteSelected,
    pasteItem,
    deleteSelected,
    togglePinSelected,
    setSelectedIndex,
  } = useClipboardStore();

  const { isSettingsOpen, closeSettings } = useSettingsStore();
  const { isActive: queueModeActive, pasteNext } = useQueueStore();

  const handleKeyDown = useCallback(
    async (e: KeyboardEvent) => {
      // Don't handle if in input field (except for specific keys)
      const target = e.target as HTMLElement;
      const isInput =
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable;

      // Escape always works
      if (e.key === 'Escape') {
        if (isSettingsOpen) {
          closeSettings();
        } else {
          await invoke('hide_window');
        }
        return;
      }

      // If settings is open, don't handle other keys
      if (isSettingsOpen) return;

      // Navigation keys work even in search input
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectNext();
        return;
      }

      if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectPrevious();
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
      selectedIndex,
      selectNext,
      selectPrevious,
      pasteSelected,
      pasteItem,
      deleteSelected,
      togglePinSelected,
      setSelectedIndex,
      isSettingsOpen,
      closeSettings,
      queueModeActive,
      pasteNext,
    ]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);

  // Listen for quick-switch events from backend
  useEffect(() => {
    let unlistenCycle: (() => void) | undefined;
    let unlistenConfirm: (() => void) | undefined;

    const setupQuickSwitchListeners = async () => {
      // Cycle event: move to next item
      unlistenCycle = await listen('quick-switch-cycle', () => {
        selectNext();
      });

      // Confirm event: paste selected item
      unlistenConfirm = await listen('quick-switch-confirm', () => {
        pasteSelected();
      });
    };

    setupQuickSwitchListeners();

    return () => {
      unlistenCycle?.();
      unlistenConfirm?.();
    };
  }, [selectNext, pasteSelected]);
}
