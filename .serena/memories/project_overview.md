# Yoink - Clipboard Manager

## Purpose
macOS clipboard manager desktop app (Flycut/Alfred-style). Stores clipboard history, supports pinned items, collections, queue mode, and hotkey-triggered paste.

## Tech Stack
- **Frontend**: React 19 + TypeScript, Zustand state management, TailwindCSS, Vite
- **Backend**: Tauri 2 (Rust), SQLite (rusqlite), NSPanel for floating window
- **macOS-specific**: tauri-nspanel (floating panel), cocoa/objc crates, core-graphics for keyboard simulation

## Key Architecture
- Window is an NSPanel (floating, can appear over all spaces)
- App runs as Accessory (no dock icon, tray-only)
- Hotkey mode: user holds Cmd+Shift+V to open, can cycle with V, releases modifiers to paste
- `PreviousAppState` captures frontmost app before showing panel, restores focus on hide
- `HotkeyModeState` prevents panel auto-hide while modifiers are held
- Backend polls `CGEventSourceFlagsState` to detect modifier release (NSPanel doesn't reliably deliver keyup events)

## Project Structure
- `src/` - React frontend (components, stores, hooks, lib)
- `src-tauri/src/` - Rust backend (window.rs, hotkey.rs, keyboard.rs, clipboard.rs, etc.)
- Key stores: clipboardStore, hotkeyModeStore, settingsStore, queueStore

## Commands
- `pnpm dev` / `npm run dev` - Vite dev server
- `pnpm tauri dev` - Full Tauri dev (frontend + backend)
- `cargo check --manifest-path src-tauri/Cargo.toml` - Rust type check
- `npx tsc --noEmit` - TypeScript type check
