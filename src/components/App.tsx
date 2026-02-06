import { useEffect } from 'react';
import { SearchBar } from './SearchBar';
import { ClipboardList } from './ClipboardList';
import { PinnedSection } from './PinnedSection';
import { PreviewPane } from './PreviewPane';
import { SettingsPanel } from './SettingsPanel';
import { CollectionsPanel } from './CollectionsPanel';
import { QueueMode } from './QueueMode';
import { useClipboardMonitor } from '@/hooks/useClipboardMonitor';
import { useKeyboardNav } from '@/hooks/useKeyboardNav';
import { useGlobalHotkey } from '@/hooks/useGlobalHotkey';
import { useSettingsStore } from '@/stores/settingsStore';
import { useProStore } from '@/stores/proStore';
import { useHotkeyModeStore } from '@/stores/hotkeyModeStore';
import clsx from 'clsx';

export default function App() {
  // Initialize hooks
  useClipboardMonitor();
  useKeyboardNav();
  useGlobalHotkey();

  const { settings, loadSettings, setupListeners: setupSettingsListeners, applyTheme } =
    useSettingsStore();
  const { checkAuth } = useProStore();
  const { isHotkeyMode, setupListeners: setupHotkeyModeListeners } = useHotkeyModeStore();

  // Load initial data
  useEffect(() => {
    loadSettings();
    checkAuth();

    let cleanupSettings: (() => void) | undefined;
    let cleanupHotkeyMode: (() => void) | undefined;

    setupSettingsListeners().then((unsub) => {
      cleanupSettings = unsub;
    });

    setupHotkeyModeListeners().then((unsub) => {
      cleanupHotkeyMode = unsub;
    });

    // Apply theme immediately
    applyTheme();

    // Handle window blur to potentially hide (handled by backend)
    const handleBlur = () => {
      // Window manager handles auto-hide
    };

    window.addEventListener('blur', handleBlur);

    return () => {
      cleanupSettings?.();
      cleanupHotkeyMode?.();
      window.removeEventListener('blur', handleBlur);
    };
  }, [loadSettings, checkAuth, setupSettingsListeners, setupHotkeyModeListeners, applyTheme]);

  return (
    <div
      className={clsx(
        'h-full flex flex-col',
        'glass rounded-xl overflow-hidden',
        'border border-[var(--border-color)]',
        'shadow-xl',
        // Visual indicator for hotkey mode (Flycut-style)
        isHotkeyMode && 'ring-2 ring-[var(--accent-color)] ring-inset'
      )}
    >
      {/* Search bar - only shown in sticky mode */}
      {settings.sticky_mode && <SearchBar />}

      {/* Collections (Pro feature) */}
      <CollectionsPanel />

      {/* Pinned items */}
      <PinnedSection />

      {/* Main clipboard list */}
      <ClipboardList />

      {/* Queue mode indicator (Pro feature) */}
      <QueueMode />

      {/* Preview pane */}
      <PreviewPane />

      {/* Settings modal */}
      <SettingsPanel />
    </div>
  );
}
