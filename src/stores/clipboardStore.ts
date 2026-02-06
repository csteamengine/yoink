import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface ClipboardItem {
  id: string;
  content_type: string;
  content: string;
  preview: string;
  hash: string;
  is_pinned: boolean;
  collection_id: string | null;
  created_at: string;
  expires_at: string | null;
}

export interface Collection {
  id: string;
  name: string;
  color: string;
  created_at: string;
}

export interface Tag {
  id: string;
  name: string;
}

interface ClipboardState {
  items: ClipboardItem[];
  pinnedItems: ClipboardItem[];
  selectedIndex: number;
  search: string;
  selectedCollectionId: string | null;
  collections: Collection[];
  tags: Tag[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadItems: () => Promise<void>;
  loadPinnedItems: () => Promise<void>;
  loadCollections: () => Promise<void>;
  loadTags: () => Promise<void>;
  setSearch: (search: string) => void;
  setSelectedIndex: (index: number) => void;
  selectNext: () => void;
  selectPrevious: () => void;
  pasteSelected: () => Promise<void>;
  pasteItem: (id: string) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  deleteSelected: () => Promise<void>;
  pinItem: (id: string) => Promise<void>;
  unpinItem: (id: string) => Promise<void>;
  togglePinSelected: () => Promise<void>;
  clearHistory: () => Promise<void>;
  moveToCollection: (itemId: string, collectionId: string | null) => Promise<void>;
  setSelectedCollection: (collectionId: string | null) => void;
  createCollection: (name: string, color: string) => Promise<Collection>;
  deleteCollection: (id: string) => Promise<void>;
  updateCollection: (id: string, name: string, color: string) => Promise<void>;
  createTag: (name: string) => Promise<Tag>;
  addTagToItem: (itemId: string, tagId: string) => Promise<void>;
  removeTagFromItem: (itemId: string, tagId: string) => Promise<void>;
  setExpiration: (itemId: string, expiresAt: string | null) => Promise<void>;
  setupListeners: () => Promise<() => void>;
}

export const useClipboardStore = create<ClipboardState>((set, get) => ({
  items: [],
  pinnedItems: [],
  selectedIndex: 0,
  search: '',
  selectedCollectionId: null,
  collections: [],
  tags: [],
  isLoading: false,
  error: null,

  loadItems: async () => {
    set({ isLoading: true, error: null });
    try {
      const { search, selectedCollectionId } = get();
      const items = await invoke<ClipboardItem[]>('get_clipboard_items', {
        limit: 100,
        offset: 0,
        search: search || null,
        collectionId: selectedCollectionId,
      });
      set({ items, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  loadPinnedItems: async () => {
    try {
      const pinnedItems = await invoke<ClipboardItem[]>('get_pinned_items');
      set({ pinnedItems });
    } catch (error) {
      console.error('Failed to load pinned items:', error);
    }
  },

  loadCollections: async () => {
    try {
      const collections = await invoke<Collection[]>('get_collections');
      set({ collections });
    } catch (error) {
      console.error('Failed to load collections:', error);
    }
  },

  loadTags: async () => {
    try {
      const tags = await invoke<Tag[]>('get_tags');
      set({ tags });
    } catch (error) {
      console.error('Failed to load tags:', error);
    }
  },

  setSearch: (search: string) => {
    set({ search, selectedIndex: 0 });
    get().loadItems();
  },

  setSelectedIndex: (index: number) => {
    const { items } = get();
    if (index >= 0 && index < items.length) {
      set({ selectedIndex: index });
    }
  },

  selectNext: () => {
    const { items, selectedIndex } = get();
    if (selectedIndex < items.length - 1) {
      set({ selectedIndex: selectedIndex + 1 });
    }
  },

  selectPrevious: () => {
    const { selectedIndex } = get();
    if (selectedIndex > 0) {
      set({ selectedIndex: selectedIndex - 1 });
    }
  },

  pasteSelected: async () => {
    const { items, selectedIndex } = get();
    if (items[selectedIndex]) {
      await get().pasteItem(items[selectedIndex].id);
    }
  },

  pasteItem: async (id: string) => {
    try {
      // Write to clipboard, hide window, restore focus, and simulate Cmd+V
      await invoke('paste_and_simulate', { id });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  deleteItem: async (id: string) => {
    try {
      await invoke('delete_clipboard_item', { id });
      await get().loadItems();
      await get().loadPinnedItems();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  deleteSelected: async () => {
    const { items, selectedIndex } = get();
    if (items[selectedIndex]) {
      await get().deleteItem(items[selectedIndex].id);
    }
  },

  pinItem: async (id: string) => {
    try {
      await invoke('pin_item', { id });
      await get().loadItems();
      await get().loadPinnedItems();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  unpinItem: async (id: string) => {
    try {
      await invoke('unpin_item', { id });
      await get().loadItems();
      await get().loadPinnedItems();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  togglePinSelected: async () => {
    const { items, selectedIndex } = get();
    const item = items[selectedIndex];
    if (item) {
      if (item.is_pinned) {
        await get().unpinItem(item.id);
      } else {
        await get().pinItem(item.id);
      }
    }
  },

  clearHistory: async () => {
    try {
      await invoke('clear_history');
      await get().loadItems();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  moveToCollection: async (itemId: string, collectionId: string | null) => {
    try {
      await invoke('move_to_collection', { itemId, collectionId });
      await get().loadItems();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setSelectedCollection: (collectionId: string | null) => {
    set({ selectedCollectionId: collectionId, selectedIndex: 0 });
    get().loadItems();
  },

  createCollection: async (name: string, color: string) => {
    const collection = await invoke<Collection>('create_collection', { name, color });
    await get().loadCollections();
    return collection;
  },

  deleteCollection: async (id: string) => {
    try {
      await invoke('delete_collection', { id });
      await get().loadCollections();
      if (get().selectedCollectionId === id) {
        set({ selectedCollectionId: null });
        await get().loadItems();
      }
    } catch (error) {
      set({ error: String(error) });
    }
  },

  updateCollection: async (id: string, name: string, color: string) => {
    try {
      await invoke('update_collection', { id, name, color });
      await get().loadCollections();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  createTag: async (name: string) => {
    const tag = await invoke<Tag>('create_tag', { name });
    await get().loadTags();
    return tag;
  },

  addTagToItem: async (itemId: string, tagId: string) => {
    try {
      await invoke('add_tag_to_item', { itemId, tagId });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  removeTagFromItem: async (itemId: string, tagId: string) => {
    try {
      await invoke('remove_tag_from_item', { itemId, tagId });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setExpiration: async (itemId: string, expiresAt: string | null) => {
    try {
      await invoke('set_expiration', { itemId, expiresAt });
      await get().loadItems();
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setupListeners: async () => {
    const unlistenClipboard = await listen<ClipboardItem>('clipboard-changed', () => {
      get().loadItems();
      get().loadPinnedItems();
    });

    return () => {
      unlistenClipboard();
    };
  },
}));
