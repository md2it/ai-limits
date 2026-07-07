use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;

use ai_limits::notifications::{Notification, NotificationDelivery};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn start_notification_bridge(app: AppHandle) {
    thread::spawn(move || {
        let Ok(listener) =
            TcpListener::bind(ai_limits::notifications::TAURI_NOTIFICATION_BRIDGE_ADDR)
        else {
            return;
        };

        for stream in listener.incoming().flatten() {
            handle_bridge_request(stream, &app);
        }
    });
}

fn handle_bridge_request(stream: TcpStream, app: &AppHandle) {
    let mut line = String::new();
    let mut reader = BufReader::new(stream);
    if reader.read_line(&mut line).is_err() {
        return;
    }

    let Ok(notification) = serde_json::from_str::<Notification>(&line) else {
        return;
    };

    let _ = TauriNotificationDelivery { app: app.clone() }.deliver(&notification);
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
