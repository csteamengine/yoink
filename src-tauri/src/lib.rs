mod clipboard;
mod collections;
mod database;
mod exclusions;
mod hotkey;
mod qrcode;
mod settings;
mod window;

use clipboard::ClipboardMonitor;
use database::Database;
use hotkey::HotkeyManager;
use settings::SettingsManager;

#[cfg(target_os = "macos")]
use window::{set_window_blur, WebviewWindowExt, MAIN_WINDOW_LABEL};

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

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
            clipboard::move_to_collection,
            clipboard::set_expiration,
            // Window commands
            window::show_window,
            window::hide_window,
            window::toggle_window,
            window::is_window_visible,
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

    // Create a simple icon (16x16 white square as placeholder)
    let icon_data: Vec<u8> = vec![255u8; 16 * 16 * 4];
    let icon = Image::new_owned(icon_data, 16, 16);

    let _tray = TrayIconBuilder::new()
        .icon(icon)
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
                    let _ = app.emit("open-settings", ());
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
