## 7. Tech Stack & Implementation Architecture

To guarantee maximum execution speed, zero lag on large graphs, and instant index synchronizations, all performance-critical computational workloads are implemented in **Rust** and shared across both the Tauri desktop shell and the native CLI binary.

- [ ] **Core Engine Crate (`legend_core` - Rust):** A shared, high-performance Rust library responsible for:

    - [ ] **DAG Graph Traversal & Cycle Detection:** High-speed Depth-First Search (DFS) cycle checks and dynamic `blocked` state evaluations executed in single-digit milliseconds.

    - [ ] **Multi-Level Traversal & Aggregation:** Recursive post-order Quest Point ($QP$) aggregation, locked $QP\%$ metrics, and effective landmark inheritance resolution.

    - [ ] **Synchronous Indexing & Parsing:** Ultra-fast YAML frontmatter parsing, filesystem indexing, and synchronous generation of `.legend/index.json` and `.legend/INDEX.md`.

- [ ] **Force-Directed Graph Engine (Rust / Wasm Acceleration):**

    - [ ] Layout physics calculations (Barnes-Hut spatial quad-tree partitioning, Hooke attraction springs, and Coulomb repulsion forces) are calculated in **Rust** (compiled directly to WebAssembly or executed in a dedicated Tauri worker thread).

    - [ ] Smooth 60+ FPS layout calculations across thousands of visual nodes and edges, passing calculated $(x, y)$ position buffer arrays directly to the rendering pipeline.

- [ ] **Desktop Shell & Native Runtime:** **Tauri (Rust)**

    - [ ] **Lightweight Footprint:** Utilizes the native OS webview (WebView2 on Windows, WebKit on macOS/Linux), yielding tiny binaries (~10–15 MB) and low RAM usage (~30–50 MB).

    - [ ] **System Capabilities & File Watching:** Rust native file watcher (`notify` crate) with write debouncing to observe external filesystem edits without index-thrashing loops.

- [ ] **Headless Command Line Interface (`bin/legend`):**

    - [ ] Standalone CLI binary written in **Rust** using `clap` and `legend_core`, ensuring terminal and AI agent queries run with zero runtime startup overhead (instant startup, $< 5\text{ms}$ execution).

- [ ] **Frontend Framework:** **Vue 3 (Composition API + TypeScript)**

    - [ ] Fine-grained reactivity for UI inspectors, markdown context editing, and visual graph presentation (using Canvas/SVG and Vue Flow).
