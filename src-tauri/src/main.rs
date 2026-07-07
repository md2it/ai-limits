mod commands;
mod notifications;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(Arc::new(Mutex::new(HashSet::<String>::new())))
        .setup(|app| {
            notifications::start_notification_bridge(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_provider_limits,
            commands::get_single_provider_limits,
            commands::open_external_url,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}
