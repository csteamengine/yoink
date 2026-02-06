import { useState } from 'react';
import { useSettingsStore } from '@/stores/settingsStore';
import { useProStore } from '@/stores/proStore';
import { useClipboardStore } from '@/stores/clipboardStore';
import clsx from 'clsx';

const TABS = [
  { id: 'general', label: 'General' },
  { id: 'appearance', label: 'Appearance' },
  { id: 'pro', label: 'Pro Features' },
  { id: 'account', label: 'Account' },
  { id: 'about', label: 'About' },
];

export function SettingsPanel() {
  const { isSettingsOpen, closeSettings, activeTab, setActiveTab } =
    useSettingsStore();
  const { isPro } = useProStore();

  if (!isSettingsOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 animate-fade-in">
      <div
        className={clsx(
          'w-[600px] max-h-[500px] rounded-xl overflow-hidden',
          'bg-[var(--bg-primary)] shadow-xl',
          'flex flex-col animate-slide-up'
        )}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[var(--border-color)]">
          <h2 className="text-lg font-semibold text-[var(--text-primary)]">Settings</h2>
          <button
            onClick={closeSettings}
            className="p-1.5 rounded-lg hover:bg-[var(--bg-secondary)] transition-colors"
          >
            <svg className="w-5 h-5 text-[var(--text-secondary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <div className="w-40 border-r border-[var(--border-color)] py-2">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={clsx(
                  'w-full px-4 py-2 text-left text-sm transition-colors',
                  activeTab === tab.id
                    ? 'bg-accent-100 dark:bg-accent-900/30 text-accent-700 dark:text-accent-300 font-medium'
                    : 'text-[var(--text-secondary)] hover:bg-[var(--bg-secondary)]'
                )}
              >
                {tab.label}
                {tab.id === 'pro' && !isPro && (
                  <span className="ml-1 text-xs text-accent-500">Pro</span>
                )}
              </button>
            ))}
          </div>

          {/* Tab content */}
          <div className="flex-1 p-6 overflow-auto">
            {activeTab === 'general' && <GeneralTab />}
            {activeTab === 'appearance' && <AppearanceTab />}
            {activeTab === 'pro' && <ProFeaturesTab />}
            {activeTab === 'account' && <AccountTab />}
            {activeTab === 'about' && <AboutTab />}
          </div>
        </div>
      </div>
    </div>
  );
}

function GeneralTab() {
  const { settings, setHotkey, updateSettings } = useSettingsStore();
  const { clearHistory } = useClipboardStore();
  const [hotkeyInput, setHotkeyInput] = useState(settings.hotkey);

  const handleHotkeyChange = async () => {
    await setHotkey(hotkeyInput);
  };

  return (
    <div className="space-y-6">
      <SettingRow
        label="Global Hotkey"
        description="Keyboard shortcut to open Yoink"
      >
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={hotkeyInput}
            onChange={(e) => setHotkeyInput(e.target.value)}
            className={clsx(
              'px-3 py-1.5 rounded-lg w-48',
              'bg-[var(--bg-secondary)] text-[var(--text-primary)]',
              'border border-[var(--border-color)]',
              'focus:outline-none focus:ring-2 focus:ring-accent-500'
            )}
            placeholder="Cmd+Shift+V"
          />
          <button
            onClick={handleHotkeyChange}
            className="px-3 py-1.5 rounded-lg bg-accent-500 text-white text-sm hover:bg-accent-600"
          >
            Set
          </button>
        </div>
      </SettingRow>

      <SettingRow
        label="Launch at startup"
        description="Automatically start Yoink when you log in"
      >
        <Toggle
          checked={settings.launch_at_startup}
          onChange={(checked) => updateSettings({ launch_at_startup: checked })}
        />
      </SettingRow>

      <SettingRow
        label="History limit"
        description="Maximum number of clipboard items to store"
      >
        <select
          value={settings.history_limit}
          onChange={(e) =>
            updateSettings({ history_limit: parseInt(e.target.value) })
          }
          className={clsx(
            'px-3 py-1.5 rounded-lg',
            'bg-[var(--bg-secondary)] text-[var(--text-primary)]',
            'border border-[var(--border-color)]',
            'focus:outline-none focus:ring-2 focus:ring-accent-500'
          )}
        >
          <option value={50}>50 items</option>
          <option value={100}>100 items</option>
          <option value={200}>200 items</option>
          <option value={500}>500 items</option>
        </select>
      </SettingRow>

      <SettingRow
        label="Auto-paste"
        description="Automatically paste into the previous app when selecting an item"
      >
        <Toggle
          checked={settings.auto_paste}
          onChange={(checked) => updateSettings({ auto_paste: checked })}
        />
      </SettingRow>

      <SettingRow
        label="Sticky mode"
        description="Keep window open until manually closed (shows search bar)"
      >
        <Toggle
          checked={settings.sticky_mode}
          onChange={(checked) => updateSettings({ sticky_mode: checked })}
        />
      </SettingRow>

      <SettingRow
        label="Clear clipboard history"
        description="Delete all non-pinned clipboard items"
      >
        <button
          onClick={clearHistory}
          className="px-3 py-1.5 rounded-lg bg-red-500 text-white text-sm hover:bg-red-600"
        >
          Clear History
        </button>
      </SettingRow>
    </div>
  );
}

function AppearanceTab() {
  const { settings, setTheme, setAccentColor, updateSettings } = useSettingsStore();

  return (
    <div className="space-y-6">
      <SettingRow label="Theme" description="Choose your preferred color scheme">
        <div className="flex gap-2">
          {(['light', 'dark', 'system'] as const).map((theme) => (
            <button
              key={theme}
              onClick={() => setTheme(theme)}
              className={clsx(
                'px-3 py-1.5 rounded-lg text-sm capitalize',
                settings.theme === theme
                  ? 'bg-accent-500 text-white'
                  : 'bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)]'
              )}
            >
              {theme}
            </button>
          ))}
        </div>
      </SettingRow>

      <SettingRow label="Accent color" description="Primary color for highlights">
        <div className="flex gap-2">
          {(['blue', 'purple', 'green'] as const).map((color) => (
            <button
              key={color}
              onClick={() => setAccentColor(color)}
              className={clsx(
                'w-8 h-8 rounded-full border-2',
                settings.accent_color === color
                  ? 'border-[var(--text-primary)]'
                  : 'border-transparent',
                color === 'blue' && 'bg-blue-500',
                color === 'purple' && 'bg-purple-500',
                color === 'green' && 'bg-green-500'
              )}
              title={color}
            />
          ))}
        </div>
      </SettingRow>

      <SettingRow label="Font size" description="Text size in the clipboard list">
        <select
          value={settings.font_size}
          onChange={(e) =>
            updateSettings({ font_size: parseInt(e.target.value) })
          }
          className={clsx(
            'px-3 py-1.5 rounded-lg',
            'bg-[var(--bg-secondary)] text-[var(--text-primary)]',
            'border border-[var(--border-color)]',
            'focus:outline-none focus:ring-2 focus:ring-accent-500'
          )}
        >
          <option value={12}>Small (12px)</option>
          <option value={14}>Medium (14px)</option>
          <option value={16}>Large (16px)</option>
        </select>
      </SettingRow>

      <SettingRow
        label="Show timestamps"
        description="Display relative time for clipboard items"
      >
        <Toggle
          checked={settings.show_timestamps}
          onChange={(checked) => updateSettings({ show_timestamps: checked })}
        />
      </SettingRow>
    </div>
  );
}

function ProFeaturesTab() {
  const { isPro, openUpgrade } = useProStore();

  if (!isPro) {
    return (
      <div className="flex flex-col items-center justify-center py-8">
        <div className="w-16 h-16 rounded-full bg-accent-100 dark:bg-accent-900/30 flex items-center justify-center mb-4">
          <svg className="w-8 h-8 text-accent-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
          </svg>
        </div>
        <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
          Upgrade to Pro
        </h3>
        <p className="text-sm text-[var(--text-secondary)] text-center mb-6 max-w-sm">
          Unlock collections, tags, queue mode, app exclusions, scheduled expiration,
          and more for just $4.99 (one-time payment).
        </p>
        <button
          onClick={openUpgrade}
          className="px-6 py-2.5 rounded-lg bg-accent-500 text-white font-medium hover:bg-accent-600 transition-colors"
        >
          Upgrade for $4.99
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SettingRow
        label="Collections"
        description="Organize clips into custom collections"
      >
        <span className="text-sm text-green-500">Enabled</span>
      </SettingRow>

      <SettingRow
        label="Tags"
        description="Add tags to clips for easy filtering"
      >
        <span className="text-sm text-green-500">Enabled</span>
      </SettingRow>

      <SettingRow
        label="Queue Mode"
        description="Paste multiple items in sequence"
      >
        <span className="text-sm text-green-500">Enabled</span>
      </SettingRow>

      <SettingRow
        label="App Exclusions"
        description="Prevent capturing from specific apps"
      >
        <span className="text-sm text-green-500">Enabled</span>
      </SettingRow>
    </div>
  );
}

function AccountTab() {
  const { user, isAuthenticated, isLoading, login, logout } = useProStore();
  const [email, setEmail] = useState('');

  const handleLogin = async () => {
    if (email) {
      await login(email);
    }
  };

  if (isAuthenticated && user) {
    return (
      <div className="space-y-6">
        <div className="flex items-center gap-4 p-4 rounded-lg bg-[var(--bg-secondary)]">
          <div className="w-12 h-12 rounded-full bg-accent-500 flex items-center justify-center text-white font-semibold text-lg">
            {user.email[0].toUpperCase()}
          </div>
          <div>
            <p className="text-sm font-medium text-[var(--text-primary)]">{user.email}</p>
            {user.is_pro && (
              <span className="text-xs text-accent-500 font-medium">Pro</span>
            )}
          </div>
        </div>

        <button
          onClick={logout}
          disabled={isLoading}
          className="px-4 py-2 rounded-lg bg-red-500 text-white text-sm hover:bg-red-600 disabled:opacity-50"
        >
          {isLoading ? 'Logging out...' : 'Log out'}
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <p className="text-sm text-[var(--text-secondary)]">
        Sign in to sync your Pro license across devices.
      </p>

      <div className="space-y-3">
        <input
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="Enter your email"
          className={clsx(
            'w-full px-4 py-2.5 rounded-lg',
            'bg-[var(--bg-secondary)] text-[var(--text-primary)]',
            'border border-[var(--border-color)]',
            'placeholder-[var(--text-tertiary)]',
            'focus:outline-none focus:ring-2 focus:ring-accent-500'
          )}
        />
        <button
          onClick={handleLogin}
          disabled={isLoading || !email}
          className={clsx(
            'w-full py-2.5 rounded-lg font-medium transition-colors',
            'bg-accent-500 text-white hover:bg-accent-600',
            'disabled:opacity-50 disabled:cursor-not-allowed'
          )}
        >
          {isLoading ? 'Sending...' : 'Send Magic Link'}
        </button>
      </div>

      <p className="text-xs text-[var(--text-tertiary)]">
        We'll send you a magic link to sign in. No password needed.
      </p>
    </div>
  );
}

function AboutTab() {
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <div className="w-16 h-16 rounded-xl bg-accent-500 flex items-center justify-center">
          <span className="text-2xl font-bold text-white">Y</span>
        </div>
        <div>
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">Yoink</h3>
          <p className="text-sm text-[var(--text-secondary)]">Version 1.0.0</p>
        </div>
      </div>

      <p className="text-sm text-[var(--text-secondary)]">
        A modern clipboard manager built with Tauri, React, and Rust.
      </p>

      <div className="space-y-2">
        <a
          href="https://github.com/yoink-app/yoink"
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-2 text-sm text-accent-500 hover:text-accent-600"
        >
          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
            <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.87 8.17 6.84 9.5.5.08.66-.23.66-.5v-1.69c-2.77.6-3.36-1.34-3.36-1.34-.46-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.6.07-.6 1 .07 1.53 1.03 1.53 1.03.87 1.52 2.34 1.07 2.91.83.09-.65.35-1.09.63-1.34-2.22-.25-4.55-1.11-4.55-4.92 0-1.11.38-2 1.03-2.71-.1-.25-.45-1.29.1-2.64 0 0 .84-.27 2.75 1.02.79-.22 1.65-.33 2.5-.33.85 0 1.71.11 2.5.33 1.91-1.29 2.75-1.02 2.75-1.02.55 1.35.2 2.39.1 2.64.65.71 1.03 1.6 1.03 2.71 0 3.82-2.34 4.66-4.57 4.91.36.31.69.92.69 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12A10 10 0 0012 2z" />
          </svg>
          GitHub Repository
        </a>
      </div>
    </div>
  );
}

// Helper components
interface SettingRowProps {
  label: string;
  description: string;
  children: React.ReactNode;
}

function SettingRow({ label, description, children }: SettingRowProps) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="text-sm font-medium text-[var(--text-primary)]">{label}</p>
        <p className="text-xs text-[var(--text-tertiary)]">{description}</p>
      </div>
      {children}
    </div>
  );
}

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function Toggle({ checked, onChange }: ToggleProps) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={clsx(
        'w-11 h-6 rounded-full transition-colors relative',
        checked ? 'bg-accent-500' : 'bg-[var(--bg-tertiary)]'
      )}
    >
      <span
        className={clsx(
          'absolute top-1 w-4 h-4 rounded-full bg-white transition-transform',
          checked ? 'translate-x-6' : 'translate-x-1'
        )}
      />
    </button>
  );
}
