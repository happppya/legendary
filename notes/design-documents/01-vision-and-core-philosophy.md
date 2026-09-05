## 1. Vision & Core Philosophy

This application is a local-first, Markdown-native, graph-based project management tool designed specifically for game developers. It merges tactile organization with the infinite depth and interconnected flexibility of node-based graph editors (like Obsidian).

### Key Principles

- [ ] **Local-First & Plain-Text:** All data resides on the user's filesystem as standard Markdown (`.md`) files with YAML frontmatter. Zero vendor lock-in, fully version-controllable via Git.

- [ ] **Pure ECS Architecture (Entities & Components):** Graph nodes are light entities composed of modular frontmatter components. Functional attributes like Priority, Status, Disciplines, Epics, Quest Points, and Dependencies are treated as distinct components attached to an entity.

- [ ] **Flat Filesystem Storage:** All node entities reside directly in a single `Nodes/` folder, removing filesystem nesting. Dynamic grouping, filtering, and visual hierarchies are managed in-app via ECS components rather than filesystem folder paths.

- [ ] **Heterogeneous Node Types:** Tasks are split into structural **Card** groups, executable **Action** nodes, quality-gate **Guard** nodes, and speculative **Idea** nodes within a Directed Acyclic Graph (DAG).

- [ ] **Flexible Parent Grouping:** While **Card** nodes are the primary recommended organizational containers, _any_ node entity can technically serve as a parent group node if child entities set its ID in their `parent` key.

- [ ] **Functional Groupings & Checkpoints:** Projects are organized into logical feature domains (**Epics**) and targeted completion goals (**Landmarks**), eliminating strict artificial sprint cycles in favor of continuous flow and clear milestone targets.

- [ ] **Keyboard-First & Fluid Interaction:** Complete feature parity between mouse-driven graph operations and keyboard shortcuts, anchored by an instant universal Command Palette.

- [ ] **AI Agent First:** The filesystem layout, file naming conventions, metadata indexes, and CLI interface are designed so AI tools (Cursor, Claude Desktop, CLI scripts) can inspect, query, and modify the graph using built-in app logic without reinventing frontmatter parsing or DAG traversal.
