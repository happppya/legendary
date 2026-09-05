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
