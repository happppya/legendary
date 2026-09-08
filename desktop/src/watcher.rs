//! Realm file watcher (spec §4.2 "External Edits", §7 file watching).
//!
//! Watches the open realm's directory recursively with the `notify` crate
//! and debounces bursts of raw events (editors and agents often touch a
//! file several times per save) into one `realm://changed` event per quiet
//! period. The renderer then re-opens the realm over IPC (`realm_open` is
//! reload-by-path), so index state, computed blocked status and the whole
//! UI stay in sync without index-thrashing loops.

use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// How long to wait after the last raw event before treating the burst as
/// one edit (VS Code / agents typically fire 2–6 events per save).
const DEBOUNCE: Duration = Duration::from_millis(350);

/// Which event paths trigger a refresh: any Markdown/YAML/JSON file under
/// the realm (node files, registries, indexes). Everything else is noise.
fn is_relevant(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("md") | Some("yaml") | Some("yml") | Some("json")
    )
}

/// Handle owning the watcher + debounce threads. Dropping it stops watching.
pub struct RealmWatcher {
    _watcher: RecommendedWatcher,
}

/// Spawn the debounce pump: raw pings collapse into one forwarded ping per
/// `DEBOUNCE` quiet window. Returns when the raw side disconnects.
fn spawn_debouncer(rx: Receiver<()>, out: Sender<()>) {
    thread::spawn(move || loop {
        // Wait for the first ping of a burst; channel close ends the loop.
        match rx.recv() {
            Ok(()) => {}
            Err(_) => return,
        }
        loop {
            match rx.recv_timeout(DEBOUNCE) {
                Ok(()) => continue,                      // still inside the burst
                Err(RecvTimeoutError::Timeout) => break, // quiet → fire
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        if out.send(()).is_err() {
            return;
        }
    });
}

/// Start watching `root` and emit `realm://changed` on the app handle after
/// every debounced burst of relevant file events.
pub fn watch_realm(root: &Path, app: AppHandle) -> notify::Result<RealmWatcher> {
    let (raw_tx, raw_rx) = mpsc::channel::<()>();
    let raw_tx_notify = raw_tx.clone();
    let (fire_tx, fire_rx) = mpsc::channel::<()>();

    spawn_debouncer(raw_rx, fire_tx);

    // Fire pump: emit the Tauri event off the notify thread.
    thread::spawn(move || {
        while fire_rx.recv().is_ok() {
            if let Err(e) = app.emit("realm://changed", ()) {
                eprintln!("[watcher] emit failed: {e}");
            }
        }
    });

    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                if event.paths.iter().any(|p| is_relevant(p)) {
                    let _ = raw_tx_notify.send(());
                }
            }
            Err(e) => eprintln!("[watcher] error: {e}"),
        })?;

    watcher.configure(NotifyConfig::default()).ok();
    watcher.watch(root, RecursiveMode::Recursive)?;

    Ok(RealmWatcher { _watcher: watcher })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relevance_filters_non_node_files() {
        assert!(is_relevant(Path::new("Nodes/CARD-K9F2_combat.md")));
        assert!(is_relevant(Path::new(".legend/taxonomy.yaml")));
        assert!(is_relevant(Path::new(".legend/index.json")));
        assert!(is_relevant(Path::new("Landmarks/Landmark_01.md")));
        assert!(!is_relevant(Path::new("Nodes/tmp-swap-file.tmp")));
        assert!(!is_relevant(Path::new("notes.txt")));
    }

    /// The debounce pump collapses a burst of pings into exactly one fire.
    #[test]
    fn debounce_collapses_bursts() {
        let (raw_tx, raw_rx) = mpsc::channel::<()>();
        let (fire_tx, fire_rx) = mpsc::channel::<()>();
        spawn_debouncer(raw_rx, fire_tx);

        for _ in 0..10 {
            raw_tx.send(()).unwrap();
        }
        let fires = fire_rx.recv_timeout(DEBOUNCE + Duration::from_millis(150));
        assert!(fires.is_ok(), "one debounced fire expected");
        // No second fire within another quiet window.
        assert!(fire_rx
            .recv_timeout(DEBOUNCE + Duration::from_millis(150))
            .is_err());
    }
}
