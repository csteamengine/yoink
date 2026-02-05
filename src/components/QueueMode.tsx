import { useQueueStore } from '@/stores/queueStore';
import { useProStore } from '@/stores/proStore';
import clsx from 'clsx';

export function QueueMode() {
  const { isPro } = useProStore();
  const { queue, currentIndex, isActive, toggleActive, clearQueue, removeFromQueue } =
    useQueueStore();

  if (!isPro || queue.length === 0) {
    return null;
  }

  return (
    <div className="px-4 py-2 border-t border-[var(--border-color)] bg-accent-50 dark:bg-accent-900/20">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <svg
            className="w-4 h-4 text-accent-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 6h16M4 10h16M4 14h16M4 18h16"
            />
          </svg>
          <span className="text-xs font-medium text-accent-700 dark:text-accent-300">
            Queue Mode ({queue.length} items)
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={toggleActive}
            className={clsx(
              'px-2.5 py-1 rounded text-xs font-medium transition-colors',
              isActive
                ? 'bg-accent-500 text-white'
                : 'bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]'
            )}
          >
            {isActive ? 'Active' : 'Start'}
          </button>
          <button
            onClick={clearQueue}
            className="px-2.5 py-1 rounded text-xs text-red-500 hover:bg-red-100 dark:hover:bg-red-900/30"
          >
            Clear
          </button>
        </div>
      </div>

      {/* Queue items */}
      <div className="flex gap-1.5 overflow-x-auto pb-1">
        {queue.map((item, index) => {
          const truncatedPreview =
            item.preview.length > 20 ? item.preview.slice(0, 20) + '...' : item.preview;
          const isCurrent = isActive && index === currentIndex;

          return (
            <div
              key={item.id}
              className={clsx(
                'flex items-center gap-1.5 px-2 py-1 rounded',
                'bg-[var(--bg-primary)] border border-[var(--border-color)]',
                'text-xs',
                isCurrent && 'ring-2 ring-accent-500'
              )}
            >
              <span
                className={clsx(
                  'w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-medium',
                  isCurrent
                    ? 'bg-accent-500 text-white'
                    : 'bg-[var(--bg-tertiary)] text-[var(--text-tertiary)]'
                )}
              >
                {index + 1}
              </span>
              <span className="text-[var(--text-primary)] truncate max-w-[80px]">
                {truncatedPreview}
              </span>
              <button
                onClick={() => removeFromQueue(item.id)}
                className="text-[var(--text-tertiary)] hover:text-red-500"
              >
                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>
          );
        })}
      </div>

      {isActive && (
        <p className="text-xs text-accent-600 dark:text-accent-400 mt-2">
          Press Enter to paste item {currentIndex + 1} of {queue.length}
        </p>
      )}
    </div>
  );
}
