# Legendary – Design Documents (Agent Working Copy)

This directory holds the full design specification for **Legendary** (local-first, Markdown-native, graph-based game dev task engine), split into small, per-topic documents so an agent only reads what it needs.

Each specification statement in these files is written as a checklist item (`- [ ]`). As you implement a requirement, mark its box done: replace `- [ ]` with `- [x]`. Do not reword, renumber, or delete any item — wording is authoritative and shared with the master reference below.

- **Master reference:** [`../design-overview.md`](../design-overview.md) — the original, unsplit prose version. **Never edit it here; it is kept unchanged as the canonical source.** If these files and the master ever disagree, the master wins and these files should be regenerated.
- **Conversion rule used to generate these files:** every bullet and numbered list item (at every nesting level) became `- [ ]`; headings, tables, code blocks, ASCII diagrams, blockquotes, and formulas are untouched. `design-overview.md` has identical content except it keeps plain `-`/numbered markers.
- **Rule of thumb for reading:** for anything touching *node data or files*, start with document 03 and cross-check 02 for vocabulary. For anything touching *behavior*, start with 04. For anything *visual*, start with 05. For *agent/CLI surfaces*, start with 06.

## Document Index

| File | Covers (original §) | Read it when you are… |
|---|---|---|
| [`01-vision-and-core-philosophy.md`](01-vision-and-core-philosophy.md) | §1 – Vision & core principles (ECS, flat storage, node types, keyboard-first, AI-first) | Making high-level product/architecture decisions, writing the README pitch, or choosing what NOT to build (scope guardrails). |
| [`02-taxonomy-and-terminology.md`](02-taxonomy-and-terminology.md) | §2 – Node types, Epic/Landmark/Discipline/Priority/Status/QP vocabulary | Implementing UI labels, frontmatter enums, pickers, or documentation wording. Definitive glossary. |
| [`03-data-contract-and-node-schema.md`](03-data-contract-and-node-schema.md) | §3 – ECS component model, taxonomy/landmark registries, Base36 ID system, flat `Nodes/` layout, example node files | Parsing/writing node Markdown, frontmatter validation, ID generation, or setting up the repository scaffold. |
| [`04-functional-requirements-and-graph-operations.md`](04-functional-requirements-and-graph-operations.md) | §4 – Wrap/Promote-to-Card, persistence contract, DAG blocked-state/cascade/cycle rules, QP aggregation & velocity, subgraph delete/reparent, landmark inheritance | Implementing graph engine behavior, CLI mutations, DAG traversal, or progression rules. |
| [`05-user-interface-and-view-systems.md`](05-user-interface-and-view-systems.md) | §5 – Split workspace, node shapes, overlay modes, tree navigators, Realm Map, Landmark Matrix, command palette & hotkeys | Building any UI view, canvas rendering, or keyboard shortcuts. |
| [`06-agent-interoperability-and-filesystem-strategy.md`](06-agent-interoperability-and-filesystem-strategy.md) | §6 – `.legend/` indexes, flat-directory rationale, file naming, `legend` CLI contract, AGENTS.md content | Building the CLI, index generation, or wiring up agent tooling. |
| [`07-tech-stack-and-implementation-architecture.md`](07-tech-stack-and-implementation-architecture.md) | §7 – Rust `legend_core`, force-directed engine (Wasm), Tauri shell, CLI, Vue 3 frontend | Scaffolding the project, choosing libraries, or splitting work across crates/packages. |

## Working Conventions

- **Check items off only when truly done** — an unchecked box is a promise of work; a checked box is a claim that the requirement is implemented and verified.
- If a single box needs sub-tracking, do not split or edit it — track the breakdown elsewhere (e.g., as node files in `Nodes/` once the tool exists) and leave the box `[ ]` until everything it covers is complete.
- If you spot a wording/spec conflict with the master doc, flag it rather than silently editing either copy.
