mod commands;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(HashSet::<String>::new())))
        .invoke_handler(tauri::generate_handler![
            commands::get_provider_limits,
            commands::get_single_provider_limits,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}
