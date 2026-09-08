//! Shared mutation operations over a [`Realm`] (spec §4, §6.4).
//!
//! This module is the single source of truth for *how* a mutation touches
//! the realm: every CLI command and every Tauri IPC command in the desktop
//! shell delegates here, so the two surfaces cannot drift apart. Each
//! operation persists node files, recomputes runtime DAG state, and
//! synchronously regenerates `.legend/index.json` + `.legend/INDEX.md`.
//!
//! Callers own presentation: operations return a [`MutationOutcome`]
//! (created/changed/deleted ids + a human message) rather than printing.

use crate::dag;
use crate::id::{generate_unique, slugify_title};
use crate::model::{utc_now_rfc3339, Node, NodeFile, NodeKind, Priority, Status};
use crate::realm::{Realm, RealmError};

/// Why a mutation was rejected. Rendered verbatim by both surfaces (the CLI
/// exits 1 with this message; the desktop shell returns it to the renderer).
#[derive(Debug, Clone, thiserror::Error)]
pub enum OpsError {
    #[error("no such node `{0}`")]
    NoSuchNode(String),
    #[error("`{0}` is already a Card; wrap only applies to Action, Guard or Idea nodes")]
    WrapTargetIsCard(String),
    #[error("a node cannot be its own parent")]
    SelfParent,
    #[error("unknown parent `{0}`")]
    UnknownParent(String),
    #[error("CycleDetectedError: reparenting {id} under {parent} would close a cycle")]
    CycleDetected { id: String, parent: String },
    #[error("quest_points must be Fibonacci: 1, 2, 3, 5, 8, 13, 21")]
    InvalidQuestPoints,
    #[error("`blocked` is a computed runtime state and cannot be set; resolve or vanquish its prerequisites instead")]
    BlockedNotSettable,
    #[error("unknown status `{0}` (expected unstarted, active or vanquished)")]
    UnknownStatus(String),
    #[error("unknown kind `{0}` (expected card, genre, action, guard or idea)")]
    UnknownKind(String),
    #[error("unknown priority `{0}` (expected critical, high, medium or low)")]
    UnknownPriority(String),
    #[error("node {id} has {count} descendant(s). Use recursive to delete the whole subgraph.")]
    NeedsRecursive { id: String, count: usize },
    #[error("unknown dependency `{0}`")]
    UnknownDependency(String),
    #[error("a node cannot block itself")]
    SelfDependency,
    #[error(
        "unknown discipline `{0}`: declare it under `disciplines:` in `.legend/taxonomy.yaml`"
    )]
    UnknownDiscipline(String),
    #[error("unknown epic `{0}`: declare it under `epics:` in `.legend/taxonomy.yaml`")]
    UnknownEpic(String),
    #[error("unknown landmark `{0}`: declare it in `.legend/landmarks.yaml` or drop a matching file into `Landmarks/`")]
    UnknownLandmark(String),
    #[error("Cannot vanquish Card {id}. {remaining} child tasks remaining. Use --cascade to force complete.")]
    CannotVanquish { id: String, remaining: usize },
    /// Backend realm/io/json failure, flattened to its message (these
    /// error types are neither `Clone` nor `PartialEq`).
    #[error("{0}")]
    Realm(String),
}

/// Structural equality for the variant payloads that tests (and callers)
/// match on; io/json backend errors compare by their Display strings.
impl PartialEq for OpsError {
    fn eq(&self, other: &Self) -> bool {
        use OpsError as E;
        match (self, other) {
            (E::NoSuchNode(a), E::NoSuchNode(b)) => a == b,
            (E::WrapTargetIsCard(a), E::WrapTargetIsCard(b)) => a == b,
            (E::SelfParent, E::SelfParent) => true,
            (E::UnknownParent(a), E::UnknownParent(b)) => a == b,
            (E::CycleDetected { id: a, parent: pa }, E::CycleDetected { id: b, parent: pb }) => {
                a == b && pa == pb
            }
            (E::InvalidQuestPoints, E::InvalidQuestPoints) => true,
            (E::BlockedNotSettable, E::BlockedNotSettable) => true,
            (E::UnknownStatus(a), E::UnknownStatus(b)) => a == b,
            (E::UnknownKind(a), E::UnknownKind(b)) => a == b,
            (E::UnknownPriority(a), E::UnknownPriority(b)) => a == b,
            (E::NeedsRecursive { id: a, count: ca }, E::NeedsRecursive { id: b, count: cb }) => {
                a == b && ca == cb
            }
            (E::UnknownDependency(a), E::UnknownDependency(b)) => a == b,
            (E::SelfDependency, E::SelfDependency) => true,
            (E::UnknownDiscipline(a), E::UnknownDiscipline(b)) => a == b,
            (E::UnknownEpic(a), E::UnknownEpic(b)) => a == b,
            (E::UnknownLandmark(a), E::UnknownLandmark(b)) => a == b,
            (
                E::CannotVanquish {
                    id: a,
                    remaining: ra,
                },
                E::CannotVanquish {
                    id: b,
                    remaining: rb,
                },
            ) => a == b && ra == rb,
            (E::Realm(a), E::Realm(b)) => a == b,
            _ => false,
        }
    }
}

impl From<RealmError> for OpsError {
    fn from(e: RealmError) -> Self {
        OpsError::Realm(e.to_string())
    }
}

/// What a mutation changed, in filesystem terms plus a human-readable
/// summary for status lines / toasts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MutationOutcome {
    /// Node ids whose files were rewritten (canonical frontmatter).
    pub changed: Vec<String>,
    /// Node ids created by the operation.
    pub created: Vec<String>,
    /// Node ids deleted by the operation.
    pub deleted: Vec<String>,
    /// Remaining node ids whose `blocked_by` had dangling references
    /// stripped as a side effect (subgraph delete, spec §4.5).
    pub references_stripped: Vec<String>,
    /// One-line human summary (CLI status output, renderer toast).
    pub message: String,
}

/// Per-kind unmanaged body template (doc 04 §4.1).
pub fn kind_body_template(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Card => "\n## System Context & Architectural Design\n",
        NodeKind::Genre => "\n## Genre Scope\n",
        NodeKind::Action => "\n## Action Goal\n",
        NodeKind::Guard => "\n## Quality Check Criteria\n",
        NodeKind::Idea => "\n## Idea Concept & Mechanics Exploration\n",
    }
}

/// Persist files for `ids`, then recompute runtime state and synchronously
/// rewrite both index artifacts. Every mutating operation funnels here.
fn commit(realm: &mut Realm, changed: &[String]) -> Result<(), RealmError> {
    for id in changed {
        realm.write_node(id)?;
    }
    realm.recompute();
    realm.sync_index()?;
    Ok(())
}

/// Create a node (`legend create`, spec §6.4). The engine assigns a fresh
/// Base36 id, stamps `created_at`, seeds the per-kind body template, and
/// writes the file as `Nodes/[ID]_[slug].md`. Returns the new id via
/// [`MutationOutcome::created`].
pub struct CreateArgs {
    pub kind: NodeKind,
    pub title: String,
    pub parent: Option<String>,
    pub priority: Priority,
    pub quest_points: Option<u32>,
}

pub fn create(realm: &mut Realm, args: CreateArgs) -> Result<MutationOutcome, OpsError> {
    if let Some(qp) = args.quest_points {
        if !crate::model::is_valid_quest_points(qp) {
            return Err(OpsError::InvalidQuestPoints);
        }
    }
    if let Some(p) = &args.parent {
        if !realm.nodes.contains_key(p) {
            return Err(OpsError::UnknownParent(p.clone()));
        }
    }
    let now = utc_now_rfc3339();
    let mut file = NodeFile {
        node: Node {
            id: String::new(),
            kind: args.kind,
            title: args.title.clone(),
            status: Status::Unstarted,
            priority: args.priority,
            disciplines: vec![],
            epics: vec![],
            quest_points: args.quest_points,
            landmark: None,
            parent: args.parent.clone(),
            blocked_by: vec![],
            created_at: Some(now),
            completed_at: None,
            tags: vec![],
        },
        body: kind_body_template(args.kind).to_string(),
    };
    let id = generate_unique(args.kind, |candidate| realm.nodes.contains_key(candidate));
    file.node.id = id.to_string();
    realm.create_node_file(file, &slugify_title(&args.title))?;
    commit(realm, &[])?;
    Ok(MutationOutcome {
        created: vec![id.to_string()],
        message: format!(
            "created {id} under {}",
            args.parent.as_deref().unwrap_or("root")
        ),
        ..MutationOutcome::default()
    })
}

/// Update a node's lifecycle status (`legend update --status`, spec §4.3).
/// Cards require cascade when descendant leaf tasks remain unvanquished.
pub struct UpdateArgs {
    pub id: String,
    /// The new *stored* lifecycle status (`blocked` is never stored).
    pub status: Status,
    /// Force-complete all descendant leaf tasks when vanquishing a Card.
    pub cascade: bool,
}

pub fn update(realm: &mut Realm, args: UpdateArgs) -> Result<MutationOutcome, OpsError> {
    let Some(node) = realm.nodes.get(&args.id).map(|f| f.node.clone()) else {
        return Err(OpsError::NoSuchNode(args.id.clone()));
    };

    let mut changed: Vec<String> = vec![args.id.clone()];
    if node.kind.is_container() && args.status == Status::Vanquished {
        let remaining = dag::count_unvanquished_leaves(&realm.node_map(), &args.id);
        if remaining > 0 && !args.cascade {
            return Err(OpsError::CannotVanquish {
                id: args.id.clone(),
                remaining,
            });
        }
        if args.cascade {
            let targets = dag::cascade_ids(&realm.node_map(), &args.id);
            for t in &targets {
                let f = realm.nodes.get_mut(t).unwrap();
                f.node.status = Status::Vanquished;
                f.node.completed_at = Some(utc_now_rfc3339());
            }
            changed.extend(targets);
        }
    }
    {
        let f = realm.nodes.get_mut(&args.id).unwrap();
        f.node.status = args.status;
        if args.status == Status::Vanquished {
            f.node.completed_at = Some(utc_now_rfc3339());
        } else {
            f.node.completed_at = None;
        }
    }
    changed.sort();
    changed.dedup();
    commit(realm, &changed)?;
    let extra = if changed.len() > 1 {
        format!(" (+{} cascaded)", changed.len() - 1)
    } else {
        String::new()
    };
    Ok(MutationOutcome {
        changed,
        message: format!("updated {} → {}{}", args.id, args.status, extra),
        ..MutationOutcome::default()
    })
}

/// Explicit rename (`legend rename --title`, spec §3.3): updates the `title`
/// component and re-derives the file slug; the id never changes.
pub fn rename(realm: &mut Realm, id: &str, new_title: &str) -> Result<MutationOutcome, OpsError> {
    if !realm.nodes.contains_key(id) {
        return Err(OpsError::NoSuchNode(id.to_string()));
    }
    let outcome = realm.rename_node(id, new_title)?;
    realm.recompute();
    realm.sync_index()?;
    let message = if outcome.title_changed || outcome.file_renamed {
        let mut parts: Vec<&str> = Vec::new();
        if outcome.title_changed {
            parts.push("title");
        }
        if outcome.file_renamed {
            parts.push("file slug");
        }
        format!("renamed {id} ({}): {}", parts.join(" + "), outcome.new_name)
    } else {
        format!("no change for {id} (title and slug already match)")
    };
    Ok(MutationOutcome {
        changed: vec![id.to_string()],
        message,
        ..MutationOutcome::default()
    })
}

/// Wrap a node in a new Card group ("Promote to Card", spec §4.1): the new
/// Card inherits components from the target, takes over its parent slot,
/// and the target is reparented beneath it.
pub fn wrap(
    realm: &mut Realm,
    id: &str,
    title: Option<String>,
) -> Result<MutationOutcome, OpsError> {
    let Some(target) = realm.nodes.get(id).map(|f| f.node.clone()) else {
        return Err(OpsError::NoSuchNode(id.to_string()));
    };
    if target.kind.is_container() {
        return Err(OpsError::WrapTargetIsCard(id.to_string()));
    }
    let card_title = title.unwrap_or_else(|| format!("{} (Group)", target.title));
    let now = utc_now_rfc3339();
    let mut card_file = NodeFile {
        node: Node {
            id: String::new(),
            kind: NodeKind::Card,
            title: card_title.clone(),
            status: Status::Unstarted,
            priority: target.priority,
            disciplines: target.disciplines.clone(),
            epics: target.epics.clone(),
            quest_points: None,
            landmark: target.landmark.clone(),
            parent: target.parent.clone(),
            blocked_by: vec![],
            created_at: Some(now),
            completed_at: None,
            tags: vec![],
        },
        body: kind_body_template(NodeKind::Card).to_string(),
    };
    let new_id = generate_unique(NodeKind::Card, |candidate| {
        realm.nodes.contains_key(candidate)
    });
    card_file.node.id = new_id.to_string();
    realm.create_node_file(card_file, &slugify_title(&card_title))?;
    {
        let f = realm.nodes.get_mut(id).unwrap();
        f.node.parent = Some(new_id.to_string());
    }
    commit(realm, &[id.to_string()])?;
    Ok(MutationOutcome {
        created: vec![new_id.to_string()],
        changed: vec![id.to_string()],
        message: format!("wrapped {id} under new card {new_id}"),
        ..MutationOutcome::default()
    })
}

/// Re-parent a node (or promote it to root when `parent` is `None`),
/// cycle-checked before any write (spec §4.3 strict cycle prevention).
pub fn reparent(
    realm: &mut Realm,
    id: &str,
    parent: Option<String>,
) -> Result<MutationOutcome, OpsError> {
    if !realm.nodes.contains_key(id) {
        return Err(OpsError::NoSuchNode(id.to_string()));
    }
    if let Some(p) = &parent {
        if p == id {
            return Err(OpsError::SelfParent);
        }
        if !realm.nodes.contains_key(p) {
            return Err(OpsError::UnknownParent(p.clone()));
        }
        let extra = vec![(id.to_string(), p.clone())];
        if dag::would_create_cycle(&realm.node_map(), &extra) {
            return Err(OpsError::CycleDetected {
                id: id.to_string(),
                parent: p.clone(),
            });
        }
    }
    {
        let f = realm.nodes.get_mut(id).unwrap();
        f.node.parent = parent.clone();
    }
    commit(realm, &[id.to_string()])?;
    let where_to = parent.as_deref().unwrap_or("root (parent: null)");
    Ok(MutationOutcome {
        changed: vec![id.to_string()],
        message: format!("reparented {id} → {where_to}"),
        ..MutationOutcome::default()
    })
}

/// Delete a node, optionally with its whole subgraph (spec §4.5). Dangling
/// `blocked_by` references are stripped from the survivors and their files
/// are rewritten.
pub fn delete(realm: &mut Realm, id: &str, recursive: bool) -> Result<MutationOutcome, OpsError> {
    if !realm.nodes.contains_key(id) {
        return Err(OpsError::NoSuchNode(id.to_string()));
    }
    let descendants = dag::descendants_ids(&realm.node_map(), id);
    if !descendants.is_empty() && !recursive {
        return Err(OpsError::NeedsRecursive {
            id: id.to_string(),
            count: descendants.len(),
        });
    }
    let mut doomed = vec![id.to_string()];
    doomed.extend(descendants);
    let stripped = realm.delete_ids(&doomed);
    commit(realm, &stripped)?;
    Ok(MutationOutcome {
        deleted: doomed,
        references_stripped: stripped,
        message: format!("deleted {id} (subgraph + dangling refs stripped)"),
        ..MutationOutcome::default()
    })
}

/// Overwrite a node's unmanaged Markdown body, preserving frontmatter.
pub fn set_body(realm: &mut Realm, id: &str, body: &str) -> Result<MutationOutcome, OpsError> {
    if !realm.nodes.contains_key(id) {
        return Err(OpsError::NoSuchNode(id.to_string()));
    }
    realm.nodes.get_mut(id).unwrap().body = body.to_string();
    commit(realm, &[id.to_string()])?;
    Ok(MutationOutcome {
        changed: vec![id.to_string()],
        message: format!("saved body of {id}"),
        ..MutationOutcome::default()
    })
}

/// Editable subset of the ECS components for one node (doc 05 §5.1 metadata
/// panel). `None` leaves a component untouched; supplied values replace it.
/// `kind` edits are intentionally not offered — the spec has no kind-move
/// operation (wrap/promote is the structured path).
#[derive(Debug, Clone, Default)]
pub struct ComponentEdits {
    pub priority: Option<Priority>,
    pub quest_points: Option<Option<u32>>,
    pub disciplines: Option<Vec<String>>,
    pub epics: Option<Vec<String>>,
    pub landmark: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub blocked_by: Option<Vec<String>>,
    /// `Some(None)` promotes to root (spec §4.5 batch re-parenting);
    /// cycle-checked against the *current* graph plus any new deps.
    pub parent: Option<Option<String>>,
}

/// Apply component edits to one node in a single write (`legend update`
/// component path; the desktop metadata panel's single backend).
///
/// Validation matches the graph rules: Fibonacci quest points, known
/// taxonomy branches / landmark, existing dependency and parent references,/// and strict cycle prevention for `parent`/`blocked_by` changes.
pub fn update_components(
    realm: &mut Realm,
    id: &str,
    edits: ComponentEdits,
) -> Result<MutationOutcome, OpsError> {
    let Some(existing) = realm.nodes.get(id).map(|f| f.node.clone()) else {
        return Err(OpsError::NoSuchNode(id.to_string()));
    };

    let mut next = existing.clone();
    if let Some(vs) = &edits.disciplines {
        for v in vs {
            if !realm.taxonomy.is_valid_discipline(v) {
                return Err(OpsError::UnknownDiscipline(v.clone()));
            }
        }
    }
    if let Some(vs) = &edits.epics {
        for v in vs {
            if !realm.taxonomy.is_valid_epic(v) {
                return Err(OpsError::UnknownEpic(v.clone()));
            }
        }
    }
    if let Some(Some(v)) = &edits.landmark {
        if !realm.landmarks.is_valid(v) {
            return Err(OpsError::UnknownLandmark(v.clone()));
        }
    }
    if let Some(deps) = &edits.blocked_by {
        if deps.iter().any(|d| d == id) {
            return Err(OpsError::SelfDependency);
        }
        if let Some(missing) = deps.iter().find(|d| !realm.nodes.contains_key(*d)) {
            return Err(OpsError::UnknownDependency(missing.clone()));
        }
        let mut deduped = deps.clone();
        deduped.sort();
        deduped.dedup();
        next.blocked_by = deduped;
    }
    if let Some(Some(p)) = &edits.parent {
        if p == id {
            return Err(OpsError::SelfParent);
        }
        if !realm.nodes.contains_key(p) {
            return Err(OpsError::UnknownParent(p.clone()));
        }
    }
    if let Some(qp) = &edits.quest_points {
        if let Some(v) = qp {
            if !crate::model::is_valid_quest_points(*v) {
                return Err(OpsError::InvalidQuestPoints);
            }
        }
        next.quest_points = *qp;
    }
    if let Some(v) = &edits.priority {
        next.priority = *v;
    }
    if let Some(v) = &edits.disciplines {
        next.disciplines = v.clone();
    }
    if let Some(v) = &edits.epics {
        next.epics = v.clone();
    }
    if let Some(v) = &edits.landmark {
        next.landmark = v.clone();
    }
    if let Some(v) = &edits.tags {
        next.tags = v.clone();
    }
    if let Some(parent) = &edits.parent {
        next.parent = parent.clone();
    }

    // Cycle checks against the current graph + the pending structural
    // changes (parent edge and any new dependency edges).
    if edits.parent.is_some() || edits.blocked_by.is_some() {
        let mut graph = realm.node_map();
        graph.insert(id.to_string(), next.clone());
        if dag::cyclic_ids(&graph).contains(&id.to_string()) {
            return Err(OpsError::CycleDetected {
                id: id.to_string(),
                parent: next
                    .parent
                    .clone()
                    .or_else(|| next.blocked_by.first().cloned())
                    .unwrap_or_else(|| "?".to_string()),
            });
        }
    }

    // Persist components (canonicalizes aliases on the way out).
    realm.nodes.get_mut(id).unwrap().node = next;
    commit(realm, &[id.to_string()])?;
    Ok(MutationOutcome {
        changed: vec![id.to_string()],
        message: format!("updated components of {id}"),
        ..MutationOutcome::default()
    })
}

/// Recompute + rewrite both index artifacts after manual external edits and
/// collect a validation report (clean, or per-node issues + fix suggestions).
pub fn index_sync(realm: &mut Realm) -> Result<Vec<(String, Vec<String>)>, RealmError> {
    realm.recompute();
    realm.sync_index()?;
    let issues: Vec<(String, Vec<String>)> = realm
        .sorted_ids()
        .into_iter()
        .filter(|id| realm.computed[id].validation_error.is_some())
        .map(|id| {
            let id_str = id.as_str();
            (id.to_string(), realm.suggestions_for(id_str))
        })
        .collect();
    Ok(issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn sample_realm_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("legend_ops_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".legend")).unwrap();
        fs::create_dir_all(dir.join("Nodes")).unwrap();
        dir
    }

    fn open_fresh(name: &str) -> (PathBuf, Realm) {
        let dir = sample_realm_dir(name);
        let realm = Realm::open(&dir).unwrap();
        (dir, realm)
    }

    fn create_action(realm: &mut Realm, title: &str, parent: Option<&str>) -> String {
        let out = create(
            realm,
            CreateArgs {
                kind: NodeKind::Action,
                title: title.to_string(),
                parent: parent.map(|s| s.to_string()),
                priority: Priority::Medium,
                quest_points: Some(1),
            },
        )
        .unwrap();
        out.created[0].clone()
    }

    #[test]
    fn create_writes_file_and_index() {
        let (dir, mut realm) = open_fresh("create");
        let id = create_action(&mut realm, "Test Create Action", None);
        assert!(id.starts_with("ACT-"));
        let path = dir
            .join("Nodes")
            .join(format!("{id}_test_create_action.md"));
        assert!(path.exists());
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("title: Test Create Action"));
        assert!(text.contains("## Action Goal"), "body template seeded");
        assert!(text.contains("created_at:"));
        assert!(dir.join(".legend").join("index.json").exists());
        assert!(dir.join(".legend").join("INDEX.md").exists());
        assert_eq!(
            realm.computed[&id].effective_status, "unstarted",
            "fresh node is unstarted"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn create_rejects_unknown_parent_and_bad_qp() {
        let (_, mut realm) = open_fresh("create_bad");
        assert_eq!(
            create(
                &mut realm,
                CreateArgs {
                    kind: NodeKind::Action,
                    title: "x".into(),
                    parent: Some("CARD-0000".into()),
                    priority: Priority::Medium,
                    quest_points: None,
                }
            )
            .unwrap_err(),
            OpsError::UnknownParent("CARD-0000".into())
        );
        assert_eq!(
            create(
                &mut realm,
                CreateArgs {
                    kind: NodeKind::Action,
                    title: "x".into(),
                    parent: None,
                    priority: Priority::Medium,
                    quest_points: Some(4),
                }
            )
            .unwrap_err(),
            OpsError::InvalidQuestPoints
        );
    }

    #[test]
    fn update_vanquish_flow_with_cascade() {
        let (dir, mut realm) = open_fresh("update");
        let card = create(
            &mut realm,
            CreateArgs {
                kind: NodeKind::Card,
                title: "Vanquish Flow".into(),
                parent: None,
                priority: Priority::Medium,
                quest_points: None,
            },
        )
        .unwrap()
        .created
        .remove(0);
        let act = create_action(&mut realm, "Leaf", Some(&card));

        // Non-cascade vanquish of a Card with an unvanquished leaf fails.
        let err = update(
            &mut realm,
            UpdateArgs {
                id: card.clone(),
                status: Status::Vanquished,
                cascade: false,
            },
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("1 child tasks remaining"),
            "got: {err}"
        );

        // The leaf vanquishes directly.
        update(
            &mut realm,
            UpdateArgs {
                id: act.clone(),
                status: Status::Vanquished,
                cascade: false,
            },
        )
        .unwrap();
        assert_eq!(realm.computed[&act].effective_status, "vanquished");
        assert!(realm.nodes[&act].node.completed_at.is_some());

        // Now the card vanquishes without cascade.
        update(
            &mut realm,
            UpdateArgs {
                id: card.clone(),
                status: Status::Vanquished,
                cascade: false,
            },
        )
        .unwrap();
        assert_eq!(realm.computed[&card].effective_status, "vanquished");

        // Cascading marks every descendant leaf.
        let card2 = create(
            &mut realm,
            CreateArgs {
                kind: NodeKind::Card,
                title: "Cascade Flow".into(),
                parent: None,
                priority: Priority::Medium,
                quest_points: None,
            },
        )
        .unwrap()
        .created
        .remove(0);
        let a = create_action(&mut realm, "A", Some(&card2));
        let b = create_action(&mut realm, "B", Some(&card2));
        let out = update(
            &mut realm,
            UpdateArgs {
                id: card2.clone(),
                status: Status::Vanquished,
                cascade: true,
            },
        )
        .unwrap();
        assert!(
            out.changed.contains(&a) && out.changed.contains(&b) && out.changed.contains(&card2)
        );
        assert_eq!(realm.nodes[&a].node.status, Status::Vanquished);
        assert_eq!(realm.nodes[&b].node.status, Status::Vanquished);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_rejects_blocked_status() {
        let (dir, mut realm) = open_fresh("update_blocked");
        let act = create_action(&mut realm, "X", None);
        assert_eq!(
            parse_then_update(&mut realm, &act, "blocked"),
            Err(OpsError::BlockedNotSettable)
        );
        let _ = fs::remove_dir_all(&dir);
    }

    /// Parse a raw status string the way both surfaces do before calling
    /// [`update`] (shared rejection of the computed `blocked` state).
    fn parse_then_update(
        realm: &mut Realm,
        id: &str,
        status: &str,
    ) -> Result<MutationOutcome, OpsError> {
        let status = if status == "blocked" {
            return Err(OpsError::BlockedNotSettable);
        } else {
            status
                .parse::<Status>()
                .map_err(|_| OpsError::UnknownStatus(status.to_string()))?
        };
        update(
            realm,
            UpdateArgs {
                id: id.to_string(),
                status,
                cascade: false,
            },
        )
    }

    #[test]
    fn wrap_creates_inheriting_card() {
        let (dir, mut realm) = open_fresh("wrap");
        let act = create_action(&mut realm, "Wrappable Task", None);
        let out = wrap(&mut realm, &act, None).unwrap();
        let card = out.created[0].clone();
        assert!(card.starts_with("CARD-"));
        assert_eq!(realm.nodes[&card].node.title, "Wrappable Task (Group)");
        assert_eq!(
            realm.nodes[&act].node.parent.as_deref(),
            Some(card.as_str())
        );
        // Wrap of a Card is rejected.
        let err = wrap(&mut realm, &card, None).unwrap_err();
        assert!(matches!(err, OpsError::WrapTargetIsCard(_)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reparent_rejects_cycles_and_unknown() {
        let (dir, mut realm) = open_fresh("reparent");
        let a = create(
            &mut realm,
            CreateArgs {
                kind: NodeKind::Card,
                title: "A".into(),
                parent: None,
                priority: Priority::Medium,
                quest_points: None,
            },
        )
        .unwrap()
        .created
        .remove(0);
        let b = create(
            &mut realm,
            CreateArgs {
                kind: NodeKind::Card,
                title: "B".into(),
                parent: Some(a.clone()),
                priority: Priority::Medium,
                quest_points: None,
            },
        )
        .unwrap()
        .created
        .remove(0);
        // b → a is fine; a → b closes a cycle.
        assert_eq!(
            reparent(&mut realm, &a, Some(b.clone())).unwrap_err(),
            OpsError::CycleDetected {
                id: a.clone(),
                parent: b.clone()
            }
        );
        assert!(
            realm.nodes[&b].node.parent.is_some(),
            "failed op wrote nothing"
        );
        // Promote to root works.
        reparent(&mut realm, &b, None).unwrap();
        assert_eq!(realm.nodes[&b].node.parent, None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_requires_recursive_and_strips_refs() {
        let (dir, mut realm) = open_fresh("delete");
        let card = create(
            &mut realm,
            CreateArgs {
                kind: NodeKind::Card,
                title: "Doomed".into(),
                parent: None,
                priority: Priority::Medium,
                quest_points: None,
            },
        )
        .unwrap()
        .created
        .remove(0);
        let child = create_action(&mut realm, "Child", Some(&card));
        let other = create_action(&mut realm, "Other", None);
        realm.nodes.get_mut(&other).unwrap().node.blocked_by = vec![child.clone()];
        realm.recompute();

        let err = delete(&mut realm, &card, false).unwrap_err();
        assert!(matches!(err, OpsError::NeedsRecursive { count: 1, .. }));

        let out = delete(&mut realm, &card, true).unwrap();
        assert_eq!(out.deleted.len(), 2);
        assert!(out.references_stripped.contains(&other));
        assert!(!realm.nodes.contains_key(&child));
        assert!(realm.nodes[&other].node.blocked_by.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_via_ops_updates_title_and_slug() {
        let (dir, mut realm) = open_fresh("rename");
        let act = create_action(&mut realm, "First Title", None);
        let out = rename(&mut realm, &act, "Second Bigger Title").unwrap();
        assert!(out.message.contains("title + file slug"));
        assert!(out.message.contains("second_bigger_title"));
        assert_eq!(realm.nodes[&act].node.title, "Second Bigger Title");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn set_body_persists_and_preserves_frontmatter() {
        let (dir, mut realm) = open_fresh("set_body");
        let act = create_action(&mut realm, "Body Test", None);
        let out = set_body(&mut realm, &act, "\n## Rewritten\n\nNew content.\n").unwrap();
        assert_eq!(out.changed, vec![act.clone()]);
        let path = dir.join("Nodes").join(format!("{act}_body_test.md"));
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("title: Body Test"));
        assert!(text.contains("## Rewritten"));
        assert!(!text.contains("## Action Goal"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_components_edits_and_validates() {
        let (dir, mut realm) = open_fresh("components");
        let act = create_action(&mut realm, "Editable", None);

        update_components(
            &mut realm,
            &act,
            ComponentEdits {
                priority: Some(Priority::High),
                quest_points: Some(Some(8)),
                tags: Some(vec!["movement".into(), "core".into()]),
                ..Default::default()
            },
        )
        .unwrap();
        let n = &realm.nodes[&act].node;
        assert_eq!(n.priority, Priority::High);
        assert_eq!(n.quest_points, Some(8));
        assert_eq!(n.tags, vec!["movement".to_string(), "core".to_string()]);

        // Non-Fibonacci QP is rejected.
        assert_eq!(
            update_components(
                &mut realm,
                &act,
                ComponentEdits {
                    quest_points: Some(Some(4)),
                    ..Default::default()
                }
            )
            .unwrap_err(),
            OpsError::InvalidQuestPoints
        );

        // Unknown dependency and self-dependency are rejected.
        assert_eq!(
            update_components(
                &mut realm,
                &act,
                ComponentEdits {
                    blocked_by: Some(vec!["ACT-0000".into()]),
                    ..Default::default()
                }
            )
            .unwrap_err(),
            OpsError::UnknownDependency("ACT-0000".into())
        );
        assert_eq!(
            update_components(
                &mut realm,
                &act,
                ComponentEdits {
                    blocked_by: Some(vec![act.clone()]),
                    ..Default::default()
                }
            )
            .unwrap_err(),
            OpsError::SelfDependency
        );

        // Cycle through parent change: make a Card, child under it, then
        // try to parent the Card under the child.
        let card = create(
            &mut realm,
            CreateArgs {
                kind: NodeKind::Card,
                title: "Parent Card".into(),
                parent: None,
                priority: Priority::Medium,
                quest_points: None,
            },
        )
        .unwrap()
        .created
        .remove(0);
        update_components(
            &mut realm,
            &act,
            ComponentEdits {
                parent: Some(Some(card.clone())),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            realm.nodes[&act].node.parent.as_deref(),
            Some(card.as_str())
        );
        assert_eq!(
            update_components(
                &mut realm,
                &card,
                ComponentEdits {
                    parent: Some(Some(act.clone())),
                    ..Default::default()
                }
            )
            .unwrap_err(),
            OpsError::CycleDetected {
                id: card.clone(),
                parent: act.clone()
            }
        );

        // Clearing quest points and promoting to root both work.
        update_components(
            &mut realm,
            &act,
            ComponentEdits {
                quest_points: Some(None),
                parent: Some(None),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(realm.nodes[&act].node.quest_points, None);
        assert_eq!(realm.nodes[&act].node.parent, None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn index_sync_reports_clean_and_issue_realm() {
        let (dir, mut realm) = open_fresh("sync");
        create_action(&mut realm, "Clean", None);
        let issues = index_sync(&mut realm).unwrap();
        assert!(issues.is_empty(), "clean realm: {issues:?}");

        // An unknown parent shows up as a validation issue with a fix hint.
        let bad = create_action(&mut realm, "Bad", None);
        realm.nodes.get_mut(&bad).unwrap().node.parent = Some("CARD-0000".into());
        realm.recompute();
        let issues = index_sync(&mut realm).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].0, bad);
        assert!(issues[0].1.iter().any(|s| s.contains("does not exist")));
        let _ = fs::remove_dir_all(&dir);
    }
}
