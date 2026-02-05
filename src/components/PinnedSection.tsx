import { useClipboardStore } from '@/stores/clipboardStore';
import clsx from 'clsx';

export function PinnedSection() {
  const { pinnedItems, pasteItem, unpinItem } = useClipboardStore();

  if (pinnedItems.length === 0) {
    return null;
  }

  return (
    <div className="px-4 py-2 border-b border-[var(--border-color)]">
      <div className="flex items-center gap-2 mb-2">
        <svg
          className="w-3.5 h-3.5 text-accent-500"
          fill="currentColor"
          viewBox="0 0 24 24"
        >
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
        </svg>
        <span className="text-xs font-medium text-[var(--text-secondary)]">
          Pinned ({pinnedItems.length})
        </span>
      </div>

      <div className="flex flex-wrap gap-1.5">
        {pinnedItems.slice(0, 10).map((item, index) => {
          const shortcut = index < 9 ? index + 1 : index === 9 ? 0 : null;
          const truncatedPreview =
            item.preview.length > 30 ? item.preview.slice(0, 30) + '...' : item.preview;

          return (
            <button
              key={item.id}
              onClick={() => pasteItem(item.id)}
              onContextMenu={(e) => {
                e.preventDefault();
                unpinItem(item.id);
              }}
              className={clsx(
                'group relative px-2.5 py-1.5 rounded-lg',
                'bg-[var(--bg-secondary)] hover:bg-accent-100 dark:hover:bg-accent-900/30',
                'border border-[var(--border-color)]',
                'text-xs text-[var(--text-primary)]',
                'transition-colors duration-100',
                'max-w-[150px] truncate'
              )}
              title={`${item.preview}\n\nRight-click to unpin\n${shortcut !== null ? `Cmd/Ctrl+Shift+${shortcut} to paste` : ''}`}
            >
              <span className="truncate">{truncatedPreview}</span>
              {shortcut !== null && (
                <span
                  className={clsx(
                    'absolute -top-1.5 -right-1.5',
                    'w-4 h-4 rounded-full text-[10px]',
                    'bg-accent-500 text-white',
                    'flex items-center justify-center',
                    'font-mono font-medium'
                  )}
                >
                  {shortcut}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {pinnedItems.length > 10 && (
        <p className="text-xs text-[var(--text-tertiary)] mt-2">
          +{pinnedItems.length - 10} more pinned items
        </p>
      )}
    </div>
  );
}
