import { useEffect } from 'react';
import { useClipboardStore } from '@/stores/clipboardStore';

export function useClipboardMonitor() {
  const { loadItems, loadPinnedItems, loadCollections, loadTags, setupListeners } =
    useClipboardStore();

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

    return () => {
      cleanup?.();
    };
  }, [loadItems, loadPinnedItems, loadCollections, loadTags, setupListeners]);
}
