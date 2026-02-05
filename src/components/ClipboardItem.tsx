import React from 'react';
import { formatDistanceToNow } from 'date-fns';
import clsx from 'clsx';
import type { ClipboardItem as ClipboardItemType } from '@/stores/clipboardStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useQueueStore } from '@/stores/queueStore';

interface ClipboardItemProps {
  item: ClipboardItemType;
  index: number;
  isSelected: boolean;
  onSelect: () => void;
  onPaste: () => void;
}

const contentTypeIcons: Record<string, React.ReactElement> = {
  text: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h7" />
    </svg>
  ),
  code: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
    </svg>
  ),
  url: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
    </svg>
  ),
  file: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
    </svg>
  ),
  files: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
    </svg>
  ),
  image: (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
    </svg>
  ),
};

export function ClipboardItem({ item, index, isSelected, onSelect, onPaste }: ClipboardItemProps) {
  const { settings } = useSettingsStore();
  const { queue, addToQueue, removeFromQueue, isActive } = useQueueStore();

  const isInQueue = queue.some((i) => i.id === item.id);
  const queuePosition = queue.findIndex((i) => i.id === item.id) + 1;

  const icon = contentTypeIcons[item.content_type] || contentTypeIcons.text;

  const truncatedPreview =
    item.preview.length > 100 ? item.preview.slice(0, 100) + '...' : item.preview;

  const timestamp = formatDistanceToNow(new Date(item.created_at), { addSuffix: true });

  const handleQueueToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (isInQueue) {
      removeFromQueue(item.id);
    } else {
      addToQueue(item);
    }
  };

  return (
    <div
      className={clsx(
        'px-4 py-2.5 cursor-pointer transition-colors duration-100',
        'border-b border-[var(--border-color)] last:border-b-0',
        isSelected
          ? 'bg-accent-100 dark:bg-accent-900/30'
          : 'hover:bg-[var(--bg-secondary)]'
      )}
      onClick={onSelect}
      onDoubleClick={onPaste}
    >
      <div className="flex items-start gap-3">
        {/* Icon */}
        <div
          className={clsx(
            'flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center',
            'bg-[var(--bg-tertiary)] text-[var(--text-secondary)]'
          )}
        >
          {icon}
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <p
            className={clsx(
              'text-sm text-[var(--text-primary)] truncate',
              'whitespace-pre-wrap break-words line-clamp-2'
            )}
          >
            {truncatedPreview}
          </p>

          {/* Metadata row */}
          <div className="flex items-center gap-2 mt-1">
            {settings.show_timestamps && (
              <span className="text-xs text-[var(--text-tertiary)]">{timestamp}</span>
            )}
            {item.content_type !== 'text' && (
              <span className="text-xs text-[var(--text-tertiary)] capitalize">
                {item.content_type}
              </span>
            )}
          </div>
        </div>

        {/* Right side indicators */}
        <div className="flex items-center gap-1.5 flex-shrink-0">
          {/* Keyboard shortcut hint */}
          {index < 9 && (
            <span
              className={clsx(
                'w-5 h-5 rounded text-xs flex items-center justify-center',
                'bg-[var(--bg-tertiary)] text-[var(--text-tertiary)]',
                'font-mono'
              )}
            >
              {index + 1}
            </span>
          )}

          {/* Queue indicator */}
          {isActive && (
            <button
              onClick={handleQueueToggle}
              className={clsx(
                'w-5 h-5 rounded text-xs flex items-center justify-center',
                'transition-colors',
                isInQueue
                  ? 'bg-accent-500 text-white'
                  : 'bg-[var(--bg-tertiary)] text-[var(--text-tertiary)] hover:bg-accent-200'
              )}
            >
              {isInQueue ? queuePosition : '+'}
            </button>
          )}

          {/* Pin indicator */}
          {item.is_pinned && (
            <svg
              className="w-4 h-4 text-accent-500"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
            </svg>
          )}
        </div>
      </div>
    </div>
  );
}
