//! Legendary desktop shell (Tauri v2).
//!
//! This shell hosts the Vue 3 renderer (`../frontend`) and exposes
//! `legend_core` operations over Tauri commands: read-only realm preview
//! (`realm_open`, `node_body`) plus the full mutation set (`node_create`,
//! `node_update_status`, `node_rename`, `node_wrap`, `node_reparent`,
//! `node_delete`, `index_sync`) — all delegating to `legend_core::ops` so
//! the CLI and this shell share one implementation. A `notify`-based file
//! watcher (spec §4.2/§7) emits `realm://changed` after external edits;
//! it is armed by `realm_open` once a realm path is known.

mod ipc;
mod watcher;

use ipc::RealmState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RealmState::default())
        .invoke_handler(tauri::generate_handler![
            ipc::realm_open,
            ipc::node_body,
            ipc::node_create,
            ipc::node_update_status,
            ipc::node_rename,
            ipc::node_set_body,
            ipc::node_update_components,
            ipc::node_wrap,
            ipc::node_reparent,
            ipc::node_delete,
            ipc::index_sync,
        ])
        .setup(|_app| {
            println!("Legendary desktop shell initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Legendary desktop shell");
}
