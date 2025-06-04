mod crypto_types;
mod key_management;
mod signing;

use key_management::*;
use signing::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Key Management
            generate_key_pair,
            list_keys,
            get_key_details,
            // Signing & Verification
            sign_document,
            verify_signature,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
