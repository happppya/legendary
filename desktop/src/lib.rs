//! Legendary desktop shell (Tauri v2).
//!
//! This shell hosts the Vue 3 renderer (`../frontend`) and exposes
//! `legend_core` operations over Tauri commands. Read-only realm preview is
//! live (`realm_open`, `node_body` in [`ipc`]); file watching and mutation
//! commands are tracked in `notes/design-documents/05-…` and `07-…`.

mod ipc;

use ipc::RealmState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RealmState::default())
        .invoke_handler(tauri::generate_handler![ipc::realm_open, ipc::node_body,])
        .setup(|_app| {
            println!("Legendary desktop shell initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Legendary desktop shell");
}
