/// Keyboard simulation module for macOS
/// Uses CGEvent to simulate Cmd+V keystroke

#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

/// Virtual key code for 'V' on macOS
#[cfg(target_os = "macos")]
const KEY_V: CGKeyCode = 9;

/// Simulate Cmd+V keystroke using CGEvent API
#[cfg(target_os = "macos")]
pub fn simulate_cmd_v() -> Result<(), String> {
    // Use CombinedSessionState for better compatibility across applications
    // This combines both HID system state and session state
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|_| "Failed to create CGEventSource")?;

    // Create key down event for 'V'
    let key_down = CGEvent::new_keyboard_event(source.clone(), KEY_V, true)
        .map_err(|_| "Failed to create key down event")?;

    // Create key up event for 'V'
    let key_up = CGEvent::new_keyboard_event(source, KEY_V, false)
        .map_err(|_| "Failed to create key up event")?;

    // Set Command modifier flag
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);

    // Post events to Session level (application level, more compatible)
    // Session posts to the current login session which works better across apps
    key_down.post(CGEventTapLocation::Session);
    key_up.post(CGEventTapLocation::Session);

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn simulate_cmd_v() -> Result<(), String> {
    // Not implemented for other platforms
    Err("Keyboard simulation not implemented for this platform".to_string())
}
