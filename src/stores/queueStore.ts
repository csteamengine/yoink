import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ClipboardItem } from './clipboardStore';

interface QueueState {
  queue: ClipboardItem[];
  currentIndex: number;
  isActive: boolean;

  // Actions
  addToQueue: (item: ClipboardItem) => void;
  removeFromQueue: (id: string) => void;
  reorderQueue: (fromIndex: number, toIndex: number) => void;
  clearQueue: () => void;
  pasteNext: () => Promise<void>;
  toggleActive: () => void;
  setActive: (active: boolean) => void;
}

export const useQueueStore = create<QueueState>((set, get) => ({
  queue: [],
  currentIndex: 0,
  isActive: false,

  addToQueue: (item: ClipboardItem) => {
    const { queue } = get();
    // Don't add duplicates
    if (queue.some((i) => i.id === item.id)) return;
    set({ queue: [...queue, item] });
  },

  removeFromQueue: (id: string) => {
    const { queue, currentIndex } = get();
    const newQueue = queue.filter((i) => i.id !== id);
    const removedIndex = queue.findIndex((i) => i.id === id);

    // Adjust current index if needed
    let newIndex = currentIndex;
    if (removedIndex < currentIndex) {
      newIndex = Math.max(0, currentIndex - 1);
    } else if (removedIndex === currentIndex && newIndex >= newQueue.length) {
      newIndex = Math.max(0, newQueue.length - 1);
    }

    set({ queue: newQueue, currentIndex: newIndex });
  },

  reorderQueue: (fromIndex: number, toIndex: number) => {
    const { queue, currentIndex } = get();
    const newQueue = [...queue];
    const [removed] = newQueue.splice(fromIndex, 1);
    newQueue.splice(toIndex, 0, removed);

    // Adjust current index
    let newIndex = currentIndex;
    if (fromIndex === currentIndex) {
      newIndex = toIndex;
    } else if (fromIndex < currentIndex && toIndex >= currentIndex) {
      newIndex = currentIndex - 1;
    } else if (fromIndex > currentIndex && toIndex <= currentIndex) {
      newIndex = currentIndex + 1;
    }

    set({ queue: newQueue, currentIndex: newIndex });
  },

  clearQueue: () => {
    set({ queue: [], currentIndex: 0, isActive: false });
  },

  pasteNext: async () => {
    const { queue, currentIndex, isActive } = get();

    if (!isActive || queue.length === 0) return;

    const item = queue[currentIndex];
    if (!item) {
      // Queue exhausted
      set({ isActive: false, currentIndex: 0 });
      return;
    }

    try {
      await invoke('paste_item', { id: item.id });
      await invoke('hide_window');

      // Move to next item
      const nextIndex = currentIndex + 1;
      if (nextIndex >= queue.length) {
        // Queue completed
        set({ isActive: false, currentIndex: 0, queue: [] });
      } else {
        set({ currentIndex: nextIndex });
      }
    } catch (error) {
      console.error('Failed to paste queue item:', error);
    }
  },

  toggleActive: () => {
    const { isActive, queue } = get();
    if (!isActive && queue.length === 0) return;
    set({ isActive: !isActive, currentIndex: isActive ? get().currentIndex : 0 });
  },

  setActive: (active: boolean) => {
    if (active && get().queue.length === 0) return;
    set({ isActive: active, currentIndex: active ? 0 : get().currentIndex });
  },
}));
