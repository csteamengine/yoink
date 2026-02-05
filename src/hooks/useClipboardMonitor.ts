import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useClipboardStore } from '@/stores/clipboardStore';

export function useClipboardMonitor() {
  const { loadItems, loadPinnedItems, loadCollections, loadTags, setupListeners } =
    useClipboardStore();
  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    // Load initial data
    loadItems();
    loadPinnedItems();
    loadCollections();
    loadTags();

    // Setup event listeners
    let cleanup: (() => void) | undefined;

    setupListeners().then((unsub) => {
      cleanup = unsub;
    });

    // Poll for clipboard changes every 500ms
    const pollClipboard = async () => {
      try {
        await invoke('check_clipboard');
      } catch (error) {
        console.error('Clipboard poll error:', error);
      }
    };

    intervalRef.current = window.setInterval(pollClipboard, 500);

    return () => {
      cleanup?.();
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [loadItems, loadPinnedItems, loadCollections, loadTags, setupListeners]);
}
