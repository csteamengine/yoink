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
  const { isHotkeyMode, exitHotkeyMode, pasteAndSimulate } = useHotkeyModeStore();

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
          await invoke('hide_window');
          return;
        }
        if (isSettingsOpen) {
          closeSettings();
        } else {
          await invoke('hide_window');
        }
        return;
      }

      // In hotkey mode, V key (with modifiers) cycles to next item
      if (isHotkeyMode && e.key.toLowerCase() === 'v' && (e.metaKey || e.shiftKey)) {
        console.log('[HotkeyMode] V pressed with modifiers, cycling to next');
        e.preventDefault();
        selectNext();
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
      isHotkeyMode,
      exitHotkeyMode,
    ]
  );

  // Handle key up for detecting modifier release in hotkey mode
  const handleKeyUp = useCallback(
    async (e: KeyboardEvent) => {
      // Only process modifier key releases in hotkey mode
      if (!isHotkeyMode) return;

      // Check if this is a modifier key being released
      const isModifierRelease = e.key === 'Meta' || e.key === 'Shift';
      if (!isModifierRelease) return;

      console.log(`[HotkeyMode] Modifier released: ${e.key}, meta=${e.metaKey}, shift=${e.shiftKey}`);

      // Use the event's actual modifier state (more reliable than tracking)
      // After releasing Meta, e.metaKey will be false
      // After releasing Shift, e.shiftKey will be false
      const metaStillHeld = e.metaKey;
      const shiftStillHeld = e.shiftKey;

      // Only paste when BOTH modifiers are now released
      if (!metaStillHeld && !shiftStillHeld) {
        console.log('[HotkeyMode] Both modifiers released, pasting...');
        const selectedItem = items[selectedIndex];
        if (selectedItem) {
          await pasteAndSimulate(selectedItem.id);
        } else {
          exitHotkeyMode();
          await invoke('hide_window');
        }
      }
    },
    [isHotkeyMode, items, selectedIndex, pasteAndSimulate, exitHotkeyMode]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [handleKeyDown, handleKeyUp]);
}
