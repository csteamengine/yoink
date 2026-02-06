use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Runtime};

#[cfg(target_os = "macos")]
use tauri::Emitter;

/// Monitors keyboard events for quick-switch mode
pub struct InputMonitor {
    is_active: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
}

impl InputMonitor {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if quick-switch mode is currently active
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    /// Start monitoring for quick-switch mode
    /// This should be called when the hotkey is triggered
    #[cfg(target_os = "macos")]
    pub fn start_quick_switch<R: Runtime>(&self, app: AppHandle<R>) {
        // Don't start if already active
        if self.is_active.load(Ordering::SeqCst) {
            log::info!("Quick-switch already active, skipping");
            return;
        }

        self.is_active.store(true, Ordering::SeqCst);
        self.stop_flag.store(false, Ordering::SeqCst);

        let is_active = self.is_active.clone();
        let stop_flag = self.stop_flag.clone();

        // Track modifier state
        let cmd_held = Arc::new(AtomicBool::new(true)); // Assume held since hotkey was just pressed
        let shift_held = Arc::new(AtomicBool::new(true));

        let cmd_held_clone = cmd_held.clone();
        let shift_held_clone = shift_held.clone();

        thread::spawn(move || {
            use rdev::{listen, Event, EventType, Key};

            log::info!("Starting quick-switch keyboard monitor");

            let callback = move |event: Event| {
                // Check stop flag
                if stop_flag.load(Ordering::SeqCst) {
                    return;
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        match key {
                            Key::MetaLeft | Key::MetaRight => {
                                cmd_held_clone.store(true, Ordering::SeqCst);
                            }
                            Key::ShiftLeft | Key::ShiftRight => {
                                shift_held_clone.store(true, Ordering::SeqCst);
                            }
                            Key::KeyV => {
                                // V pressed while modifiers held -> cycle
                                if cmd_held_clone.load(Ordering::SeqCst)
                                    && shift_held_clone.load(Ordering::SeqCst)
                                {
                                    log::info!("Quick-switch: V pressed, emitting cycle event");
                                    let _ = app.emit("quick-switch-cycle", ());
                                }
                            }
                            _ => {}
                        }
                    }
                    EventType::KeyRelease(key) => {
                        match key {
                            Key::MetaLeft | Key::MetaRight => {
                                cmd_held_clone.store(false, Ordering::SeqCst);
                                // Check if both modifiers released
                                if !shift_held_clone.load(Ordering::SeqCst) {
                                    log::info!("Quick-switch: modifiers released, emitting confirm");
                                    let _ = app.emit("quick-switch-confirm", ());
                                    is_active.store(false, Ordering::SeqCst);
                                } else {
                                    // Just Cmd released, still confirm
                                    log::info!("Quick-switch: Cmd released, emitting confirm");
                                    let _ = app.emit("quick-switch-confirm", ());
                                    is_active.store(false, Ordering::SeqCst);
                                }
                            }
                            Key::ShiftLeft | Key::ShiftRight => {
                                shift_held_clone.store(false, Ordering::SeqCst);
                                // Check if Cmd also released
                                if !cmd_held_clone.load(Ordering::SeqCst) {
                                    log::info!("Quick-switch: modifiers released, emitting confirm");
                                    let _ = app.emit("quick-switch-confirm", ());
                                    is_active.store(false, Ordering::SeqCst);
                                } else {
                                    // Just Shift released, still confirm
                                    log::info!("Quick-switch: Shift released, emitting confirm");
                                    let _ = app.emit("quick-switch-confirm", ());
                                    is_active.store(false, Ordering::SeqCst);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            };

            // Run the listener - this blocks
            if let Err(error) = listen(callback) {
                log::error!("Error in keyboard listener: {:?}", error);
                is_active.store(false, Ordering::SeqCst);
            }
        });
    }

    #[cfg(not(target_os = "macos"))]
    pub fn start_quick_switch<R: Runtime>(&self, _app: AppHandle<R>) {
        // Not implemented for other platforms yet
        log::info!("Quick-switch not implemented for this platform");
    }

    /// Stop the quick-switch monitor
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        self.is_active.store(false, Ordering::SeqCst);
    }
}

// Tauri commands
#[tauri::command]
pub fn is_quick_switch_active(monitor: tauri::State<'_, InputMonitor>) -> bool {
    monitor.is_active()
}

#[tauri::command]
pub fn stop_quick_switch(monitor: tauri::State<'_, InputMonitor>) {
    monitor.stop();
}
