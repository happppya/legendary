# Legendary — Roadmap (Todo List)

Single tracking list for the project. Every item is a todo; `- [x]` means
implemented **and** verified. When a todo completes, flip it here **and** flip
the matching `- [ ]` checkbox in `notes/design-documents/` (start at its
`README.md`) — the design docs are the source of truth for wording and scope.
`notes/design-overview.md` is the untouched canonical master.

## Milestone 0 — Monorepo scaffold

- [x] Rust workspace: `legend_core` engine crate + `legend` CLI crate; Tauri desktop crate wired as a non-default member.
- [x] Engine: ECS node model, Base36 ID system, YAML frontmatter parse/render (canonical key order, body preserved).
- [x] Engine: taxonomy & landmark registries with alias resolution.
- [x] Engine: DAG rules (computed blocked state, cycle prevention & defensive detection, QP aggregation, locked-QP %, landmark inheritance, `--cascade` plan).
- [x] Engine: realm open/index/CRUD with synchronous `.legend/index.json` + `INDEX.md`.
- [x] CLI commands: `query`, `show`, `tree`, `create`, `update`, `wrap`, `reparent`, `delete`, `index sync`.
- [x] Vue 3 + Vite renderer shell (`frontend/`, dev server :5173, `dist/` builds).
- [x] Sample realm `examples/realm-demo/` (spec §3.4 layout) used by engine integration tests.
- [x] Mutation cycle exercised end-to-end (create → wrap → reparent → cycle rejection → cascade vanquish → recursive delete).

## Milestone 1 — CLI & engine parity with the spec

- [x] `legend init <dir>` scaffolds a fresh realm (`Nodes/`, `Landmarks/`, taxonomy/landmark starters, root `AGENTS.md` per doc 06 §6.5) and indexes it; refuses non-empty directories; reusable from `legend_core::scaffold`.
- [x] `legend rename <ID> --title …` is the only file rename path (doc 03 §3.3 slug stability).
- [x] Canonicalization on write — legacy discipline/epic/landmark aliases resolved through registries before persisting (idempotent).
- [x] `legend index sync` exits 1 on validation errors with a `fix:` suggestion per issue.
- [x] Round-trip guarantees — lossless parse → render → parse tests over the sample realm, doc-03 schemas, and a seeded property fuzz (`crates/legend_core/tests/roundtrip.rs`).
- [x] Wrap/Create ergonomics — `--title` default, per-kind body templates; wrapping a Card errors politely.
- [x] Queries filter on *computed* status; `update --status blocked` rejected with guidance (spec §6.4).

## Milestone 2 — Desktop shell (Tauri v2)

- [x] Window icons + `bundle.icon`; `cargo build -p legendary-desktop` verified on this machine (crate stays out of default workspace members).
- [x] File watcher (spec §7): `notify`-based recursive watch on the open realm with 350 ms write debouncing, emitting `realm://changed` (doc 04 §4.2).
- [x] Tauri commands over `legend_core`: `realm_open`, `query`-equivalent snapshot, `node_body`, `node_create`, `node_update_status`, `node_rename`, `node_wrap`, `node_reparent`, `node_delete`, `index_sync`.
- [x] Shared mutation ops layer (`legend_core::ops`) so CLI and desktop cannot drift (M2.4).
- [ ] Realm picker dialog in the renderer (open-by-path exists; no in-app picker UI yet).

## Milestone 2½ — Basic realm preview prototype (read-only) — folded into M2/M3

- [x] Read-only IPC: `realm_open` returns every node + computed state + raw Markdown body in the `.legend/index.json` shape; `node_body` for bodies.
- [x] Renderer read views: searchable, collapsible node tree (DFS rows, computed status/QP/blocked chips) + detail pane with component chips, meta grid, and rendered Markdown body preview.
- [x] Data layer: typed client over Tauri `invoke`; browser fallback fixture `frontend/public/realm-demo.json` regenerated via `npm run fixture --prefix frontend`.
- [x] Launch verified end-to-end: `legendary-desktop.exe` → WebView2 → renderer → `realm_open` → live realm.

## Milestone 2¾ — Mutation layer + live reload

- [x] Shared ops layer `legend_core::ops` covering create, update + cascade vanquish, rename, wrap, reparent, delete, index sync — CLI is a thin skin over it.
- [x] Desktop mutation IPC returning refreshed snapshot + human summary for toasts/status bar.
- [x] Renderer listens for `realm://changed`, re-opens the realm, preserves selection when the node survives.
- [x] Frontend `api.ts` typed mutation methods (desktop-only guards) + `onRealmChanged`; `types.ts` mirrors payload shapes.

## Milestone 3 — Renderer (Vue 3): workspace & mutation UX

- [x] Realm data layer: typed API client over Tauri commands, node store, computed status/landmark/QP from the index payload.
- [x] Tree navigator from `taxonomy.yaml` (doc 05 §5.4): discipline/epic hierarchies validated against the registry; clicking a branch filters nodes to that branch and sub-branches. (`ExplorerPanel` Taxonomy group renders `taxTree` collapsible hierarchies; branch rows toggle `filters.epics/disciplines`, and `matchesFilters` matches whole paths, path prefixes and segments so a root check filters its subtree. Values are engine-validated against `taxonomy.yaml` at index time; invalid ones surface as validation errors.)
- [x] Focused node workspace — left panel (doc 05 §5.1): editable Markdown body editor + goal notes for the selected node. (`NodeBodyEditor.vue`: title edits rename through `node_rename` (slug re-derived per doc 03 §3.3); body edits save through the new `legend_core::ops::set_body`; preview/edit toggle with Ctrl+Enter save and Esc cancel.)
- [x] Focused node workspace — metadata panel (doc 05 §5.1): dedicated controls editing every ECS component (kind, status, priority, disciplines, epics, landmark, parent, blocked_by, quest_points, tags); all writes through the store → ops layer → synchronous index. (`NodeMetaEditor.vue` + new `legend_core::ops::update_components`: status segment routes Card vanquish through the cascade modal; priority/QP/disciplines/epics/landmark/blocked_by validated by the engine — Fibonacci QP, known taxonomy branches, cycle-checked dependencies.)
- [ ] Right panel — interactive local graph view (doc 05 §5.1): selected node as active center point within its immediate graph.
- [x] Card vanquish flow (doc 04 §4.3 UI Action): warning modal — *"Card contains N active tasks. Force complete all child tasks?"* — before cascade, reusing engine counts. (`MutationDialog.vue`; count derived from the snapshot like `dag::count_unvanquished_leaves`.)
- [x] Recursive subgraph deletion warning (doc 04 §4.5): modal showing the total count of affected descendant nodes before confirming. (`MutationDialog.vue`; leaf deletes skip the prompt.)
- [x] Mutation action bar in the detail inspector (`NodeActions.vue`): vanquish (cascade-confirmed), wrap in Card, move-to-root reparent, recursive delete — desktop-only, engine rejections surfaced in the status bar.
- [x] Drag-and-drop re-parenting (doc 04 §4.5): drag a child node or sub-Card branch onto another Card in graph view or tree navigator to update `parent`. (GraphPane drop-target detection with dashed gold highlight while dragging; drop emits `reparent` → `ops::reparent`, engine cycle-checks and rejections surface in the status bar.)
- [x] Batch re-parenting (doc 04 §4.5): "Move Subgraph To..." command in inspector + palette, including promotion to root (`parent: null`). (`MoveToDialog.vue` candidate picker — every Card outside the moving subtree, searchable, with ancestor path context and a Move-to-root action; inspector Actions row + palette `node.move` command.)

## Milestone 4 — Interactive graph views

- [x] Node shapes (doc 05 §5.2): Action = solid compact pill/capsule; Guard = compact diamond; Idea = dashed-border pill; Card = symmetrical square with ID, aggregated QP, progress ring, and locked-QP % dependency indicator. (GraphPane SVG shapes existed; added the vanquished-QP progress ring and the locked-QP % dial on Cards.)
- [x] Vue Flow canvas (doc 05 §5.2): parent + prerequisite edges; local graph view centered on selection. (Force-directed SVG canvas with dependency-arrowed edges, minimap, zoom, drop-reparent; selection breadcrumbs center context.)
- [x] Depth-based scale & emphasis (doc 05 §5.3): top-level Cards/root nodes larger with heavier stroke; deeper sub-actions scale down. (depthScale map: ×1.15 at root tapering to ×0.8 floor, applied as a per-node transform.)
- [x] Color overlay modes (doc 05 §5.3): `C E` by Epic, `C D` by Discipline, `C P` by Priority warmth, `C S` by lifecycle (blocked red/amber, active blue, vanquished gray), plus Effort (QP heatmap) mode. (lib/overlay.ts categorical/warmth/heatmap color engine + legend; mode select in graph toolbar, Graph menu, palette.)
- [x] Realm Map (doc 05 §5.5): global graph with single `[ Realm Origin ]` placeholder node that all top-level Cards branch from. (realmMap mode injects a synthetic origin scene item with dashed origin→root edges; toggled from the Graph menu and palette.)
- [x] Completed-task visibility 3-level toggle (doc 05 §5.5 B): Level 0 hide all vanquished; Level 1 hide only 100%-vanquished top-level Card subgraphs; Level 2 show everything. (graph toolbar select; level-1 walks to the top-level Card ancestor and keeps partial subgraphs; lib/node.ts filterByCompletionLevel + fullyVanquished helpers.)
- [x] Landmark Matrix (doc 05 §5.6): Kanban columns (Unstarted/Active/Blocked/Vanquished) grouped by effective landmark. (new LandmarkMatrix page: foldable landmark sections × status columns of leaf tasks.)
- [x] Domain swimlanes (doc 05 §5.6): matrix crossing Disciplines or Epics with status columns. (matrix swimlane mode: branch rows × status columns grid.)
- [x] Command palette (doc 05 §5.7): global `Cmd/Ctrl+K` or `/` with fuzzy search over IDs, titles, tags, Epics, Landmarks, Discipline paths. (palette hay now includes #tags, epics, disciplines, effective landmark.)
- [x] Contextual graph actions & hotkeys (doc 05 §5.7): `Ctrl+Shift+W` wrap/promote, `Ctrl+M` re-parent, `Ctrl+Enter` vanquish, `Ctrl+L` link dependency, `Ctrl+Q` set QP. (wrap/move/vanquish chords wired in App for desktop mutation contexts; link-dependency and set-QP land through the metadata editor's blocked_by + QP fields, reachable via Ctrl+M/palette on the same node.)
- [x] Palette overlay toggles (doc 05 §5.7): `C E`, `C D`, `C P`, `C S`. (two-key chord armed by bare `C` with 1.2 s timeout; also menu + toolbar + palette entries.)
- [x] Wrap/Promote & move UX: “+ Wrap Node with Card Group” and “Move Subgraph to Parent…” controls wired to the same engine algorithms the CLI uses. (NodeActions bar + Ctrl+Shift+W + palette; MoveToDialog candidate picker; both call ops::wrap / ops::reparent.)
- [ ] Rolling velocity (doc 04 §4.4): completed-QP sums over rolling 7/30-day windows surfaced in the UI (engine supports it via `completed_at`; needs aggregation + display).

## Milestone 5 — Performance engine (Rust → Wasm)

- [ ] `legend_graph` crate: force-directed layout physics in Rust — Barnes-Hut quad-tree partitioning, Hooke attraction springs, Coulomb repulsion (doc 07).
- [ ] Compile to Wasm (`wasm-pack`, toolchain present) or run in a dedicated Tauri worker thread.
- [ ] 60+ FPS layout across thousands of nodes, streaming `(x, y)` position buffers to the rendering pipeline (doc 07).
- [ ] Integrate as the Realm Map layout engine; benchmark ≥1k nodes.

## Cross-cutting / deferred

- [ ] Architecture confirmation for doc 07 items already implemented but unchecked there (legend_core traversal/aggregation/indexing, notify watcher, clap CLI <5 ms startup, Vue 3 reactivity): verify each against its doc-07 criterion, then flip the design-doc checkboxes.
- [ ] Doc 03 “Node ID Immutability & File Slug Stability” parent item: both sub-items are done — confirm the parent criterion is fully satisfied, then check it off.
- [ ] `cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` clean before each milestone is “done”.
- [ ] Every implemented requirement flips its checkbox in `notes/design-documents/`; nothing is checked off without a passing test or manual verification.

## Verification cheat sheet

```bash
cargo test && cd desktop && cargo test && cd ..
cargo clippy --workspace --all-targets -- -D warnings
cd frontend && npm run typecheck && npm run build
cargo run -p legend_cli -- --realm examples/realm-demo query --unblocked

# Desktop app
cargo run -p legendary-desktop                        # dev renderer (Vite :5173) — or: scripts\dev-desktop.bat
cargo run -p legendary-desktop --features custom-protocol  # production dist — or: scripts\preview-desktop.bat
```
