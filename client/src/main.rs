#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use farmlink_client::commands::{self, AppState};
use farmlink_client::config::Config;

fn main() {
    let config = Config::load().unwrap_or_default();
    let app_state = AppState::new(config);

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::scan_mods,
            commands::sync_server,
            commands::launch_game,
            commands::set_mods_path,
            commands::get_companion_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running farmlink tauri application");
}
