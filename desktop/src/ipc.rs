//! Realm IPC for the desktop shell.
//!
//! Thin Tauri commands over `legend_core` mirroring the CLI read semantics:
//! open a realm (parse + recompute + validate), and read a node's Markdown
//! body for the preview pane. Mutation commands (update/wrap/reparent/…)
//! land in a later milestone; the renderer is read-only for now.
//!
//! Payload shapes mirror `.legend/index.json` (camelCase) plus a per-node
//! `body`, so the web/dev renderer can render a fixture snapshot identically.

use legend_core::model::{utc_now_rfc3339, NodeFile, NodeKind, Priority, Status};
use legend_core::ops::{self, ComponentEdits, MutationOutcome, UpdateArgs};
use legend_core::realm::{Realm, RealmError};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::watcher::{self, RealmWatcher};

/// Managed app state: the currently open realm (if any) plus the file
/// watcher armed for it (replaced on every open/reload).
#[derive(Default)]
pub struct RealmState {
    pub realm: Mutex<Option<Realm>>,
    pub watcher: Mutex<Option<RealmWatcher>>,
}

/// Per-node view payload — every component plus runtime-computed state plus
/// the raw Markdown body (mirrors the `.legend/index.json` entry shape).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeView {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub effective_status: String,
    pub blocked: bool,
    pub priority: String,
    pub disciplines: Vec<String>,
    pub epics: Vec<String>,
    pub quest_points: Option<u32>,
    pub landmark: Option<String>,
    pub explicit_landmark: Option<String>,
    pub parent: Option<String>,
    pub blocked_by: Vec<String>,
    pub total_qp: u32,
    pub locked_qp_percent: Option<f64>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
    pub tags: Vec<String>,
    pub path: String,
    pub validation_error: Option<String>,
    pub body: String,
}

/// Top-level `realm_open` payload.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RealmPayload {
    pub root: String,
    pub generated_at: String,
    pub node_count: usize,
    pub warnings: Vec<String>,
    pub nodes: Vec<NodeView>,
}

fn to_node_view(realm: &Realm, id: &str, file: &NodeFile) -> NodeView {
    let n = &file.node;
    let comp = realm.computed.get(id).cloned().unwrap_or_default();
    let name = realm.file_names.get(id).map(String::as_str).unwrap_or(id);
    NodeView {
        id: n.id.clone(),
        kind: n.kind.to_string(),
        title: n.title.clone(),
        status: n.status.to_string(),
        effective_status: comp.effective_status,
        blocked: comp.blocked,
        priority: n.priority.to_string(),
        disciplines: n.disciplines.clone(),
        epics: n.epics.clone(),
        quest_points: n.quest_points,
        landmark: comp.effective_landmark,
        explicit_landmark: n.landmark.clone(),
        parent: n.parent.clone(),
        blocked_by: n.blocked_by.clone(),
        total_qp: comp.total_qp,
        locked_qp_percent: comp.locked_qp_pct,
        completed_at: n.completed_at.clone(),
        created_at: n.created_at.clone(),
        tags: n.tags.clone(),
        path: format!("Nodes/{name}"),
        validation_error: comp.validation_error,
        body: file.body.clone(),
    }
}

fn payload(realm: &Realm, root: String) -> RealmPayload {
    RealmPayload {
        root,
        generated_at: utc_now_rfc3339(),
        node_count: realm.nodes.len(),
        warnings: realm.warnings.clone(),
        nodes: realm
            .sorted_ids()
            .into_iter()
            .filter_map(|id| realm.file(&id).map(|f| to_node_view(realm, &id, f)))
            .collect(),
    }
}

/// Payload for a successful mutation: the refreshed realm snapshot plus a
/// one-line human summary (renderer toast / status bar).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationPayload {
    pub realm: RealmPayload,
    pub message: String,
}

fn mutation_payload(realm: &Realm, root: String, out: MutationOutcome) -> MutationPayload {
    MutationPayload {
        realm: payload(realm, root),
        message: out.message,
    }
}

/// Resolve the user-supplied realm path: absolute stays absolute, relative is
/// resolved against the current working directory (prototype default:
/// `examples/realm-demo` when launched from the repo root).
fn resolve_root(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir().map(|cwd| cwd.join(&p)).unwrap_or(p)
    }
}

/// Open (or re-open, i.e. reload after external edits) a realm by path.
/// Arms (or re-arms) the file watcher on the realm directory so subsequent
/// external edits emit `realm://changed` to the renderer (spec §4.2).
#[tauri::command]
pub fn realm_open(
    app: AppHandle,
    state: State<'_, RealmState>,
    path: String,
) -> Result<RealmPayload, String> {
    let realm = Realm::open(resolve_root(&path)).map_err(|e: RealmError| e.to_string())?;
    let root = realm.root.display().to_string();
    let view = payload(&realm, root.clone());
    println!("[ipc] realm_open `{root}` → {} nodes", view.node_count);

    // Re-arm the watcher for the (possibly new) realm root. A stale watcher
    // from a previous realm is dropped here.
    match watcher::watch_realm(&realm.root, app.clone()) {
        Ok(w) => {
            if let Ok(mut g) = state.watcher.lock() {
                *g = Some(w);
            }
        }
        Err(e) => eprintln!("[ipc] file watcher unavailable: {e}"),
    }

    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    *guard = Some(realm);
    Ok(view)
}

/// Fetch the raw Markdown of one node (frontmatter already shown as chips;
/// the body drives the preview pane).
#[tauri::command]
pub fn node_body(state: State<'_, RealmState>, id: String) -> Result<String, String> {
    let guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_ref().ok_or_else(|| "no realm open".to_string())?;
    realm
        .file(&id)
        .map(|f| f.body.clone())
        .ok_or_else(|| format!("no such node `{id}`"))
}

/* ---- mutation commands (shared legend_core::ops semantics, spec §4/§6.4) -- */

/// Create a node (`legend create`).
#[tauri::command]
pub fn node_create(
    state: State<'_, RealmState>,
    kind: String,
    title: String,
    parent: Option<String>,
    priority: Option<String>,
    quest_points: Option<u32>,
) -> Result<MutationPayload, String> {
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let kind_v: NodeKind = kind.parse().map_err(|_| {
        format!("unknown kind `{kind}` (expected card, genre, action, guard or idea)")
    })?;
    let priority_v = match priority.as_deref() {
        Some(p) => p
            .parse::<Priority>()
            .map_err(|_| format!("unknown priority `{p}`"))?,
        None => Priority::Medium,
    };
    let out = ops::create(
        realm,
        ops::CreateArgs {
            kind: kind_v,
            title,
            parent,
            priority: priority_v,
            quest_points,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(mutation_payload(realm, root, out))
}

/// Update a node's stored lifecycle status (`legend update --status`).
/// Cards with unvanquished leaf tasks require `cascade: true`.
#[tauri::command]
pub fn node_update_status(
    state: State<'_, RealmState>,
    id: String,
    status: String,
    cascade: bool,
) -> Result<MutationPayload, String> {
    if status == "blocked" {
        return Err(
            "`blocked` is a computed runtime state and cannot be set; resolve or vanquish its prerequisites instead"
                .to_string(),
        );
    }
    let status_v: Status = status.parse().map_err(|_| {
        format!("unknown status `{status}` (expected unstarted, active or vanquished)")
    })?;
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let out = ops::update(
        realm,
        UpdateArgs {
            id,
            status: status_v,
            cascade,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(mutation_payload(realm, root, out))
}

/// Rename a node: new title + re-derived file slug (spec §3.3).
#[tauri::command]
pub fn node_rename(
    state: State<'_, RealmState>,
    id: String,
    title: String,
) -> Result<MutationPayload, String> {
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let out = ops::rename(realm, &id, &title).map_err(|e| e.to_string())?;
    Ok(mutation_payload(realm, root, out))
}

/// Wrap a node in a new Card group ("Promote to Card", spec §4.1).
#[tauri::command]
pub fn node_wrap(
    state: State<'_, RealmState>,
    id: String,
    title: Option<String>,
) -> Result<MutationPayload, String> {
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let out = ops::wrap(realm, &id, title).map_err(|e| e.to_string())?;
    Ok(mutation_payload(realm, root, out))
}

/// Re-parent a node (cycle-checked) or promote it to root when `parent` is
/// null (spec §4.5).
#[tauri::command]
pub fn node_reparent(
    state: State<'_, RealmState>,
    id: String,
    parent: Option<String>,
) -> Result<MutationPayload, String> {
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let out = ops::reparent(realm, &id, parent).map_err(|e| e.to_string())?;
    Ok(mutation_payload(realm, root, out))
}

/// Delete a node or its whole subgraph (spec §4.5); dangling `blocked_by`
/// references are stripped from survivors.
#[tauri::command]
pub fn node_delete(
    state: State<'_, RealmState>,
    id: String,
    recursive: bool,
) -> Result<MutationPayload, String> {
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let out = ops::delete(realm, &id, recursive).map_err(|e| e.to_string())?;
    Ok(mutation_payload(realm, root, out))
}

/// Overwrite a node's unmanaged Markdown body (frontmatter preserved).
#[tauri::command]
pub fn node_set_body(
    state: State<'_, RealmState>,
    id: String,
    body: String,
) -> Result<MutationPayload, String> {
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let out = ops::set_body(realm, &id, &body).map_err(|e| e.to_string())?;
    Ok(mutation_payload(realm, root, out))
}

/// Edit node components in one write (doc 05 §5.1 metadata panel): priority,
/// quest points, disciplines, epics, landmark, tags, dependencies and parent.
///
/// JSON cannot carry `Option<Option<T>>`, so the nullable components (quest
/// points, landmark, parent) use an explicit `clear_*` flag: `clear_x = true`
/// clears the component, otherwise a supplied value replaces it and an
/// omitted one leaves it untouched.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn node_update_components(
    state: State<'_, RealmState>,
    id: String,
    priority: Option<String>,
    quest_points: Option<u32>,
    clear_quest_points: bool,
    disciplines: Option<Vec<String>>,
    epics: Option<Vec<String>>,
    landmark: Option<String>,
    clear_landmark: bool,
    tags: Option<Vec<String>>,
    blocked_by: Option<Vec<String>>,
    parent: Option<String>,
    clear_parent: bool,
) -> Result<MutationPayload, String> {
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let priority_v = match priority.as_deref() {
        Some(p) => Some(p.parse::<Priority>().map_err(|_| {
            format!("unknown priority `{p}` (expected critical, high, medium or low)")
        })?),
        None => None,
    };
    let edits = ComponentEdits {
        priority: priority_v,
        quest_points: if clear_quest_points {
            Some(None)
        } else {
            quest_points.map(Some)
        },
        disciplines,
        epics,
        landmark: if clear_landmark {
            Some(None)
        } else {
            landmark.map(Some)
        },
        tags,
        blocked_by,
        parent: if clear_parent {
            Some(None)
        } else {
            parent.map(Some)
        },
    };
    let out = ops::update_components(realm, &id, edits).map_err(|e| e.to_string())?;
    Ok(mutation_payload(realm, root, out))
}

/// Rebuild indexes after external/manual edits (`legend index sync`),
/// returning the same payload shape as `realm_open` plus per-node fix
/// suggestions for any validation issues.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPayload {
    pub realm: RealmPayload,
    pub issues: Vec<SyncIssue>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncIssue {
    pub id: String,
    pub suggestions: Vec<String>,
}

#[tauri::command]
pub fn index_sync(state: State<'_, RealmState>) -> Result<SyncPayload, String> {
    let mut guard = state.realm.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_mut().ok_or_else(|| "no realm open".to_string())?;
    let root = realm.root.display().to_string();
    let issues = ops::index_sync(realm).map_err(|e| e.to_string())?;
    let view = payload(realm, root);
    let issues: Vec<SyncIssue> = issues
        .into_iter()
        .map(|(id, suggestions)| SyncIssue { id, suggestions })
        .collect();
    Ok(SyncPayload {
        realm: view,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_outcomes_shape_messages() {
        // Messages must be one-line renderer/CLI summaries.
        let out = MutationOutcome {
            created: vec!["CARD-0001".into()],
            changed: vec!["ACT-0001".into()],
            message: "wrapped ACT-0001 under new card CARD-0001".into(),
            ..Default::default()
        };
        assert_eq!(out.created.len() + out.changed.len(), 2);
        assert!(out.message.contains("wrapped"));
    }

    #[test]
    fn payload_mirrors_index_entries() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/realm-demo");
        let realm = Realm::open(&root).unwrap();
        let view = payload(&realm, root.display().to_string());
        assert_eq!(view.node_count, 7, "6 tasks + the Player Systems genre");
        assert!(view.warnings.is_empty());
        let genre = view.nodes.iter().find(|n| n.id == "GENRE-3Z8N").unwrap();
        assert_eq!(genre.kind, "genre");
        assert_eq!(
            genre.parent.as_deref(),
            None,
            "the genre is a top-level categorization branch"
        );
        let act = view.nodes.iter().find(|n| n.id == "ACT-3X7P").unwrap();
        assert_eq!(act.effective_status, "blocked");
        assert!(act.blocked);
        assert_eq!(act.landmark.as_deref(), Some("Landmark_01_Demo"));
        assert!(act.body.contains("## Action Goal"));
        let card = view.nodes.iter().find(|n| n.id == "CARD-K9F2").unwrap();
        assert_eq!(card.total_qp, 9);
        let grd = view.nodes.iter().find(|n| n.id == "GRD-7M2Q").unwrap();
        assert_eq!(grd.kind, "guard");
        assert_eq!(grd.parent.as_deref(), Some("CARD-K9F2"));
        assert!(grd.body.contains("## Quality Check Criteria"));
        // every node entry carries a parseable body
        assert!(view.nodes.iter().all(|n| !n.body.is_empty()));
    }
}
