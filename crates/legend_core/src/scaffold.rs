//! Realm scaffolding (`legend init <dir>`, spec §3.4 layout + §6.5 agent
//! readme). A fresh realm starts with the same flat layout Legendary manages:
//!
//! ```text
//! realm/
//! ├── AGENTS.md            # agent instructions for this realm
//! ├── .legend/
//! │   ├── taxonomy.yaml    # starter canonical Epics & Disciplines
//! │   └── landmarks.yaml   # starter registry (auto-extraction enabled)
//! ├── Landmarks/           # *.md milestone declarations
//! └── Nodes/               # flat entity database
//! ```
//!
//! The starter registries mirror the canonical examples from design doc 03
//! §3.2 so a bootstrapped realm can immediately create nodes whose paths are
//! valid; the `Landmarks/` folder stays empty and accepts milestone files.

use std::fs;
use std::io;
use std::path::Path;

/// Root `AGENTS.md` for a fresh realm (spec §6.5). Agents operating on the
/// project read this file to learn how to query and mutate the graph.
pub const AGENTS_MD: &str = r#"# Agent Task Engine Instructions

This repository uses a local-first Markdown DAG task manager based on a pure ECS node graph.

## Preferred Interaction Method (CLI Tool)
- You can query and update the project graph using the bundled CLI tool: `bin/legend` or `legend`.
- Run `legend query --unblocked` to find tasks ready for implementation.
- Run `legend query --epic "Combat_Engine"` to list tasks under a specific feature epic tree.
- Run `legend query --kind idea` to find speculative feature concepts and pitches.
- Run `legend show [ID]` to view node context and parent/child relationships.
- Run `legend update [ID] --status vanquished` when completing an Action, Guard, or Idea node.
- Note: To vanquish a Card node, all child tasks must already be vanquished, or pass `--cascade` to force-complete all child tasks.
- Run `legend wrap [ID]` if an action or idea needs to be expanded into a Card group with child subtasks.
- Run `legend reparent [ID] --parent [PARENT_ID]` to re-organize nodes under a new parent Card.

## Taxonomy & Landmark Validation Rules
- Epics and Disciplines are canonically defined in `.legend/taxonomy.yaml`.
- Landmarks are canonically defined in `.legend/landmarks.yaml` or auto-extracted from `Landmarks/`.

## Node Storage & Layout (Direct Editing)
- ALL entity files reside directly in `Nodes/` as flat Markdown files (`Nodes/[ID]_[slug].md`).
- NEVER create nested subfolders inside `Nodes/`.

## Node Types
- `card`: Container/Group node for context and child organization (recommended group type).
- `action`: Discrete execution task with Quest Points (`quest_points`).
- `guard`: Quality gate node (Code Review, QA, Testing).
- `idea`: Speculative feature pitch or concept exploration node.
- Note: Any node type can technically serve as a parent container if child nodes reference its ID in `parent`.

## Components & Frontmatter
- `status`: Single enum (`unstarted`, `active`, `vanquished`). Note: `blocked` is a computed runtime state.
- `priority`: Single enum (`critical`, `high`, `medium`, `low`).
- `disciplines`: Array of hierarchical string paths (e.g. `["Programming/View", "Art/UI"]`).
- `epics`: Array of hierarchical string paths (e.g. `["Combat_Engine/Locomotion", "Core_Systems/Player"]`).
- `landmark`: Landmark assignment (`null` inherits landmark from parent Card across nesting levels).
- `parent`: Single ID reference to container parent node.
- `blocked_by`: Array of prerequisite node IDs.

## Rules for Editing Nodes Directly
- NEVER remove YAML frontmatter keys (`id`, `kind`, `status`, `priority`, `disciplines`, `epics`, `parent`, `blocked_by`, `landmark`).
- The `id` key (e.g. `ACT-3X7P`, `IDEA-5T2P`) is immutable. Filenames do not need to be renamed when titles change.
- Never manually introduce dependency or parent loops (`A -> B -> A`).
- When completing an Action, Guard, or Idea node:
  - Set `status: vanquished`
  - Set `completed_at: "YYYY-MM-DDTHH:MM:SSZ"`
- Run `legend index sync` if you manually edit Markdown files directly instead of using the `legend` CLI.
"#;

/// Starter canonical taxonomy (spec §3.2 A sample). Real projects edit this
/// file as their domains evolve.
pub const STARTER_TAXONOMY_YAML: &str = r#"# .legend/taxonomy.yaml
disciplines:
  Programming:
    - Locomotion
    - Input
    - Systems
    - Optimization
    - View
  Art:
    - Concept
    - 3D_Models
    - UI
  QualityAssurance:
    - Automated
    - Playtesting

epics:
  Combat_Engine:
    - Locomotion
    - Melee
    - Abilities
  Core_Systems:
    - Player
    - SaveSystem
  UI:
    - HUD
    - Menus

# Branch aliases map legacy or renamed paths without rewriting markdown files
aliases:
  "Programming/Movement": "Programming/Locomotion"
  "Combat/Melee": "Combat_Engine/Melee"
"#;

/// Starter landmark registry: empty by design. Landmarks are declared here or
/// auto-extracted from Markdown files dropped into `Landmarks/` (spec §3.2 B).
pub const STARTER_LANDMARKS_YAML: &str = r#"# .legend/landmarks.yaml
# Declare build checkpoints under `landmarks:`, for example:
#
#   landmarks:
#     Landmark_01_Demo:
#       title: "Steam Playtest Demo"
#       target_date: "2026-11-15"
#
# Alternatively, drop a `Landmark_Name.md` file into `Landmarks/` and it is
# auto-extracted into the registry on the next index.
#
# Aliases map legacy or renamed landmark ids without rewriting node files:
#   aliases:
#     "Demo_v1": "Landmark_01_Demo"
landmarks: {}
aliases: {}
"#;

/// Create the realm directory layout at `root`.
///
/// `root` may not exist yet or may be an empty directory; an existing
/// non-empty directory is refused so a real project is never touched.
pub fn scaffold(root: &Path) -> io::Result<()> {
    if root.exists() {
        if !root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("`{}` exists and is not a directory", root.display()),
            ));
        }
        let mut entries = fs::read_dir(root)?;
        if entries.next().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "`{}` is not empty; init expects a new or empty directory",
                    root.display()
                ),
            ));
        }
    } else {
        fs::create_dir_all(root)?;
    }

    fs::create_dir_all(root.join(".legend"))?;
    fs::create_dir_all(root.join("Nodes"))?;
    fs::create_dir_all(root.join("Landmarks"))?;
    fs::write(
        root.join(".legend").join("taxonomy.yaml"),
        STARTER_TAXONOMY_YAML,
    )?;
    fs::write(
        root.join(".legend").join("landmarks.yaml"),
        STARTER_LANDMARKS_YAML,
    )?;
    fs::write(root.join("AGENTS.md"), AGENTS_MD)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm::Realm;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("legend_scaffold_{name}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn scaffolds_layout_into_new_directory() {
        let dir = temp_root("fresh");
        scaffold(&dir).unwrap();
        assert!(dir.join("AGENTS.md").is_file());
        assert!(dir.join(".legend").join("taxonomy.yaml").is_file());
        assert!(dir.join(".legend").join("landmarks.yaml").is_file());
        assert!(dir.join("Nodes").is_dir());
        assert!(dir.join("Landmarks").is_dir());

        // The realm opens and indexes immediately (zero nodes, valid starter
        // taxonomy so future creates validate).
        let realm = Realm::open(&dir).unwrap();
        assert!(realm.nodes.is_empty());
        assert!(realm.taxonomy.is_valid_discipline("Programming/Locomotion"));
        assert!(realm.taxonomy.is_valid_epic("UI/HUD"));
        assert!(
            realm.landmarks.landmark_names().is_empty(),
            "no landmark files yet"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn refuses_non_empty_directory() {
        let dir = temp_root("nonempty");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("existing.txt"), "keep me").unwrap();
        let err = scaffold(&dir).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert!(dir.join("existing.txt").exists(), "must not touch contents");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn refuses_file_path() {
        let dir = temp_root("filepath");
        fs::write(&dir, "not a dir").unwrap();
        let err = scaffold(&dir).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        let _ = fs::remove_dir_all(&dir);
    }
}
