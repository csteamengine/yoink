use tauri::{Manager, Runtime, WebviewWindow};

#[cfg(target_os = "macos")]
use tauri::Emitter;

#[cfg(target_os = "macos")]
use tauri_nspanel::{
    objc_id::ShareId,
    panel_delegate,
    raw_nspanel::RawNSPanel,
    ManagerExt, WebviewWindowExt as NsPanelExt,
};

pub const MAIN_WINDOW_LABEL: &str = "main";

#[cfg(target_os = "macos")]
pub trait WebviewWindowExt {
    fn to_yoink_panel(&self) -> tauri::Result<ShareId<RawNSPanel>>;
    fn center_at_cursor_monitor(&self) -> Result<(), String>;
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)] // For panel_delegate! macro from tauri-nspanel which uses old objc
impl<R: Runtime> WebviewWindowExt for WebviewWindow<R> {
    fn to_yoink_panel(&self) -> tauri::Result<ShareId<RawNSPanel>> {
        let panel = self.to_panel()?;

        // Set panel level to floating (NSFloatingWindowLevel = 5)
        panel.set_level(5);

        // Set collection behavior for proper space handling
        // NSWindowCollectionBehaviorCanJoinAllSpaces (1) | NSWindowCollectionBehaviorFullScreenAuxiliary (256) | NSWindowCollectionBehaviorTransient (8)
        use objc2_app_kit::NSWindowCollectionBehavior;
        let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Transient;
        panel.set_collection_behaviour(behavior);

        // Set as floating panel
        panel.set_floating_panel(true);

        // Setup delegate for event handling
        let app_handle = self.app_handle().clone();
        let delegate = panel_delegate!(YoinkPanelDelegate {
            window_did_resign_key
        });

        delegate.set_listener(Box::new(move |delegate_name: String| {
            if delegate_name == "window_did_resign_key" {
                log::info!("panel resigned key window");
                // Hide panel when it loses focus
                if let Ok(panel) = app_handle.get_webview_panel(MAIN_WINDOW_LABEL) {
                    if panel.is_visible() {
                        panel.order_out(None);
                        let _ = app_handle.emit("panel-hidden", ());
                    }
                }
            }
        }));

        panel.set_delegate(delegate);

        Ok(panel)
    }

    fn center_at_cursor_monitor(&self) -> Result<(), String> {
        // Get monitor with cursor
        let monitor = monitor::get_monitor_with_cursor()
            .ok_or_else(|| "Monitor with cursor not found".to_string())?;

        let scale = monitor.scale_factor();
        let monitor_size = monitor.size().to_logical::<f64>(scale);
        let monitor_pos = monitor.position().to_logical::<f64>(scale);

        // Get window size
        let window_size = self.outer_size()
            .map_err(|e| e.to_string())?
            .to_logical::<f64>(scale);

        // Calculate centered position (slightly above center)
        let x = monitor_pos.x + (monitor_size.width - window_size.width) / 2.0;
        let y = monitor_pos.y + (monitor_size.height - window_size.height) / 2.0 - 50.0;

        self.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)))
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

/// Apply native macOS vibrancy effect using modern objc2 APIs
#[cfg(target_os = "macos")]
pub fn set_window_blur<R: Runtime>(window: &WebviewWindow<R>, _enabled: bool) -> Result<(), String> {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{msg_send, sel, ClassType};
    use objc2_app_kit::{
        NSColor, NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
        NSVisualEffectState, NSVisualEffectView, NSWindow,
    };
    use objc2_foundation::{CGRect, MainThreadMarker, NSAutoresizingMaskOptions};

    let ns_window_ptr = match window.ns_window() {
        Ok(w) => w,
        Err(e) => return Err(e.to_string()),
    };

    if ns_window_ptr.is_null() {
        return Err("ns_window is null".to_string());
    }

    // SAFETY: We got this pointer from Tauri which guarantees it's a valid NSWindow
    let ns_window: &NSWindow = unsafe { &*(ns_window_ptr as *const NSWindow) };

    // Get main thread marker - required for AppKit operations
    let mtm = match MainThreadMarker::new() {
        Some(m) => m,
        None => return Err("Not on main thread".to_string()),
    };

    unsafe {
        // Make window transparent
        ns_window.setOpaque(false);
        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
        ns_window.setTitlebarAppearsTransparent(true);

        let content_view = match ns_window.contentView() {
            Some(v) => v,
            None => return Err("content_view is null".to_string()),
        };

        // Enable layer backing
        content_view.setWantsLayer(true);
        if let Some(layer) = content_view.layer() {
            let _: () = msg_send![&layer, setCornerRadius: 10.0_f64];
            let _: () = msg_send![&layer, setMasksToBounds: true];
        }

        let bounds = content_view.bounds();

        // Create NSVisualEffectView
        let visual_effect_view = NSVisualEffectView::initWithFrame(
            NSVisualEffectView::alloc(),
            bounds,
        );

        // Dark vibrancy material
        visual_effect_view.setMaterial(NSVisualEffectMaterial::UltraDark);
        visual_effect_view.setState(NSVisualEffectState::Active);
        visual_effect_view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);

        // Auto-resize (width | height sizable)
        visual_effect_view.setAutoresizingMask(
            NSAutoresizingMaskOptions::NSViewWidthSizable
                | NSAutoresizingMaskOptions::NSViewHeightSizable,
        );

        // Corner radius on visual effect view
        visual_effect_view.setWantsLayer(true);
        if let Some(layer) = visual_effect_view.layer() {
            let _: () = msg_send![&layer, setCornerRadius: 10.0_f64];
            let _: () = msg_send![&layer, setMasksToBounds: true];
        }

        // Insert behind other subviews
        // NSWindowBelow = -1
        let retained_view: Retained<NSView> = Retained::cast(visual_effect_view);
        let _: () = msg_send![&content_view, addSubview: &*retained_view, positioned: -1_isize, relativeTo: std::ptr::null::<AnyObject>()];

        // Make webview transparent
        let subviews = content_view.subviews();
        for subview in subviews.iter() {
            // Skip the visual effect view we just added
            if Retained::as_ptr(&subview) == Retained::as_ptr(&retained_view) {
                continue;
            }
            // Try to make the webview transparent
            let responds: bool = msg_send![&subview, respondsToSelector: sel!(setDrawsBackground:)];
            if responds {
                let _: () = msg_send![&subview, setDrawsBackground: false];
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
        use crate::window::WebviewWindowExt;
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
            panel.order_out(None);
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
        use crate::window::WebviewWindowExt;
        if let Ok(panel) = app.get_webview_panel(MAIN_WINDOW_LABEL) {
            if panel.is_visible() {
                panel.order_out(None);
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
