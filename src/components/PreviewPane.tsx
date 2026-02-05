import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useClipboardStore } from '@/stores/clipboardStore';
import clsx from 'clsx';

export function PreviewPane() {
  const { items, selectedIndex, pasteItem, pinItem, unpinItem, deleteItem } =
    useClipboardStore();
  const [isExpanded, setIsExpanded] = useState(false);
  const [qrCode, setQrCode] = useState<string | null>(null);

  const item = items[selectedIndex];

  if (!item) {
    return null;
  }

  const handleGenerateQR = async () => {
    try {
      const svg = await invoke<string>('generate_qr_code', { content: item.content });
      setQrCode(svg);
    } catch (error) {
      console.error('Failed to generate QR code:', error);
    }
  };

  const handleCopyPlainText = async () => {
    // Copy content without any formatting
    navigator.clipboard.writeText(item.content);
  };

  const handleOpenUrl = () => {
    if (item.content_type === 'url') {
      window.open(item.content, '_blank');
    }
  };

  const isLongContent = item.content.length > 500;
  const displayContent = isExpanded ? item.content : item.content.slice(0, 500);

  return (
    <div className="border-t border-[var(--border-color)] bg-[var(--bg-secondary)]">
      {/* Preview content */}
      <div className="p-4 max-h-48 overflow-auto">
        {item.content_type === 'image' ? (
          <div className="flex items-center justify-center">
            <div className="text-sm text-[var(--text-secondary)]">
              {item.preview}
            </div>
          </div>
        ) : (
          <pre
            className={clsx(
              'text-sm text-[var(--text-primary)] whitespace-pre-wrap break-words',
              'font-mono'
            )}
          >
            {displayContent}
            {isLongContent && !isExpanded && '...'}
          </pre>
        )}

        {isLongContent && (
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="mt-2 text-xs text-accent-500 hover:text-accent-600"
          >
            {isExpanded ? 'Show less' : 'Show more'}
          </button>
        )}
      </div>

      {/* QR Code modal */}
      {qrCode && (
        <div
          className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
          onClick={() => setQrCode(null)}
        >
          <div
            className="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div
              className="w-64 h-64"
              dangerouslySetInnerHTML={{ __html: qrCode }}
            />
            <button
              onClick={() => setQrCode(null)}
              className="mt-4 w-full py-2 px-4 bg-accent-500 text-white rounded-lg hover:bg-accent-600"
            >
              Close
            </button>
          </div>
        </div>
      )}

      {/* Actions bar */}
      <div className="flex items-center justify-between px-4 py-2 border-t border-[var(--border-color)]">
        <div className="flex items-center gap-1">
          <ActionButton
            onClick={() => pasteItem(item.id)}
            icon={
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
            }
            label="Paste"
            shortcut="Enter"
          />
          <ActionButton
            onClick={() => (item.is_pinned ? unpinItem(item.id) : pinItem(item.id))}
            icon={
              <svg
                className={clsx('w-4 h-4', item.is_pinned && 'text-accent-500')}
                fill={item.is_pinned ? 'currentColor' : 'none'}
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
              </svg>
            }
            label={item.is_pinned ? 'Unpin' : 'Pin'}
            shortcut="Cmd+P"
          />
          <ActionButton
            onClick={() => deleteItem(item.id)}
            icon={
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            }
            label="Delete"
            shortcut="Del"
          />
        </div>

        <div className="flex items-center gap-1">
          <ActionButton onClick={handleCopyPlainText} label="Plain text" small />
          <ActionButton onClick={handleGenerateQR} label="QR code" small />
          {item.content_type === 'url' && (
            <ActionButton onClick={handleOpenUrl} label="Open URL" small />
          )}
        </div>
      </div>
    </div>
  );
}

interface ActionButtonProps {
  onClick: () => void;
  icon?: React.ReactNode;
  label: string;
  shortcut?: string;
  small?: boolean;
}

function ActionButton({ onClick, icon, label, shortcut, small }: ActionButtonProps) {
  return (
    <button
      onClick={onClick}
      className={clsx(
        'flex items-center gap-1.5 rounded-lg transition-colors',
        'hover:bg-[var(--bg-tertiary)]',
        small ? 'px-2 py-1 text-xs' : 'px-2.5 py-1.5 text-sm'
      )}
      title={shortcut ? `${label} (${shortcut})` : label}
    >
      {icon}
      <span className="text-[var(--text-secondary)]">{label}</span>
      {shortcut && (
        <span className="text-[var(--text-tertiary)] text-xs ml-1">{shortcut}</span>
      )}
    </button>
  );
}
