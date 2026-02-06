mod clipboard;
mod collections;
mod database;
mod exclusions;
mod hotkey;
mod keyboard;
mod qrcode;
mod settings;
mod window;

use clipboard::ClipboardMonitor;
use database::Database;
use hotkey::HotkeyManager;
use settings::SettingsManager;

#[cfg(target_os = "macos")]
use window::{set_window_blur, HotkeyModeState, PanelHideGuard, PreviousAppState, WebviewWindowExt, MAIN_WINDOW_LABEL};

#[cfg(not(target_os = "macos"))]
use window::HotkeyModeState;

use window::SelectedItemState;

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg(target_os = "macos")]
use tauri::{ActivationPolicy, Emitter};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build());

    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    builder
        .setup(|app| {
            // Hide dock icon on macOS (makes it a menu bar only app)
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            // Get app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Initialize database
            let db =
                Database::new(app_data_dir.clone()).expect("Failed to initialize database");
            app.manage(db);

            // Initialize settings
            let settings_manager = SettingsManager::new(app_data_dir);
            let settings = settings_manager.get();
            app.manage(settings_manager);

            // Initialize hotkey manager
            let hotkey_manager = HotkeyManager::new();
            let _ = hotkey_manager.register(&app.handle(), &settings.hotkey);
            app.manage(hotkey_manager);

            // Initialize clipboard monitor
            let clipboard_monitor = ClipboardMonitor::new();
            if let Some(db) = app.try_state::<Database>() {
                clipboard_monitor.init_last_hash(&db);
            }
            app.manage(clipboard_monitor);

            // Initialize previous app state tracker (for restoring focus after hiding)
            #[cfg(target_os = "macos")]
            app.manage(PreviousAppState::new());

            // Initialize panel hide guard (prevents re-entrant order_out)
            #[cfg(target_os = "macos")]
            app.manage(PanelHideGuard::new());

            // Initialize hotkey mode state (for preventing auto-hide while modifiers held)
            app.manage(HotkeyModeState::new());

            // Initialize selected item state (for hotkey mode paste on modifier release)
            app.manage(SelectedItemState::new());

            // Start modifier key polling for hotkey mode paste-on-release (macOS)
            #[cfg(target_os = "macos")]
            {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    extern "C" {
                        fn CGEventSourceFlagsState(stateID: u32) -> u64;
                        fn CGEventSourceKeyState(stateID: u32, key: u16) -> bool;
                    }

                    // kCGEventFlagMaskCommand and kCGEventFlagMaskShift
                    const MASK_COMMAND: u64 = 0x100000;
                    const MASK_SHIFT: u64 = 0x20000;

                    // macOS virtual key codes
                    const VK_ESCAPE: u16 = 53;
                    const VK_V: u16 = 9;

                    let mut was_active = false;
                    let mut v_was_pressed = false;

                    loop {
                        // Poll every 30ms - fast enough to feel instant
                        std::thread::sleep(std::time::Duration::from_millis(30));

                        // Only check when hotkey mode is active
                        let is_active = app_handle
                            .try_state::<HotkeyModeState>()
                            .map_or(false, |s| s.is_active());

                        // Unregister global shortcut when hotkey mode enters
                        // so V keydown events aren't consumed by the shortcut system
                        if is_active && !was_active {
                            v_was_pressed = true; // V is held from activation
                            if let Some(hotkey_mgr) =
                                app_handle.try_state::<HotkeyManager>()
                            {
                                let _ = hotkey_mgr.unregister(&app_handle);
                            }
                        }

                        // Re-register global shortcut when hotkey mode exits
                        if !is_active && was_active {
                            v_was_pressed = false;
                            if let Some(hotkey_mgr) =
                                app_handle.try_state::<HotkeyManager>()
                            {
                                if let Some(settings_mgr) =
                                    app_handle.try_state::<SettingsManager>()
                                {
                                    let hotkey = settings_mgr.get().hotkey.clone();
                                    let _ = hotkey_mgr.register(&app_handle, &hotkey);
                                }
                            }
                        }
                        was_active = is_active;

                        if !is_active {
                            continue;
                        }

                        // Check ESC key - cancel hotkey mode without pasting
                        // This works regardless of which modifiers are held
                        let esc_pressed = unsafe {
                            CGEventSourceKeyState(1, VK_ESCAPE)
                        };

                        // Also detect V key for cycling (edge-detect: only on new press)
                        // Try both HID state (1) and combined session state (0)
                        let v_pressed = unsafe {
                            CGEventSourceKeyState(1, VK_V)
                            || CGEventSourceKeyState(0, VK_V)
                        };
                        if v_pressed && !v_was_pressed {
                            let _ = app_handle.emit("hotkey-cycle", ());
                        }
                        v_was_pressed = v_pressed;
                        if esc_pressed {
                            if let Some(hotkey_state) =
                                app_handle.try_state::<HotkeyModeState>()
                            {
                                hotkey_state.exit();
                            }
                            // Clear selected item to prevent paste
                            if let Some(selected_state) =
                                app_handle.try_state::<SelectedItemState>()
                            {
                                selected_state.take();
                            }
                            let app = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = crate::window::hide_window(app).await;
                            });
                            continue;
                        }

                        // Query physical modifier key state from HID system
                        let (cmd_held, shift_held) = unsafe {
                            // 1 = kCGEventSourceStateHIDSystemState (physical keys)
                            let flags = CGEventSourceFlagsState(1);
                            (flags & MASK_COMMAND != 0, flags & MASK_SHIFT != 0)
                        };

                        if !cmd_held && !shift_held {
                            // Brief delay to allow ESC to cancel
                            std::thread::sleep(std::time::Duration::from_millis(50));

                            // Check ESC one more time after grace period
                            let esc_after = unsafe {
                                CGEventSourceKeyState(1, VK_ESCAPE)
                            };

                            // All modifiers released - re-check after delay
                            if let Some(hotkey_state) =
                                app_handle.try_state::<HotkeyModeState>()
                            {
                                if hotkey_state.is_active() && !esc_after {
                                    // Exit hotkey mode immediately to prevent re-entrance
                                    hotkey_state.exit();

                                    if let Some(selected_state) =
                                        app_handle.try_state::<SelectedItemState>()
                                    {
                                        if let Some(item_id) = selected_state.take() {
                                            let app = app_handle.clone();
                                            tauri::async_runtime::spawn(async move {
                                                if let Err(e) =
                                                    crate::clipboard::do_paste_and_simulate(
                                                        app, item_id,
                                                    )
                                                    .await
                                                {
                                                    log::warn!("Failed to paste on modifier release: {}", e);
                                                }
                                            });
                                        } else {
                                            // No selected item, just hide
                                            let app = app_handle.clone();
                                            tauri::async_runtime::spawn(async move {
                                                let _ =
                                                    crate::window::hide_window(app).await;
                                            });
                                        }
                                    }
                                } else if esc_after && hotkey_state.is_active() {
                                    // ESC pressed during grace period - cancel
                                    hotkey_state.exit();
                                    if let Some(selected_state) =
                                        app_handle.try_state::<SelectedItemState>()
                                    {
                                        selected_state.take();
                                    }
                                    let app = app_handle.clone();
                                    tauri::async_runtime::spawn(async move {
                                        let _ = crate::window::hide_window(app).await;
                                    });
                                }
                            }
                        }
                    }
                });
            }

            // Setup window as NSPanel on macOS
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                    // Convert to panel
                    if let Err(e) = window.to_yoink_panel() {
                        log::warn!("Failed to initialize panel: {:?}", e);
                    } else {
                        log::info!("NSPanel initialized successfully");

                        // Apply vibrancy
                        if let Err(e) = set_window_blur(&window, true) {
                            log::warn!("Failed to apply vibrancy: {:?}", e);
                        } else {
                            log::info!("Vibrancy applied");
                        }
                    }
                }
            }

            // Setup system tray
            setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Clipboard commands
            clipboard::check_clipboard,
            clipboard::get_clipboard_items,
            clipboard::get_pinned_items,
            clipboard::delete_clipboard_item,
            clipboard::pin_item,
            clipboard::unpin_item,
            clipboard::clear_history,
            clipboard::paste_item,
            clipboard::paste_and_simulate,
            clipboard::move_to_collection,
            clipboard::set_expiration,
            // Window commands
            window::show_window,
            window::hide_window,
            window::toggle_window,
            window::is_window_visible,
            window::enter_hotkey_mode,
            window::exit_hotkey_mode,
            window::set_selected_item,
            // Settings commands
            settings::get_settings,
            settings::update_settings,
            settings::set_hotkey,
            settings::set_theme,
            settings::set_accent_color,
            settings::add_excluded_app,
            settings::remove_excluded_app,
            settings::toggle_queue_mode,
            // Hotkey commands
            hotkey::register_hotkey,
            hotkey::validate_hotkey,
            // Exclusions commands
            exclusions::get_current_app,
            exclusions::check_app_excluded,
            // Collections commands
            collections::create_collection,
            collections::get_collections,
            collections::delete_collection,
            collections::update_collection,
            collections::create_tag,
            collections::get_tags,
            collections::add_tag_to_item,
            collections::remove_tag_from_item,
            collections::get_item_tags,
            // QR code command
            qrcode::generate_qr_code,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let open_item = MenuItemBuilder::with_id("open", "Open Yoink").build(app)?;
    let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
    let upgrade_item = MenuItemBuilder::with_id("upgrade", "Upgrade to Pro").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&open_item)
        .separator()
        .item(&settings_item)
        .item(&upgrade_item)
        .separator()
        .item(&quit_item)
        .build()?;

    // Load tray icon from file
    let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
        .expect("Failed to load tray icon");

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = window::show_window(app).await;
                });
            }
            "settings" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = window::show_window(app.clone()).await;
                    // Small delay to ensure window is visible and webview is ready
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    // Use eval to directly trigger settings - more reliable for NSPanel
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.eval(
                            "window.__openSettings && window.__openSettings()"
                        );
                    }
                });
            }
            "upgrade" => {
                #[allow(deprecated)]
                let _ = tauri_plugin_shell::ShellExt::shell(app)
                    .open("https://yoink.app/upgrade", None);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
