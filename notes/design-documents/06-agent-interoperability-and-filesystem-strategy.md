## 6. Agent Interoperability & Filesystem Strategy

### 6.1. Manifest & Index Pattern

- [ ] **Central Taxonomy & Landmarks (`.legend/taxonomy.yaml` & `.legend/landmarks.yaml`):** Provides AI agents with declarative registries of valid Disciplines, Epics, and Landmarks.

- [ ] **Auto-Generated Index (`.legend/index.json`):** On every save, UI edit, or CLI write mutation, the system **synchronously updates** a JSON document containing all node metadata, effective landmarks, dynamic computed blocked states, and relative paths.

- [ ] **Agent Readme (`.legend/INDEX.md`):** Human- and agent-readable Markdown table listing active Actions, blocked Guards, speculative Ideas, and high-level Cards, updated synchronously alongside `index.json`.

### 6.2. Flat Directory Advantages for Agents

All task files reside directly inside `Nodes/`. AI agents do not need to perform recursive directory walks. Moving an entity across categories requires updating YAML frontmatter _only_.

### 6.3. Self-Describing File Naming Scheme

File names follow `[KIND]-[BASE36_HASH]_[slug].md` (e.g., `Nodes/CARD-K9F2_player_controller.md`, `Nodes/IDEA-5T2P_grappling_hook_mechanic.md`).

### 6.4. Agent CLI Query & Mutation Tool (`legend` CLI)

Zero-dependency native Rust binary (`bin/legend` or `legend` executable) compiled directly from the shared core Rust engine crate.

#### Synchronous Index Management Rule:

All CLI write and mutation commands (`update`, `wrap`, `create`, `delete`, `reparent`) **MUST synchronously rewrite `.legend/index.json` and `.legend/INDEX.md` before returning exit status 0**.

#### Key Commands:

```
# Query nodes by discipline branch
legend query --discipline "Programming"

# Query nodes by epic tree branch
legend query --epic "Combat_Engine"

# Return unblocked, active tasks formatted as JSON
legend query --status active --unblocked --json

# List quality gate Guard nodes holding up a specific Landmark
legend query --kind guard --landmark Landmark_01_Demo --status blocked

# Query speculative Idea nodes
legend query --kind idea

# View full node metadata, effective landmark, and runtime status
legend show ACT-3X7P

# Print ancestor/descendant tree
legend tree CARD-K9F2

# Mark an action as vanquished (synchronously updates completed_at and indexes)
legend update ACT-3X7P --status vanquished

# Mark a card as vanquished (requires all child tasks complete unless --cascade is supplied)
legend update CARD-K9F2 --status vanquished --cascade

# Wrap node with Card group (synchronously re-indexes graph)
legend wrap ACT-3X7P --title "Refactor Dash Controller"

# Re-parent a node or sub-tree safely
legend reparent ACT-3X7P --parent CARD-4M1P

# Delete node or subgraph safely (cleans up blocked_by references)
legend delete CARD-K9F2 --recursive

# Force rebuild index files and perform cycle validation check after manual external edits
legend index sync
```

### 6.5. Root `AGENTS.md` Instruction File

```
# Agent Task Engine Instructions

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
```
