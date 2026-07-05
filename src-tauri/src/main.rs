mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::get_provider_limits])
        .run(tauri::generate_context!())
        .expect("failed to run Tauri application");
}
