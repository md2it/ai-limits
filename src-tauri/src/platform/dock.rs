//! macOS Dock icon visibility — tied to Main Window visibility per
//! docs/spec/desktop/mac-popover.md#closing.

#[cfg(target_os = "macos")]
pub fn set_visible(app: &tauri::AppHandle, visible: bool) {
    use tauri::ActivationPolicy;

    let policy = if visible {
        ActivationPolicy::Regular
    } else {
        ActivationPolicy::Accessory
    };
    let _ = app.set_activation_policy(policy);
}
