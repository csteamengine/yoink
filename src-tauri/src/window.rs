use tauri::{Manager, Runtime, WebviewWindow};
#[cfg(target_os = "macos")]
use thiserror::Error;

#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelHandle, PanelLevel, StyleMask,
    WebviewWindowExt as WebviewPanelExt,
};

pub const MAIN_WINDOW_LABEL: &str = "main";

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(YoinkPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true,
        }
    })

    panel_event!(YoinkPanelEventHandler {
        window_did_become_key(notification: &NSNotification) -> (),
        window_did_resign_key(notification: &NSNotification) -> (),
    })
}

#[cfg(target_os = "macos")]
type TauriError = tauri::Error;

#[cfg(target_os = "macos")]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to convert window to panel")]
    Panel,
    #[error("Unable to find panel: {0}")]
    PanelNotFound(String),
    #[error("Monitor with cursor not found")]
    MonitorNotFound,
}

#[cfg(target_os = "macos")]
pub trait WebviewWindowExt<R: Runtime> {
    fn to_yoink_panel(&self) -> tauri::Result<PanelHandle<R>>;
    fn center_at_cursor_monitor(&self) -> tauri::Result<()>;
}

#[cfg(target_os = "macos")]
impl<R: Runtime> WebviewWindowExt<R> for WebviewWindow<R> {
    fn to_yoink_panel(&self) -> tauri::Result<PanelHandle<R>> {
        // Convert window to panel
        let panel = self
            .to_panel::<YoinkPanel<R>>()
            .map_err(|_| TauriError::Anyhow(Error::Panel.into()))?;

        // Set panel level to floating
        panel.set_level(PanelLevel::Floating.value());

        // Set collection behavior
        panel.set_collection_behavior(
            CollectionBehavior::new()
                .full_screen_auxiliary()
                .move_to_active_space()
                .transient()
                .value(),
        );

        // Non-activating panel style
        panel.set_style_mask(StyleMask::empty().nonactivating_panel().resizable().into());

        // Setup event handlers
        let handler = YoinkPanelEventHandler::new();

        handler.window_did_become_key(move |_| {
            log::info!("panel became key window");
        });

        let app_handle = self.app_handle().clone();
        handler.window_did_resign_key(move |_| {
            log::info!("panel resigned key window");

            // Hide panel when it loses focus
            if let Ok(panel) = app_handle.get_webview_panel(MAIN_WINDOW_LABEL) {
                if panel.is_visible() {
                    panel.hide();
                    let _ = app_handle.emit("panel-hidden", ());
                }
            }
        });

        panel.set_event_handler(Some(handler.as_ref()));

        Ok(panel)
    }

    fn center_at_cursor_monitor(&self) -> tauri::Result<()> {
        use tauri_nspanel::cocoa::foundation::{NSPoint, NSRect};

        let monitor = monitor::get_monitor_with_cursor()
            .ok_or(TauriError::Anyhow(Error::MonitorNotFound.into()))?;

        let monitor_scale_factor = monitor.scale_factor();
        let monitor_size = monitor.size().to_logical::<f64>(monitor_scale_factor);
        let monitor_position = monitor.position().to_logical::<f64>(monitor_scale_factor);

        let panel = self
            .get_webview_panel(self.label())
            .map_err(|_| TauriError::Anyhow(Error::PanelNotFound(self.label().into()).into()))?;

        let panel = panel.as_panel();
        let panel_frame = panel.frame();

        let rect = NSRect {
            origin: NSPoint {
                x: (monitor_position.x + (monitor_size.width / 2.0))
                    - (panel_frame.size.width / 2.0),
                y: (monitor_position.y + (monitor_size.height / 2.0))
                    - (panel_frame.size.height / 2.0)
                    + 100.0, // Slightly above center
            },
            size: panel_frame.size,
        };

        panel.setFrame_display(rect, true);

        Ok(())
    }
}

/// Apply native macOS vibrancy effect
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn set_window_blur<R: Runtime>(window: &WebviewWindow<R>, _enabled: bool) -> Result<(), String> {
    use cocoa::appkit::{NSColor, NSWindow as NSWindowTrait};
    use cocoa::base::{id, nil, NO, YES};
    use cocoa::foundation::NSRect;
    use objc::{class, msg_send, sel, sel_impl};

    let ns_window = match window.ns_window() {
        Ok(w) => w as id,
        Err(e) => return Err(e.to_string()),
    };

    if ns_window.is_null() {
        return Err("ns_window is null".to_string());
    }

    unsafe {
        // Make window transparent
        let _: () = msg_send![ns_window, setOpaque: NO];
        ns_window.setBackgroundColor_(NSColor::clearColor(nil));
        let _: () = msg_send![ns_window, setTitlebarAppearsTransparent: YES];

        let content_view: id = ns_window.contentView();
        if content_view.is_null() {
            return Err("content_view is null".to_string());
        }

        // Enable layer backing
        let _: () = msg_send![content_view, setWantsLayer: YES];
        let content_layer: id = msg_send![content_view, layer];
        if !content_layer.is_null() {
            let _: () = msg_send![content_layer, setCornerRadius: 10.0_f64];
            let _: () = msg_send![content_layer, setMasksToBounds: YES];
        }

        let bounds: NSRect = msg_send![content_view, bounds];

        // Create NSVisualEffectView
        let visual_effect_class = class!(NSVisualEffectView);
        let visual_effect_view: id = msg_send![visual_effect_class, alloc];
        let visual_effect_view: id = msg_send![visual_effect_view, initWithFrame: bounds];

        if visual_effect_view.is_null() {
            return Err("Failed to create NSVisualEffectView".to_string());
        }

        // Dark vibrancy material (9 = UltraDark)
        let _: () = msg_send![visual_effect_view, setMaterial: 9_i64];
        // State active (1)
        let _: () = msg_send![visual_effect_view, setState: 1_i64];
        // Blending mode behind window (0)
        let _: () = msg_send![visual_effect_view, setBlendingMode: 0_i64];

        // Auto-resize (width | height sizable)
        let autoresizing: u64 = 2 | 16;
        let _: () = msg_send![visual_effect_view, setAutoresizingMask: autoresizing];

        // Corner radius
        let _: () = msg_send![visual_effect_view, setWantsLayer: YES];
        let layer: id = msg_send![visual_effect_view, layer];
        if !layer.is_null() {
            let _: () = msg_send![layer, setCornerRadius: 10.0_f64];
            let _: () = msg_send![layer, setMasksToBounds: YES];
        }

        // Insert behind webview (position -1 = below)
        let _: () = msg_send![content_view, addSubview: visual_effect_view positioned: -1_i64 relativeTo: nil];

        // Make webview transparent
        let subviews: id = msg_send![content_view, subviews];
        if !subviews.is_null() {
            let count: usize = msg_send![subviews, count];
            for i in 0..count {
                let subview: id = msg_send![subviews, objectAtIndex: i];
                if subview.is_null() || subview == visual_effect_view {
                    continue;
                }
                let responds: bool = msg_send![subview, respondsToSelector: sel!(setDrawsBackground:)];
                if responds {
                    let _: () = msg_send![subview, setDrawsBackground: NO];
                }
                let responds2: bool = msg_send![subview, respondsToSelector: sel!(setValue:forKey:)];
                if responds2 {
                    let key: id = msg_send![class!(NSString), stringWithUTF8String: b"drawsBackground\0".as_ptr()];
                    let no_value: id = msg_send![class!(NSNumber), numberWithBool: NO];
                    let _: () = msg_send![subview, setValue: no_value forKey: key];
                }
            }
        }

        log::info!("Native macOS vibrancy applied");
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn set_window_blur<R: Runtime>(_window: &WebviewWindow<R>, _enabled: bool) -> Result<(), String> {
    Ok(())
}

// Tauri commands

#[tauri::command]
pub async fn show_window<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(MAIN_WINDOW_LABEL) {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = window.center_at_cursor_monitor();
            }
            panel.show();
            panel.make_key_window();
            return Ok(());
        }
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn hide_window<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(MAIN_WINDOW_LABEL) {
            panel.hide();
            return Ok(());
        }
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.hide().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_window<R: Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(MAIN_WINDOW_LABEL) {
            if panel.is_visible() {
                panel.hide();
            } else {
                if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                    let _ = window.center_at_cursor_monitor();
                }
                panel.show();
                panel.make_key_window();
            }
            return Ok(());
        }
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let is_visible = window.is_visible().map_err(|e| e.to_string())?;
        if is_visible {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn is_window_visible<R: Runtime>(app: tauri::AppHandle<R>) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(MAIN_WINDOW_LABEL) {
            return Ok(panel.is_visible());
        }
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}
