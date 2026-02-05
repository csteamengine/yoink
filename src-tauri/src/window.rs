use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

#[cfg(target_os = "macos")]
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask,
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

pub struct WindowManager;

impl WindowManager {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "macos")]
    pub fn setup_panel<R: Runtime>(&self, window: &WebviewWindow<R>) -> Result<(), String> {
        let app_handle = window.app_handle().clone();

        // Convert window to panel
        let panel = window
            .to_panel::<YoinkPanel<R>>()
            .map_err(|e| e.to_string())?;

        // Set panel level to floating
        panel.set_level(PanelLevel::Floating.value());

        // Set collection behavior - move to active space, transient
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

        let app_for_resign = app_handle.clone();
        handler.window_did_resign_key(move |_| {
            log::info!("panel resigned key window");

            // Hide panel when it loses focus
            if let Ok(panel) = app_for_resign.get_webview_panel(MAIN_WINDOW_LABEL) {
                if panel.is_visible() {
                    panel.hide();
                    let _ = app_for_resign.emit("panel-hidden", ());
                }
            }
        });

        panel.set_event_handler(Some(handler.as_ref()));

        // Apply vibrancy effect
        if let Err(e) = set_window_blur(window) {
            log::warn!("Failed to set window blur: {}", e);
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn setup_panel<R: Runtime>(&self, _window: &WebviewWindow<R>) -> Result<(), String> {
        Ok(())
    }
}

/// Apply native macOS vibrancy effect
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn set_window_blur<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
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

        // Dark vibrancy material
        let _: () = msg_send![visual_effect_view, setMaterial: 9_i64];
        let _: () = msg_send![visual_effect_view, setState: 1_i64];
        let _: () = msg_send![visual_effect_view, setBlendingMode: 0_i64];

        // Auto-resize
        let autoresizing: u64 = 2 | 16;
        let _: () = msg_send![visual_effect_view, setAutoresizingMask: autoresizing];

        // Corner radius
        let _: () = msg_send![visual_effect_view, setWantsLayer: YES];
        let layer: id = msg_send![visual_effect_view, layer];
        if !layer.is_null() {
            let _: () = msg_send![layer, setCornerRadius: 10.0_f64];
            let _: () = msg_send![layer, setMasksToBounds: YES];
        }

        // Insert behind webview
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
pub fn set_window_blur<R: Runtime>(_window: &WebviewWindow<R>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn show_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(MAIN_WINDOW_LABEL) {
            // Move to cursor's monitor and show
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                center_on_cursor_monitor(&window)?;
            }
            panel.show();
            panel.make_key_window();
            return Ok(());
        }
    }

    // Fallback for non-macOS or if panel not available
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn hide_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
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
pub async fn toggle_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = app.get_webview_panel(MAIN_WINDOW_LABEL) {
            if panel.is_visible() {
                panel.hide();
            } else {
                if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                    center_on_cursor_monitor(&window)?;
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
pub async fn is_window_visible<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
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

/// Center window on the monitor where the cursor is located
#[cfg(target_os = "macos")]
fn center_on_cursor_monitor<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    use cocoa::foundation::{NSPoint, NSRect};

    let panel = window
        .get_webview_panel(window.label())
        .map_err(|e| e.to_string())?;

    if let Some(monitor) = get_monitor_with_cursor() {
        let scale = monitor.scale_factor();
        let size = monitor.size().to_logical::<f64>(scale);
        let pos = monitor.position().to_logical::<f64>(scale);

        let panel_ref = panel.as_panel();
        let frame = panel_ref.frame();

        let rect = NSRect {
            origin: NSPoint {
                x: pos.x + (size.width / 2.0) - (frame.size.width / 2.0),
                y: pos.y + (size.height / 2.0) - (frame.size.height / 2.0) + 100.0,
            },
            size: frame.size,
        };

        panel_ref.setFrame_display(rect, true);
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn center_on_cursor_monitor<R: Runtime>(_window: &WebviewWindow<R>) -> Result<(), String> {
    Ok(())
}

/// Get the monitor where the cursor is currently located
#[cfg(target_os = "macos")]
fn get_monitor_with_cursor() -> Option<tauri::monitor::Monitor> {
    use cocoa::base::id;
    use cocoa::foundation::NSPoint;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let mouse_location: NSPoint = msg_send![class!(NSEvent), mouseLocation];

        // Get all screens
        let screens: id = msg_send![class!(NSScreen), screens];
        if screens.is_null() {
            return None;
        }

        let count: usize = msg_send![screens, count];
        for i in 0..count {
            let screen: id = msg_send![screens, objectAtIndex: i];
            if screen.is_null() {
                continue;
            }

            let frame: cocoa::foundation::NSRect = msg_send![screen, frame];

            if mouse_location.x >= frame.origin.x
                && mouse_location.x < frame.origin.x + frame.size.width
                && mouse_location.y >= frame.origin.y
                && mouse_location.y < frame.origin.y + frame.size.height
            {
                // Found the screen, now create a Monitor
                // We'll use the frame info directly
                return Some(create_monitor_from_screen(screen, frame));
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn create_monitor_from_screen(
    _screen: cocoa::base::id,
    frame: cocoa::foundation::NSRect,
) -> tauri::monitor::Monitor {
    use tauri::dpi::{PhysicalPosition, PhysicalSize};

    // Create a simple monitor struct
    // Note: This is a workaround since we can't directly construct tauri::monitor::Monitor
    // For now, we'll just use the position from the frame

    // This is a placeholder - in practice, we should use tauri's monitor API
    // but for centering purposes, we have the frame info
    unsafe {
        std::mem::transmute(MonitorHandle {
            name: None,
            size: PhysicalSize::new(frame.size.width as u32, frame.size.height as u32),
            position: PhysicalPosition::new(frame.origin.x as i32, frame.origin.y as i32),
            scale_factor: 1.0,
        })
    }
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct MonitorHandle {
    name: Option<String>,
    size: tauri::dpi::PhysicalSize<u32>,
    position: tauri::dpi::PhysicalPosition<i32>,
    scale_factor: f64,
}

#[cfg(not(target_os = "macos"))]
fn get_monitor_with_cursor() -> Option<tauri::monitor::Monitor> {
    None
}
