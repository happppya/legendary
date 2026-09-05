# Legendary – Agent Instructions (Application Repository)

You are working in the Legendary repository: a local-first, Markdown-native, graph-based game dev task engine (ECS nodes over flat Markdown files). This repo contains the **application source** plus the **product specification**. A separate *realm* (game project) layout is exercised in `examples/realm-demo/`.

## Read first

1. [`notes/design-documents/README.md`](notes/design-documents/README.md) — index of the design spec, with per-topic routing.
2. The specific `notes/design-documents/NN-*.md` file for the area you touch (see the README table).
3. [`notes/design-overview.md`](notes/design-overview.md) — canonical prose master. **Never edit it.** The split files mirror it as checkboxes; if they disagree, the master wins and the split files should be regenerated.

Each requirement in `notes/design-documents/*.md` is a `- [ ]` item. Mark it `- [x]` **only after** implementing and verifying it. Do not reword items.

## Repository layout

```
crates/legend_core/   Shared Rust engine (model, frontmatter, dag, registry, realm)
crates/legend_cli/    `legend` CLI binary (clap)
desktop/              Tauri v2 desktop shell (not a default member yet)
frontend/             Vue 3 + TS + Vite renderer
examples/realm-demo/  Sample game realm for manual CLI testing / fixtures
notes/                Specification (design-overview.md + design-documents/)
```

## Build, test, verify

```bash
cargo test                          # engine + CLI (default workspace members)
cargo run -p legend_cli -- --help
cargo run -p legend_cli -- --realm examples/realm-demo index sync
cargo run -p legend_cli -- --realm examples/realm-demo query --json
cargo run -p legend_cli -- --realm examples/realm-demo tree CARD-K9F2
```

- Rust: keep `cargo fmt` clean and prefer `cargo clippy`-clean code. Put unit tests next to the module that owns the logic (core behavior lives in `legend_core`).
- Frontend: `cd frontend && npm install && npm run dev` (Vite on :5173). `npm run build` emits `frontend/dist`, which the Tauri shell consumes.
- Desktop: `cd desktop && cargo build` — requires `frontend/dist` to exist for a full build; this is intentionally not part of default `cargo test`/`cargo build`.

## Realm & node rules (see spec §3, §4, §6)

- Node files live **flat** in a realm's `Nodes/` as `Nodes/[ID]_[slug].md`; never create subfolders.
- `id` (`CARD-K9F2`, `ACT-3X7P`, …) is immutable; do not rename files when titles change.
- `status` is `unstarted | active | vanquished`; `blocked` is computed at runtime — never write it to frontmatter.
- Never introduce dependency or parent loops; the engine rejects them (`legend reparent`, wrap, etc.) and flags manual cycles during `index sync`.
- After manually editing node Markdown, run `legend index sync` to rebuild `.legend/index.json` and `.legend/INDEX.md`.
- YAML key ordering is canonical (fixed component order) to keep Git diffs clean.
