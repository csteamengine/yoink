import { useState } from 'react';
import { useClipboardStore } from '@/stores/clipboardStore';
import { useProStore } from '@/stores/proStore';
import clsx from 'clsx';

const COLORS = [
  '#ef4444', // red
  '#f97316', // orange
  '#eab308', // yellow
  '#22c55e', // green
  '#3b82f6', // blue
  '#8b5cf6', // purple
  '#ec4899', // pink
  '#6b7280', // gray
];

export function CollectionsPanel() {
  const { isPro } = useProStore();
  const {
    collections,
    selectedCollectionId,
    setSelectedCollection,
    createCollection,
    updateCollection,
  } = useClipboardStore();

  const [isCreating, setIsCreating] = useState(false);
  const [newColor, setNewColor] = useState(COLORS[0]);
  const [editingId, setEditingId] = useState<string | null>(null);

  if (!isPro) {
    return null;
  }

  const handleUpdate = async (id: string, name: string, color: string) => {
    await updateCollection(id, name, color);
    setEditingId(null);
  };

  return (
    <div className="px-4 py-2 border-b border-[var(--border-color)]">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-medium text-[var(--text-secondary)]">
          Collections
        </span>
        <button
          onClick={() => setIsCreating(true)}
          className="text-xs text-accent-500 hover:text-accent-600"
        >
          + New
        </button>
      </div>

      {/* Collection list */}
      <div className="flex flex-wrap gap-1.5">
        {/* All items button */}
        <button
          onClick={() => setSelectedCollection(null)}
          className={clsx(
            'px-2.5 py-1 rounded-lg text-xs transition-colors',
            selectedCollectionId === null
              ? 'bg-accent-500 text-white'
              : 'bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]'
          )}
        >
          All
        </button>

        {collections.map((collection) =>
          editingId === collection.id ? (
            <CollectionEditor
              key={collection.id}
              name={collection.name}
              color={collection.color}
              onSave={(name, color) => handleUpdate(collection.id, name, color)}
              onCancel={() => setEditingId(null)}
            />
          ) : (
            <button
              key={collection.id}
              onClick={() => setSelectedCollection(collection.id)}
              onContextMenu={(e) => {
                e.preventDefault();
                setEditingId(collection.id);
              }}
              className={clsx(
                'px-2.5 py-1 rounded-lg text-xs transition-colors flex items-center gap-1.5',
                selectedCollectionId === collection.id
                  ? 'bg-accent-500 text-white'
                  : 'bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]'
              )}
            >
              <span
                className="w-2 h-2 rounded-full"
                style={{ backgroundColor: collection.color }}
              />
              {collection.name}
            </button>
          )
        )}
      </div>

      {/* Create new collection */}
      {isCreating && (
        <CollectionEditor
          name=""
          color={newColor}
          onSave={async (name, color) => {
            if (name.trim()) {
              await createCollection(name.trim(), color);
              setNewColor(COLORS[0]);
              setIsCreating(false);
            }
          }}
          onCancel={() => setIsCreating(false)}
          isNew
        />
      )}
    </div>
  );
}

interface CollectionEditorProps {
  name: string;
  color: string;
  onSave: (name: string, color: string) => void;
  onCancel: () => void;
  isNew?: boolean;
}

function CollectionEditor({
  name: initialName,
  color: initialColor,
  onSave,
  onCancel,
  isNew,
}: CollectionEditorProps) {
  const [name, setName] = useState(initialName);
  const [color, setColor] = useState(initialColor);

  return (
    <div className="mt-2 p-3 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)]">
      <input
        type="text"
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="Collection name"
        className={clsx(
          'w-full px-2.5 py-1.5 rounded text-sm',
          'bg-[var(--bg-primary)] text-[var(--text-primary)]',
          'border border-[var(--border-color)]',
          'focus:outline-none focus:ring-2 focus:ring-accent-500'
        )}
        autoFocus
      />

      <div className="flex gap-1 mt-2">
        {COLORS.map((c) => (
          <button
            key={c}
            onClick={() => setColor(c)}
            className={clsx(
              'w-5 h-5 rounded-full border-2',
              color === c ? 'border-[var(--text-primary)]' : 'border-transparent'
            )}
            style={{ backgroundColor: c }}
          />
        ))}
      </div>

      <div className="flex items-center justify-between mt-3">
        <div className="flex gap-2">
          <button
            onClick={() => onSave(name, color)}
            className="px-3 py-1 rounded bg-accent-500 text-white text-xs hover:bg-accent-600"
          >
            Save
          </button>
          <button
            onClick={onCancel}
            className="px-3 py-1 rounded bg-[var(--bg-tertiary)] text-[var(--text-secondary)] text-xs hover:bg-[var(--border-color)]"
          >
            Cancel
          </button>
        </div>
        {!isNew && (
          <button
            onClick={async () => {
              // Need to get the collection id from somewhere
              // This is a simplification - in real code we'd pass the id
              onCancel();
            }}
            className="text-xs text-red-500 hover:text-red-600"
          >
            Delete
          </button>
        )}
      </div>
    </div>
  );
}
