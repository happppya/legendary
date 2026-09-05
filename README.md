# Legendary – Local-First Game Dev Task & Lore Engine

Legendary is a local-first, Markdown-native, graph-based project management tool built for game developers. It merges tactile organization with the interconnected flexibility of node-based graph editors, using a pure ECS architecture where every node is a Markdown file with YAML frontmatter.

This repository is the **application monorepo**:

- **`crates/legend_core`** — shared Rust engine: ECS node model, frontmatter parsing, DAG traversal & cycle detection, QP aggregation, landmark inheritance, realm indexing.
- **`crates/legend_cli`** — the `legend` CLI binary (query, show, tree, update, wrap, reparent, delete, create, index sync).
- **`desktop/`** — Tauri v2 desktop shell (wired into the workspace; built explicitly once the frontend has a production `dist`).
- **`frontend/`** — Vue 3 + TypeScript + Vite renderer.
- **`examples/realm-demo/`** — a sample game *realm* (`.legend/`, `Nodes/`, `Landmarks/`, `AGENTS.md`) used for manual CLI testing and integration tests. A realm is the directory layout any game project managed by Legendary uses.
- **`notes/`** — the full product specification (`design-overview.md`, untouched master) and its split, checkbox-tracked working copy (`design-documents/`).

## Quickstart

```bash
# Engine + CLI (default workspace members)
cargo test
cargo build --release -p legend_cli     # -> target/release/legend.exe (bin name: legend)

# Try the CLI against the sample realm
cargo run -p legend_cli -- --realm examples/realm-demo index sync
cargo run -p legend_cli -- --realm examples/realm-demo query --unblocked
cargo run -p legend_cli -- --realm examples/realm-demo tree CARD-K9F2

# Frontend (Vue renderer)
cd frontend && npm install && npm run dev   # Vite dev server on http://localhost:5173
```

## Reading the spec

Agents and humans should start at [`notes/design-documents/README.md`](notes/design-documents/README.md): it routes you to the right document and explains the `- [ ]` tracking convention used to mark requirements as implemented. [`notes/design-overview.md`](notes/design-overview.md) is the canonical prose source and must not be edited.

## Status

Initialized scaffold — repo conventions, realm sample, core engine (data model, frontmatter, DAG rules, registry, indexing), and CLI read/mutation paths. See [`notes/ROADMAP.md`](notes/ROADMAP.md) for the next steps.
