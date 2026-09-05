//! Realm IPC for the desktop shell.
//!
//! Thin Tauri commands over `legend_core` mirroring the CLI read semantics:
//! open a realm (parse + recompute + validate), and read a node's Markdown
//! body for the preview pane. Mutation commands (update/wrap/reparent/…)
//! land in a later milestone; the renderer is read-only for now.
//!
//! Payload shapes mirror `.legend/index.json` (camelCase) plus a per-node
//! `body`, so the web/dev renderer can render a fixture snapshot identically.

use legend_core::model::{utc_now_rfc3339, NodeFile};
use legend_core::realm::{Realm, RealmError};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

/// Managed app state: the currently open realm (if any).
#[derive(Default)]
pub struct RealmState(pub Mutex<Option<Realm>>);

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
#[tauri::command]
pub fn realm_open(state: State<'_, RealmState>, path: String) -> Result<RealmPayload, String> {
    let realm = Realm::open(resolve_root(&path)).map_err(|e: RealmError| e.to_string())?;
    let root = realm.root.display().to_string();
    let mut guard = state.0.lock().map_err(|_| "realm state poisoned")?;
    let view = payload(&realm, root.clone());
    println!("[ipc] realm_open `{root}` → {} nodes", view.node_count);
    *guard = Some(realm);
    Ok(view)
}

/// Fetch the raw Markdown of one node (frontmatter already shown as chips;
/// the body drives the preview pane).
#[tauri::command]
pub fn node_body(state: State<'_, RealmState>, id: String) -> Result<String, String> {
    let guard = state.0.lock().map_err(|_| "realm state poisoned")?;
    let realm = guard.as_ref().ok_or_else(|| "no realm open".to_string())?;
    realm
        .file(&id)
        .map(|f| f.body.clone())
        .ok_or_else(|| format!("no such node `{id}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_mirrors_index_entries() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/realm-demo");
        let realm = Realm::open(&root).unwrap();
        let view = payload(&realm, root.display().to_string());
        assert_eq!(view.node_count, 6);
        assert!(view.warnings.is_empty());
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
