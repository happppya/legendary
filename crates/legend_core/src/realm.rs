//! A *realm* is the flat game-project layout Legendary manages (spec §3.4):
//!
//! ```text
//! realm/
//! ├── AGENTS.md
//! ├── .legend/      # taxonomy.yaml, landmarks.yaml + generated indexes
//! ├── Landmarks/    # *.md milestone declarations
//! └── Nodes/        # flat entity database: [ID]_[slug].md
//! ```
//!
//! Loading a realm parses every node file, validates components against the
//! registries, computes runtime DAG state, and can regenerate
//! `.legend/index.json` + `.legend/INDEX.md` synchronously.

use crate::dag;
use crate::frontmatter::{self, FrontmatterError};
use crate::id::slugify_title;
use crate::model::{utc_now_rfc3339, Node, NodeFile};
use crate::registry::{LandmarkRegistry, Taxonomy};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RealmError {
    #[error("not a legendary realm (missing `.legend/`): {0}")]
    NotARealm(String),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
    #[error("parse error in `{file}`: {source}")]
    Parse {
        file: String,
        #[source]
        source: FrontmatterError,
    },
}

/// Runtime-computed per-node state (never persisted to frontmatter).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeComputed {
    pub effective_status: String,
    pub blocked: bool,
    pub effective_landmark: Option<String>,
    pub total_qp: u32,
    pub locked_qp_pct: Option<f64>,
    pub validation_error: Option<String>,
}

/// Outcome of an explicit rename (`legend rename`, spec §3.3): slug
/// stability is *not* automatic, so this is the only operation that renames
/// a node file on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameOutcome {
    pub title_changed: bool,
    pub file_renamed: bool,
    pub old_name: Option<String>,
    pub new_name: String,
}

/// A loaded realm: registries + parsed node files + computed state.
#[derive(Debug, Default)]
pub struct Realm {
    pub root: PathBuf,
    pub nodes_dir: PathBuf,
    pub legend_dir: PathBuf,
    pub landmarks_dir: PathBuf,
    pub taxonomy: Taxonomy,
    pub landmarks: LandmarkRegistry,
    pub nodes: HashMap<String, NodeFile>,
    pub file_names: HashMap<String, String>,
    pub computed: HashMap<String, NodeComputed>,
    pub warnings: Vec<String>,
}

impl Realm {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, RealmError> {
        let root = root.as_ref().to_path_buf();
        if !root.is_dir() {
            return Err(RealmError::NotARealm(root.display().to_string()));
        }
        let legend_dir = root.join(".legend");
        if !legend_dir.is_dir() {
            return Err(RealmError::NotARealm(root.display().to_string()));
        }
        let nodes_dir = root.join("Nodes");
        let landmarks_dir = root.join("Landmarks");

        let taxonomy = Taxonomy::load(&legend_dir)?;
        let landmarks = LandmarkRegistry::load(&legend_dir, &landmarks_dir)?;

        let mut realm = Realm {
            root,
            nodes_dir,
            legend_dir,
            landmarks_dir,
            taxonomy,
            landmarks,
            ..Realm::default()
        };

        if realm.nodes_dir.is_dir() {
            let mut entries: Vec<PathBuf> = fs::read_dir(&realm.nodes_dir)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "md"))
                .collect();
            entries.sort();
            for path in entries {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                let text = match fs::read_to_string(&path) {
                    Ok(t) => t,
                    Err(e) => {
                        realm.warnings.push(format!("skipping `{name}`: {e}"));
                        continue;
                    }
                };
                match frontmatter::parse_node_file(&text) {
                    Ok(file) => {
                        let id = file.node.id.clone();
                        if realm.nodes.contains_key(&id) {
                            realm
                                .warnings
                                .push(format!("duplicate node id `{id}` in `{name}`; ignored"));
                            continue;
                        }
                        realm.nodes.insert(id.clone(), file);
                        realm.file_names.insert(id, name);
                    }
                    Err(e) => {
                        realm.warnings.push(format!("skipping `{name}`: {e}"));
                    }
                }
            }
        } else {
            realm.warnings.push(format!(
                "no `Nodes/` directory at {}",
                realm.nodes_dir.display()
            ));
        }

        realm.recompute();
        Ok(realm)
    }

    /// Ids sorted for stable output: Cards first, then Actions, Guards, Ideas;
    /// alphabetical within a kind.
    pub fn sorted_ids(&self) -> Vec<String> {
        let mut ids: Vec<&String> = self.nodes.keys().collect();
        let rank = |id: &&String| match self.nodes.get(*id).map(|f| f.node.kind) {
            Some(k) => match k {
                crate::model::NodeKind::Card => 0,
                crate::model::NodeKind::Action => 1,
                crate::model::NodeKind::Guard => 2,
                crate::model::NodeKind::Idea => 3,
            },
            None => 9,
        };
        ids.sort_by(|a, b| rank(a).cmp(&rank(b)).then_with(|| a.cmp(b)));
        ids.into_iter().cloned().collect()
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id).map(|f| &f.node)
    }

    pub fn file(&self, id: &str) -> Option<&NodeFile> {
        self.nodes.get(id)
    }

    pub fn file_path(&self, id: &str) -> Option<PathBuf> {
        self.file_names
            .get(id)
            .map(|name| self.nodes_dir.join(name))
    }

    /// Deterministic child ids of a parent.
    pub fn children(&self, parent: &str) -> Vec<String> {
        dag::children_sorted(&self.node_map(), parent)
    }

    pub fn node_map(&self) -> HashMap<String, Node> {
        self.nodes
            .iter()
            .map(|(id, f)| (id.clone(), f.node.clone()))
            .collect()
    }

    /// Recompute runtime state and validation for every node after any
    /// structural mutation or manual external edit.
    pub fn recompute(&mut self) {
        let nodes = self.node_map();
        let cycles: std::collections::HashSet<String> =
            dag::cyclic_ids(&nodes).into_iter().collect();
        let mut next: HashMap<String, NodeComputed> = HashMap::new();
        for id in self.sorted_ids() {
            let file = &self.nodes[&id];
            let node = &file.node;
            let mut errors: Vec<String> = Vec::new();

            if cycles.contains(&id) {
                errors.push("Cycle detected".to_string());
            }
            if let Some(p) = node.parent.as_deref() {
                if !self.nodes.contains_key(p) {
                    errors.push(format!("unknown parent `{p}`"));
                }
            }
            for dep in &node.blocked_by {
                if !self.nodes.contains_key(dep) {
                    errors.push(format!("unknown dependency `{dep}`"));
                }
            }
            for d in &node.disciplines {
                if !self.taxonomy.is_valid_discipline(d) {
                    errors.push(format!("unknown discipline `{d}`"));
                }
            }
            for e in &node.epics {
                if !self.taxonomy.is_valid_epic(e) {
                    errors.push(format!("unknown epic `{e}`"));
                }
            }
            if let Some(lm) = &node.landmark {
                if !self.landmarks.is_valid(lm) {
                    errors.push(format!("unknown landmark `{lm}`"));
                }
            }
            if let Some(qp) = node.quest_points {
                if !crate::model::is_valid_quest_points(qp) {
                    errors.push(format!(
                        "quest_points {qp} is not Fibonacci (1,2,3,5,8,13,21)"
                    ));
                }
            }

            let effective_status = dag::effective_status(&nodes, &id);
            next.insert(
                id.clone(),
                NodeComputed {
                    blocked: effective_status == "blocked",
                    effective_status,
                    effective_landmark: dag::effective_landmark(&nodes, &id),
                    total_qp: if node.kind == crate::model::NodeKind::Card {
                        dag::aggregate_qp(&nodes, &id)
                    } else {
                        node.quest_points()
                    },
                    locked_qp_pct: if node.kind == crate::model::NodeKind::Card {
                        dag::locked_qp_percent(&nodes, &id)
                    } else {
                        None
                    },
                    validation_error: if errors.is_empty() {
                        None
                    } else {
                        Some(errors.join(", "))
                    },
                },
            );
        }
        self.computed = next;
    }

    /// Synchronously regenerate `.legend/index.json` and `.legend/INDEX.md`
    /// (spec §6.1, §6.4). Must run after every UI/CLI write mutation.
    pub fn sync_index(&self) -> Result<(), RealmError> {
        let mut nodes_map = Map::new();
        for id in self.sorted_ids() {
            let file = &self.nodes[&id];
            let n = &file.node;
            let comp = self.computed.get(&id).cloned().unwrap_or_default();
            let entry = json!({
                "id": n.id,
                "kind": n.kind.to_string(),
                "title": n.title,
                "status": n.status.to_string(),
                "effective_status": comp.effective_status,
                "blocked": comp.blocked,
                "priority": n.priority.to_string(),
                "disciplines": n.disciplines,
                "epics": n.epics,
                "quest_points": n.quest_points,
                "landmark": comp.effective_landmark,
                "explicit_landmark": n.landmark,
                "parent": n.parent,
                "blocked_by": n.blocked_by,
                "total_qp": comp.total_qp,
                "locked_qp_percent": comp.locked_qp_pct,
                "completed_at": n.completed_at,
                "created_at": n.created_at,
                "tags": n.tags,
                "path": format!("Nodes/{}", self.file_names.get(&id).unwrap_or(&n.id)),
                "validation_error": comp.validation_error,
            });
            nodes_map.insert(id.clone(), entry);
        }
        let root = json!({
            "schema_version": 1,
            "generated_at": utc_now_rfc3339(),
            "node_count": self.nodes.len(),
            "nodes": Value::Object(nodes_map),
        });
        fs::write(
            self.legend_dir.join("index.json"),
            serde_json::to_string_pretty(&root)? + "\n",
        )?;

        let mut md = String::from(
            "# Realm Index\n\n> Auto-generated by `legend index sync`. Do not edit by hand.\n\n",
        );
        md.push_str("|ID|Kind|Title|Status|Landmark|QP|\n|---|---|---|---|---|---|\n");
        for id in self.sorted_ids() {
            let file = &self.nodes[&id];
            let n = &file.node;
            let comp = self.computed.get(&id).cloned().unwrap_or_default();
            let esc = |s: &str| s.replace('|', "\\|");
            md.push_str(&format!(
                "|{}|{}|{}|{}|{}|{}|\n",
                n.id,
                n.kind.label(),
                esc(&n.title),
                comp.effective_status,
                comp.effective_landmark.as_deref().unwrap_or(""),
                comp.total_qp,
            ));
        }
        fs::write(self.legend_dir.join("INDEX.md"), md)?;
        Ok(())
    }

    /// Resolve legacy registry aliases on a node in memory (spec §3.2):
    /// discipline/epic paths and landmark ids are mapped to their canonical
    /// values so a rewrite persists canonical frontmatter instead of stale
    /// alias strings. Returns whether anything changed.
    pub fn canonicalize_node(&mut self, id: &str) -> bool {
        let Some(file) = self.nodes.get_mut(id) else {
            return false;
        };
        let n = &mut file.node;
        let mut changed = false;
        for d in &mut n.disciplines {
            let canonical = self.taxonomy.canonical_path(d);
            if canonical != *d {
                *d = canonical;
                changed = true;
            }
        }
        for e in &mut n.epics {
            let canonical = self.taxonomy.canonical_path(e);
            if canonical != *e {
                *e = canonical;
                changed = true;
            }
        }
        if let Some(lm) = &mut n.landmark {
            let canonical = self.landmarks.canonical(lm);
            if canonical != *lm {
                *lm = canonical;
                changed = true;
            }
        }
        changed
    }

    /// Persist an existing node file back to disk (canonical frontmatter,
    /// unmanaged body preserved). Legacy discipline/epic/landmark aliases are
    /// canonicalized first, so any write leaves canonical values on disk.
    pub fn write_node(&mut self, id: &str) -> Result<(), RealmError> {
        self.canonicalize_node(id);
        let file = self
            .nodes
            .get(id)
            .ok_or_else(|| RealmError::Other(format!("no such node `{id}`")))?
            .clone();
        let path = self
            .file_path(id)
            .ok_or_else(|| RealmError::Other(format!("no file registered for `{id}`")))?;
        let text = frontmatter::render_node_file(&file).map_err(|e| RealmError::Parse {
            file: path.display().to_string(),
            source: e,
        })?;
        fs::write(&path, text)?;
        Ok(())
    }

    /// Persist a brand-new node (generates the `[ID]_[slug].md` filename).
    pub fn create_node_file(&mut self, file: NodeFile, slug: &str) -> Result<PathBuf, RealmError> {
        let id = file.node.id.clone();
        if self.nodes.contains_key(&id) {
            return Err(RealmError::Other(format!("node `{id}` already exists")));
        }
        let name = format!("{id}_{slug}.md");
        let path = self.nodes_dir.join(&name);
        let text = frontmatter::render_node_file(&file).map_err(|e| RealmError::Parse {
            file: name.clone(),
            source: e,
        })?;
        fs::write(&path, text)?;
        self.nodes.insert(id.clone(), file);
        self.file_names.insert(id, name);
        Ok(path)
    }

    /// Explicit rename: update the `title` component and, when the slug
    /// changes, rename the `[ID]_[slug].md` file on disk (spec §3.3). The id
    /// prefix is immutable. The rewritten file carries canonical frontmatter
    /// and the unmanaged body is preserved.
    pub fn rename_node(&mut self, id: &str, new_title: &str) -> Result<RenameOutcome, RealmError> {
        let old_name = self.file_names.get(id).cloned();
        let Some(file) = self.nodes.get_mut(id) else {
            return Err(RealmError::Other(format!("no such node `{id}`")));
        };
        let old_title = file.node.title.clone();
        file.node.title = new_title.to_string();
        let title_changed = old_title != new_title;

        let new_name = format!("{id}_{}.md", slugify_title(new_title));
        let file_renamed = old_name.as_deref() != Some(new_name.as_str());
        if file_renamed {
            if let Some(old) = &old_name {
                let target = self.nodes_dir.join(&new_name);
                if target.exists() {
                    return Err(RealmError::Other(format!(
                        "cannot rename: `{}` already exists",
                        target.display()
                    )));
                }
                fs::rename(self.nodes_dir.join(old), &target).map_err(|e| {
                    RealmError::Other(format!("could not rename `{old}` to `{new_name}`: {e}"))
                })?;
            }
            self.file_names.insert(id.to_string(), new_name.clone());
        }
        // Persist the new title (and canonicalize aliases on the way out).
        self.write_node(id)?;
        Ok(RenameOutcome {
            title_changed,
            file_renamed,
            old_name,
            new_name,
        })
    }

    /// Human-readable fix suggestions for a node's current validation
    /// problems (surfaced by `legend index sync`). One suggestion per issue;
    /// empty when the node is clean.
    pub fn suggestions_for(&self, id: &str) -> Vec<String> {
        let Some(file) = self.nodes.get(id) else {
            return Vec::new();
        };
        let n = &file.node;
        let mut out: Vec<String> = Vec::new();

        for d in &n.disciplines {
            if !self.taxonomy.is_valid_discipline(d) {
                out.push(format!(
                    "unknown discipline `{d}`: declare it under `disciplines:` in `.legend/taxonomy.yaml`, or use one of: {}",
                    self.taxonomy.discipline_paths().join(", ")
                ));
            }
        }
        for e in &n.epics {
            if !self.taxonomy.is_valid_epic(e) {
                out.push(format!(
                    "unknown epic `{e}`: declare it under `epics:` in `.legend/taxonomy.yaml`, or use one of: {}",
                    self.taxonomy.epic_paths().join(", ")
                ));
            }
        }
        if let Some(lm) = &n.landmark {
            if !self.landmarks.is_valid(lm) {
                out.push(format!(
                    "unknown landmark `{lm}`: declare it in `.legend/landmarks.yaml` or drop a `{lm}.md` file into `Landmarks/`; known: {}",
                    if self.landmarks.landmark_names().is_empty() {
                        "none yet".to_string()
                    } else {
                        self.landmarks.landmark_names().join(", ")
                    }
                ));
            }
        }
        if let Some(p) = n.parent.as_deref() {
            if !self.nodes.contains_key(p) {
                out.push(format!(
                    "parent `{p}` does not exist: create that node first, or point `parent` at an existing node id"
                ));
            }
        }
        for dep in &n.blocked_by {
            if !self.nodes.contains_key(dep) {
                out.push(format!(
                    "blocked_by references `{dep}` which does not exist: create it, or remove the stale prerequisite"
                ));
            }
        }
        if let Some(qp) = n.quest_points {
            if !crate::model::is_valid_quest_points(qp) {
                out.push(format!(
                    "quest_points {qp} is outside the Fibonacci set: use one of 1, 2, 3, 5, 8, 13, 21"
                ));
            }
        }
        let cyclic = dag::cyclic_ids(&self.node_map());
        if cyclic.contains(&id.to_string()) {
            out.push(
                "node participates in a cycle: remove the circular parent or blocked_by link that closes the loop"
                    .to_string(),
            );
        }
        out
    }

    /// Remove node ids from the realm; strips removed ids from every
    /// remaining node's `blocked_by` (spec §4.5). Returns ids whose files
    /// need rewriting because their prerequisites were stripped.
    pub fn delete_ids(&mut self, ids: &[String]) -> Vec<String> {
        let removed: std::collections::HashSet<&String> = ids.iter().collect();
        let mut affected: Vec<String> = Vec::new();
        for id in self.sorted_ids() {
            if removed.contains(&id) {
                continue;
            }
            let before = self.nodes[&id].node.blocked_by.len();
            self.nodes
                .get_mut(&id)
                .unwrap()
                .node
                .blocked_by
                .retain(|d| !removed.contains(d));
            if self.nodes[&id].node.blocked_by.len() != before {
                affected.push(id.clone());
            }
        }
        for id in ids {
            if let Some(name) = self.file_names.remove(id) {
                let path = self.nodes_dir.join(&name);
                if let Err(e) = fs::remove_file(&path) {
                    if e.kind() != io::ErrorKind::NotFound {
                        self.warnings
                            .push(format!("could not remove `{name}`: {e}"));
                    }
                }
            }
            self.nodes.remove(id);
            self.computed.remove(id);
        }
        affected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeKind, Status};

    fn sample_realm_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("legend_realm_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".legend")).unwrap();
        fs::create_dir_all(dir.join("Nodes")).unwrap();
        dir
    }

    fn node_file(id: &str, kind: NodeKind, parent: Option<&str>, deps: &[&str]) -> NodeFile {
        let node = Node {
            id: id.to_string(),
            kind,
            title: format!("Title {id}"),
            status: Status::Active,
            priority: crate::model::Priority::Medium,
            disciplines: vec![],
            epics: vec![],
            quest_points: Some(1),
            landmark: None,
            parent: parent.map(|s| s.to_string()),
            blocked_by: deps.iter().map(|s| s.to_string()).collect(),
            created_at: None,
            completed_at: None,
            tags: vec![],
        };
        NodeFile {
            node,
            body: "\n## Notes\nBody text.\n".to_string(),
        }
    }

    #[test]
    fn opens_realm_and_computes_state() {
        let dir = sample_realm_dir("basic");
        let mut realm = Realm::open(&dir).unwrap();
        let f = node_file("ACT-1000", NodeKind::Action, None, &[]);
        realm.create_node_file(f, "some_action").unwrap();
        realm.recompute();
        assert_eq!(realm.nodes.len(), 1);
        let comp = &realm.computed["ACT-1000"];
        assert_eq!(comp.effective_status, "active");
        assert!(!comp.blocked);
    }

    #[test]
    fn index_sync_writes_both_artifacts() {
        let dir = sample_realm_dir("index");
        let mut realm = Realm::open(&dir).unwrap();
        let f = node_file("ACT-1001", NodeKind::Action, None, &[]);
        realm.create_node_file(f, "some_action").unwrap();
        realm.recompute();
        realm.sync_index().unwrap();
        let json = fs::read_to_string(dir.join(".legend").join("index.json")).unwrap();
        assert!(json.contains("ACT-1001"));
        let md = fs::read_to_string(dir.join(".legend").join("INDEX.md")).unwrap();
        assert!(md.contains("|ACT-1001|"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_strips_dangling_prerequisites() {
        let dir = sample_realm_dir("delete");
        let mut realm = Realm::open(&dir).unwrap();
        realm
            .create_node_file(node_file("ACT-2000", NodeKind::Action, None, &[]), "a")
            .unwrap();
        realm
            .create_node_file(
                node_file(
                    "ACT-2001",
                    NodeKind::Action,
                    Some("ACT-2000"),
                    &["ACT-2000", "ACT-2002"],
                ),
                "b",
            )
            .unwrap();
        realm.recompute();
        assert!(realm.computed["ACT-2001"]
            .validation_error
            .as_deref()
            .unwrap()
            .contains("unknown dependency `ACT-2002`"));

        let affected = realm.delete_ids(&["ACT-2000".to_string()]);
        realm.recompute();
        assert_eq!(realm.nodes.len(), 1);
        assert!(affected.contains(&"ACT-2001".to_string()));
        assert!(!realm.nodes["ACT-2001"]
            .node
            .blocked_by
            .contains(&"ACT-2000".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    const TAXONOMY_WITH_ALIASES: &str = "disciplines:\n  Programming:\n    - Locomotion\nepics:\n  Combat_Engine:\n    - Melee\naliases:\n  \"Programming/Movement\": \"Programming/Locomotion\"\n  \"Combat/Melee\": \"Combat_Engine/Melee\"\n";

    const LANDMARKS_WITH_ALIASES: &str =
        "landmarks:\n  Landmark_01_Demo:\n    title: Demo\naliases:\n  Demo_v1: Landmark_01_Demo\n";

    #[test]
    fn write_canonicalizes_legacy_aliases() {
        let dir = sample_realm_dir("canonicalize");
        fs::write(
            dir.join(".legend").join("taxonomy.yaml"),
            TAXONOMY_WITH_ALIASES,
        )
        .unwrap();
        fs::write(
            dir.join(".legend").join("landmarks.yaml"),
            LANDMARKS_WITH_ALIASES,
        )
        .unwrap();
        let mut realm = Realm::open(&dir).unwrap();

        let mut node = node_file("ACT-3000", NodeKind::Action, None, &[]).node;
        node.disciplines = vec!["Programming/Movement".to_string()];
        node.epics = vec!["Combat/Melee".to_string()];
        node.landmark = Some("Demo_v1".to_string());
        let file = NodeFile {
            node,
            body: "\n## Body\n".to_string(),
        };
        realm.create_node_file(file, "alias_test").unwrap();
        realm.recompute();
        // Alias values validate (they resolve), and suggestions are empty.
        assert!(realm.suggestions_for("ACT-3000").is_empty());

        assert!(realm.write_node("ACT-3000").is_ok());
        let text = fs::read_to_string(realm.file_path("ACT-3000").unwrap()).unwrap();
        assert!(text.contains("Programming/Locomotion"));
        assert!(!text.contains("Programming/Movement"));
        assert!(text.contains("Combat_Engine/Melee"));
        assert!(!text.contains("Combat/Melee"));
        assert!(text.contains("Landmark_01_Demo"));
        assert!(!text.contains("Demo_v1"));
        // In-memory copy now holds canonical values and the write is stable.
        assert_eq!(
            realm.nodes["ACT-3000"].node.disciplines,
            vec!["Programming/Locomotion"]
        );
        assert!(!realm.canonicalize_node("ACT-3000"), "idempotent");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_updates_title_and_file_slug() {
        let dir = sample_realm_dir("rename");
        let mut realm = Realm::open(&dir).unwrap();
        let f = node_file("ACT-3001", NodeKind::Action, None, &[]);
        realm.create_node_file(f, "old_slug_name").unwrap();

        let outcome = realm
            .rename_node("ACT-3001", "Brand New Bigger Title")
            .unwrap();
        assert!(outcome.title_changed);
        assert!(outcome.file_renamed);
        assert_eq!(outcome.new_name, "ACT-3001_brand_new_bigger_title.md");
        assert_eq!(
            outcome.old_name.as_deref(),
            Some("ACT-3001_old_slug_name.md")
        );
        assert!(!dir.join("Nodes").join("ACT-3001_old_slug_name.md").exists());
        let path = dir.join("Nodes").join("ACT-3001_brand_new_bigger_title.md");
        assert!(path.exists());
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("title: Brand New Bigger Title"));
        assert!(text.contains("id: ACT-3001"));
        assert!(text.contains("## Notes"), "body preserved");
        assert_eq!(
            realm.file_names["ACT-3001"],
            "ACT-3001_brand_new_bigger_title.md"
        );

        // Renaming to the same title is a no-op file-wise.
        let again = realm
            .rename_node("ACT-3001", "Brand New Bigger Title")
            .unwrap();
        assert!(!again.title_changed && !again.file_renamed);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn suggestions_cover_every_validation_class() {
        let dir = sample_realm_dir("suggestions");
        // No taxonomy/landmarks files -> empty registries -> everything invalid.
        let mut realm = Realm::open(&dir).unwrap();
        let mut node = node_file("ACT-3002", NodeKind::Action, Some("MISSING-PARENT"), &[]).node;
        node.disciplines = vec!["Ghost/Skill".to_string()];
        node.quest_points = Some(4);
        node.landmark = Some("Missing_Landmark".to_string());
        node.blocked_by = vec!["MISSING-DEP".to_string()];
        realm
            .create_node_file(
                NodeFile {
                    node,
                    body: "\n## Body\n".to_string(),
                },
                "suggestions",
            )
            .unwrap();
        realm.recompute();
        assert!(realm.computed["ACT-3002"].validation_error.is_some());

        let s = realm.suggestions_for("ACT-3002");
        let joined = s.join("\n");
        assert!(joined.contains("unknown discipline `Ghost/Skill`"));
        assert!(joined.contains("taxonomy.yaml"));
        assert!(joined.contains("unknown landmark `Missing_Landmark`"));
        assert!(joined.contains("parent `MISSING-PARENT` does not exist"));
        assert!(joined.contains("blocked_by references `MISSING-DEP`"));
        assert!(joined.contains("Fibonacci"));
        // A clean node (no taxonomy/landmark/dep references) has none.
        realm
            .create_node_file(node_file("ACT-3003", NodeKind::Action, None, &[]), "clean")
            .unwrap();
        realm.recompute();
        assert!(realm.suggestions_for("ACT-3003").is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    /// Loads the checked-in sample realm (integration): six nodes, an
    /// inherited landmark, and a blocked action behind the stamina bar task.
    #[test]
    fn loads_sample_realm_from_examples() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/realm-demo");
        let realm = Realm::open(&root).unwrap();
        assert_eq!(realm.nodes.len(), 6, "sample realm has 6 node files");
        assert_eq!(
            realm.sorted_ids().first().map(String::as_str),
            Some("CARD-4M1P")
        );
        assert!(realm.taxonomy.is_valid_discipline("Programming/Locomotion"));
        assert!(
            realm.landmarks.is_valid("Landmark_02_Slice"),
            "auto-extracted"
        );
        assert_eq!(
            realm.computed["ACT-3X7P"].effective_status, "blocked",
            "prerequisite ACT-8J3W is still active"
        );
        assert_eq!(
            realm.computed["ACT-3X7P"].effective_landmark.as_deref(),
            Some("Landmark_01_Demo"),
            "inherited from CARD-K9F2"
        );
        assert_eq!(
            realm.computed["CARD-K9F2"].total_qp, 9,
            "3 + 1 + 5 (leaf tasks)"
        );
        assert!(
            realm.warnings.iter().all(|w| !w.contains("skipping")),
            "all node files parse: {realm:?}"
        );
    }
}
