use std::path::PathBuf;

use ai_limits::notifications::{Notification, NotificationDelivery};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

const PREVIOUS_REMAINING_STORE_FILE: &str = "notifications-previous-remaining.json";

/// Where the "100% again" previous-remaining store is persisted. Lives in the
/// Tauri app data directory because the shared core must not depend on Tauri
/// for path resolution; this is the only place that knows the concrete path.
pub fn previous_remaining_store_path(app: &AppHandle) -> tauri::Result<PathBuf> {
    Ok(app
        .path()
        .app_data_dir()?
        .join(PREVIOUS_REMAINING_STORE_FILE))
}

pub struct TauriNotificationDelivery {
    app: AppHandle,
}

impl TauriNotificationDelivery {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl NotificationDelivery for TauriNotificationDelivery {
    fn deliver(&self, notification: &Notification) -> std::io::Result<()> {
        let _ = self
            .app
            .notification()
            .builder()
            .title(&notification.title)
            .body(format!(
                "{}\n{}",
                notification.subtitle, notification.message
            ))
            .show();

        Ok(())
    }
}
