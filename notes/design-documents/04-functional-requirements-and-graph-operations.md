## 4. Functional Requirements & Graph Operations

### 4.1. The "Wrap / Promote to Card" Insertion Operation

When an **Action**, **Guard**, or **Idea** node grows beyond its initial scope, the user can execute a quick **"Promote to Card"** or **"Insert Card Above"** action in the graph, inspector, command palette, or CLI.

- [ ] **Operation Algorithm:**

    - [ ] Generate a randomized 4-character Base36 ID for a Card Node $C_{\text{new}}$ (e.g., `CARD-W4R9`).

    - [ ] Inherit components from target node $N_{\text{target}}$ (copying `epics`, `landmark`, `priority`, and `disciplines`).

    - [ ] Set $C_{\text{new}}.\text{parent} = N_{\text{target}}.\text{parent}$.

    - [ ] Reparent target node: Set $N_{\text{target}}.\text{parent} = C_{\text{new}}.\text{id}$.

    - [ ] Save the new card as a flat file in `Nodes/CARD-W4R9_[slug].md`.

    - [ ] Shift UI focus or return updated CLI JSON for $C_{\text{new}}$, allowing immediate addition of sibling nodes beneath it.

### 4.2. Local File System & Persistence Contract

- [ ] **File Operations:** The system MUST parse, create, update, and delete node files directly inside `Nodes/` without modifying unmanaged Markdown text.

- [ ] **External Edits:** The system MUST watch `Nodes/` for external file changes (e.g., edits made in VS Code, Obsidian, or an AI agent) and update UI state and indexes in real time.

- [ ] **Deterministic Frontmatter:** YAML key ordering MUST be deterministic across reads and writes to maintain clean Git diffs.

### 4.3. Dependency & Node Resolution (DAG Rules, Card Completion & Cycle Prevention)

- [ ] **Computed Runtime Blocked State:**

    - [ ] To eliminate unnecessary disk writes and Git diff churn across downstream files whenever an upstream task is finished, `blocked` is a **computed runtime state** calculated dynamically during DAG evaluation.

    - [ ] An **Action**, **Guard**, or **Idea** node $N$ is dynamically evaluated as **`Blocked`** if any node listed in $N.\text{blocked\_by}$ has $\text{status} \neq \text{vanquished}$.

    - [ ] Frontmatter `status` represents the primary intended lifecycle state (`unstarted`, `active`, `vanquished`).

- [ ] **Card / Parent Blocked Logic:**

    - [ ] A parent Card or group node is dynamically evaluated as **`Blocked`** if **any of its direct active child nodes are currently blocked**.

- [ ] **Card Vanquishing & Forced Cascade Completion Rule:**

    - [ ] A Card node **cannot** be marked `status: vanquished` directly while any of its leaf Action, Guard, or Idea nodes remain unvanquished ($\text{status} \in \{\text{unstarted}, \text{active}\}$).

    - [ ] **UI Action:** Attempting to vanquish an incomplete Card prompts the user: _"Card contains N active tasks. Force complete all child tasks?"_

    - [ ] **CLI Action:** Executing `legend update [CARD_ID] --status vanquished` without completion will fail with exit status 1 and error: `Error: Cannot vanquish Card [ID]. N child tasks remaining. Use --cascade to force complete.`

    - [ ] **Forced Cascade Option (`--cascade` / UI Force Switch):** When forced completion is requested, the system recursively sets `status: vanquished` and populates `completed_at` timestamps on **all descendant leaf Action, Guard, and Idea nodes** in the Card's tree, then marks the Card as `vanquished`.

- [ ] **Percentage of Quest Points Locked:**

    - [ ] In addition to state flags, Cards calculate and display the exact percentage of their descendant Quest Points currently locked behind blocked prerequisites:

        $$\text{Locked } QP \% = \left( \frac{\sum_{n \in \text{BlockedLeafDescendants}(\text{Card})} QP(n)}{QP_{\text{Card}}} \right) \times 100$$
- [ ] **Strict Cycle Prevention & Validation:**

    - [ ] **Prohibition:** Circular dependency loops (e.g., $A \rightarrow B \rightarrow C \rightarrow A$) or parent-child container loops ($Card A \rightarrow Card B \rightarrow Card A$) are strictly prohibited during node creation, link creation, reparenting, or CLI mutations.

    - [ ] **Validation on Mutation:** Before writing dependency links (`blocked_by`) or parent references (`parent`), the engine performs depth-first traversal (DFS) in Rust to verify the change will not close a cycle. If a cycle is detected, the operation is aborted with error `CycleDetectedError`.

    - [ ] **Defensive Indexing for Manual File Edits:** If a user or external editor manually introduces a circular link directly in Markdown files, the indexer detects the loop during `.legend/index.json` generation, flags affected nodes with `validation_error: "Cycle detected"`, displays a diagnostic error in the UI / CLI (`legend index sync`), and safely isolates the circular nodes without causing infinite recursion or stack overflows.

### 4.4. Estimation & Rolling Velocity (Heroic Momentum)

- [ ] **Point Assignment:** Fibonacci points ($QP \in \{1, 2, 3, 5, 8, 13, 21\}$) are assigned to **Action**, **Guard**, and **Idea** nodes.

- [ ] **Multi-Level Card Point Aggregation:** Cards or group containers can nest inside other Cards. Aggregated Quest Points ($QP$) for any container node are calculated using **recursive post-order traversal over all leaf Action, Guard, and Idea nodes** in its descendant tree:

    $$QP_{\text{Card}} = \sum_{n \in \text{LeafDescendants}(\text{Card})} QP(n)$$

    This ensures intermediate container nodes do not double-count points when calculating total effort, progress rings, or percentage of locked Quest Points.

- [ ] **Rolling Velocity:** Calculated by summing completed $QP$ from vanquished leaf nodes over a rolling window ($W \in \{7\text{ days}, 30\text{ days}\}$):

    $$V_{W} = \sum_{n \in \text{VanquishedInWindow}(W)} QP(n)$$

### 4.5. Subgraph Deletion, Reference Integrity & Movement Rules

- [ ] **Recursive Subgraph Deletion Warning:** Attempting to delete a Card or parent container node $C$ triggers a warning modal displaying the total count of affected descendant nodes.

    - [ ] Confirming deletion **recursively deletes** $C$ and all descendant Cards, Actions, Guards, and Ideas from `Nodes/`.

    - [ ] Deleting any entity automatically strips its `id` from all downstream `blocked_by` prerequisite arrays across the remaining graph, preventing deadlocks or dangling prerequisite links.

    - [ ] Synchronously updates `.legend/index.json` and `.legend/INDEX.md`.

- [ ] **Intuitive Subgraph Movement & Re-parenting:**

    - [ ] **Drag-and-Drop Re-parenting:** Dragging a child node or entire sub-Card branch onto another Card in the graph view or tree navigator updates its `parent` component.

    - [ ] **Batch Re-parenting Action:** Selecting child nodes provides a "Move Subgraph To..." command in the inspector and command palette, allowing rapid selection of a new parent Card or promotion to root (`parent: null`).

### 4.6. Landmark Inheritance Hierarchy & Matrix Resolution

- [ ] **Hierarchical Inheritance Rule:** Any descendant node (Action, Guard, Idea, or nested child Card) inherits its effective `landmark` assignment from its parent Card by default across any level of nesting.

- [ ] **Explicit Local Override:** Setting an explicit `landmark` key on a child Action, Guard, Idea, or nested Card overrides any inherited landmark value from ancestor Cards.

- [ ] **Effective Landmark Calculation:** Views (such as the Landmark Matrix Kanban) and CLI queries resolve a node's effective landmark according to recursive parent lookup:

    $$\text{Landmark}_{\text{effective}}(N) = \begin{cases} N.\text{landmark} & \text{if } N.\text{landmark} \neq \text{null} \\ \text{Landmark}_{\text{effective}}(N.\text{parent}) & \text{if } N.\text{parent} \neq \text{null} \\ \text{null} & \text{otherwise} \end{cases}$$
