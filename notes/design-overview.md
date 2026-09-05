# Legendary – Local-First Game Dev Task & Lore Engine

## 1. Vision & Core Philosophy

This application is a local-first, Markdown-native, graph-based project management tool designed specifically for game developers. It merges tactile organization with the infinite depth and interconnected flexibility of node-based graph editors (like Obsidian).

### Key Principles

- **Local-First & Plain-Text:** All data resides on the user's filesystem as standard Markdown (`.md`) files with YAML frontmatter. Zero vendor lock-in, fully version-controllable via Git.
    
- **Pure ECS Architecture (Entities & Components):** Graph nodes are light entities composed of modular frontmatter components. Functional attributes like Priority, Status, Disciplines, Epics, Quest Points, and Dependencies are treated as distinct components attached to an entity.
    
- **Flat Filesystem Storage:** All node entities reside directly in a single `Nodes/` folder, removing filesystem nesting. Dynamic grouping, filtering, and visual hierarchies are managed in-app via ECS components rather than filesystem folder paths.
    
- **Heterogeneous Node Types:** Tasks are split into structural **Card** groups, executable **Action** nodes, quality-gate **Guard** nodes, and speculative **Idea** nodes within a Directed Acyclic Graph (DAG).
    
- **Flexible Parent Grouping:** While **Card** nodes are the primary recommended organizational containers, _any_ node entity can technically serve as a parent group node if child entities set its ID in their `parent` key.
    
- **Functional Groupings & Checkpoints:** Projects are organized into logical feature domains (**Epics**) and targeted completion goals (**Landmarks**), eliminating strict artificial sprint cycles in favor of continuous flow and clear milestone targets.
    
- **Keyboard-First & Fluid Interaction:** Complete feature parity between mouse-driven graph operations and keyboard shortcuts, anchored by an instant universal Command Palette.
    
- **AI Agent First:** The filesystem layout, file naming conventions, metadata indexes, and CLI interface are designed so AI tools (Cursor, Claude Desktop, CLI scripts) can inspect, query, and modify the graph using built-in app logic without reinventing frontmatter parsing or DAG traversal.
    

## 2. Taxonomy & Terminology

To keep development aligned with game creation workflows, the application uses modular node types and lightweight thematic terminology.

|Concept|UI Label|Backend Key / Component|Description|
|---|---|---|---|
|**Group Node**|**Card**|`kind: card`|Primary organizational container node. Rendered as a symmetrical square in the graph. Holds markdown documentation, system context, and child nodes. Recommended default for subgraphs.|
|**Execution Node**|**Action**|`kind: action`|Single, short, clearly defined task or execution goal (e.g., "Implement Dash Input Hook"). Rendered as a compact horizontal pill or capsule shape in graph views.|
|**Quality Gate Node**|**Guard**|`kind: guard`|Specialized validation node (Code Review, QA Pass, Build Verification). Behaves like an Action node logically, rendered as a small diamond (45-degree tilted square) with distinct high-contrast accent framing.|
|**Speculative Node**|**Idea**|`kind: idea`|Brainstorming, pitch, or speculative feature concept. Functions logically like an Action node, rendered in graph views as a distinct dashed-outline pill or soft-accent capsule.|
|**Feature Domain**|**Epic**|`epics`|Hierarchical feature domain tree (e.g., `Combat_Engine/Locomotion`, `UI/HUD`). Defined canonically in `.legend/taxonomy.yaml`. **Entities can belong to multiple epics.**|
|**Milestone**|**Landmark**|`landmark`|Delivery target or build checkpoint (e.g., "Steam Playtest Demo", "Publisher Vertical Slice"). Defined in `.legend/landmarks.yaml` or auto-extracted from `Landmarks/`. Inherited by child nodes unless explicitly overridden.|
|**Domain Tree**|**Discipline**|`disciplines`|Hierarchical domain tree (e.g., `Programming/Optimization`, `Programming/View`, `Art/UI`). Defined canonically in `.legend/taxonomy.yaml`. **Entities can have multiple disciplines.**|
|**Urgency**|**Priority**|`priority`|Execution importance (`critical`, `high`, `medium`, `low`). **Strictly single-value per entity.**|
|**Lifecycle**|**Status**|`status`|User-assigned lifecycle state (`unstarted`, `active`, `vanquished`). **Note: `blocked` is dynamically computed at runtime during DAG traversal.**|
|**Effort Points**|**Quest Points (QP)**|`quest_points`|Fibonacci estimation points assigned to Action, Guard, and Idea nodes.|
|**Prerequisites**|**Blocked by**|`blocked_by`|Upstream nodes that must be completed before a node unlocks.|
|**Completed Item**|**Vanquished**|`status: vanquished`|Finished node. Cards require 100% child completion or explicit force cascade.|

> **Note on Node Parenting Flexibility:** The system permits any node type (`card`, `action`, `guard`, `idea`) to act as a parent container node by referencing its ID in a child node's `parent` field. The **Card** entity type is an organizational recommendation for clarity, rather than a hard technical constraint enforced by the engine.

## 3. Data Contract & Node Schema (ECS Composition)

Every entity in the graph is represented as a single Markdown file inside the flat `Nodes/` directory. Frontmatter keys represent composable data components attached to the entity.

### 3.1. Entity Component Model

Entities are formed by composing standard components:

|Component Key|Multiplicity|Type|Description|
|---|---|---|---|
|`id`|Single|String|Unique Base36 hash identifier (`CARD-K9F2`, `ACT-3X7P`, `IDEA-5T2P`). Immutable primary key.|
|`kind`|Single|Enum|`card` \| `action` \| `guard` \| `idea`|
|`status`|Single|Enum|Frontmatter stores primary intended lifecycle state: `unstarted` \| `active` \| `vanquished`|
|`priority`|Single|Enum|`critical` \| `high` \| `medium` \| `low`|
|`disciplines`|**Multiple**|Array $\text{String}$|Hierarchical paths (e.g., `["Programming/View", "Art/UI"]`). Validated against `.legend/taxonomy.yaml`.|
|`epics`|**Multiple**|Array $\text{String}$|Hierarchical paths (e.g., `["Combat_Engine/Melee", "Core/Input"]`). Validated against `.legend/taxonomy.yaml`.|
|`quest_points`|Single|Integer|Fibonacci score (Action/Guard/Idea nodes).|
|`landmark`|Single|String / Null|Major build checkpoint (`Landmark_01_Demo`). Validated against `.legend/landmarks.yaml` or `Landmarks/`. Inherited from parent if `null`.|
|`parent`|Single|String / Null|ID of parent node (typically a Card node).|
|`blocked_by`|**Multiple**|Array $\text{String}$|IDs of required upstream nodes.|

### 3.2. Central Taxonomy & Landmark Registries (`.legend/taxonomy.yaml` & `.legend/landmarks.yaml`)

To prevent string path drift, casing mismatches (`programming/locomotion` vs `Programming/Locomotion`), and widespread frontmatter Git churn when renaming domains or build checkpoints:

#### A. Central Taxonomy Registry (`.legend/taxonomy.yaml`)

All Epics and Disciplines are declared in a central canonical taxonomy file.

```
# .legend/taxonomy.yaml
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
```

#### B. Central Landmark Registry & Auto-Extraction (`.legend/landmarks.yaml` & `Landmarks/`)

Landmarks are declared either explicitly in `.legend/landmarks.yaml` or dynamically auto-extracted from `.md` files present in the `Landmarks/` folder.

```
# .legend/landmarks.yaml
landmarks:
  Landmark_01_Demo:
    title: "Steam Playtest Demo"
    target_date: "2026-11-15"
  Landmark_02_Slice:
    title: "Publisher Vertical Slice"
    target_date: "2027-02-28"

# Alias mapping prevents broken references when landmark names change
aliases:
  "Demo_v1": "Landmark_01_Demo"
  "Vertical_Slice": "Landmark_02_Slice"
```

- **Validation & Auto-Completion:** Frontmatter fields for `disciplines`, `epics`, and `landmark` are validated against these registries. UI and CLI inputs auto-complete using these trees.
    
- **Alias Resolution:** If an alias is mapped, queries or frontmatter using legacy strings resolve transparently to the canonical target without triggering mass search-and-replace edits across Markdown files.
    

### 3.3. Node ID System, Base36 Hash Strategy & Slug Stability

- **Randomized Base36 Allocation:** Node IDs are generated as randomized 4-character Base36 hash strings (`0-9`, `A-Z`).
    
- **Format:** `[KIND_PREFIX]-[RANDOM_BASE36_4]` (e.g., `CARD-K9F2`, `ACT-3X7P`, `GRD-7M2Q`, `IDEA-5T2P`).
    
- **Collision Resistance:** A 4-character Base36 string yields $36^4 = 1,679,616$ unique combinations per node prefix. If a collision occurs during creation, the system re-rolls or scales the ID length (`ACT-3X7PA`).
    
- **Node ID Immutability & File Slug Stability:**
    
    - The Node ID (`ACT-3X7P`) is the **immutable primary key** across all links, dependency arrays, and filesystem indexes.
        
    - Filename title slugs (e.g., `Nodes/ACT-3X7P_dash_stamina_cost.md`) are generated upon creation and **remain fixed**. Editing a title updates the `title` frontmatter key and markdown content without renaming the file on disk. File renames occur **ONLY upon explicit user/agent request** (`legend rename [ID]`).
        

### 3.4. File System Layout (Flat Storage)

All nodes are stored as direct children of `Nodes/`.

```
project-realm/
├── AGENTS.md                    # Direct instructions & rules for AI Agents
├── bin/
│   └── legend                   # Native Rust CLI binary for agent & terminal queries
├── .legend/                     # Local app metadata, cache, and agent indexes
│   ├── taxonomy.yaml            # Canonical Epics & Disciplines taxonomy definition
│   ├── landmarks.yaml           # Canonical Landmarks registry & alias definitions
│   ├── index.json               # Auto-generated full DAG & metadata index
│   └── INDEX.md                 # Human/agent-readable summary of active graph
├── Landmarks/                   # Markdown files defining major project milestones
│   └── Landmark_01_Demo.md
└── Nodes/                       # Flat Entity Database (All nodes live here directly)
    ├── CARD-K9F2_player_controller.md
    ├── ACT-3X7P_dash_stamina_cost.md
    ├── GRD-7M2Q_dash_code_review.md
    ├── IDEA-5T2P_grappling_hook_mechanic.md
    └── ACT-8J3W_stamina_bar_ui.md
```

### 3.5. Entity Component Schemas

#### A. Card Group Node (`Nodes/CARD-K9F2_player_controller.md`)

```
---
id: CARD-K9F2
kind: card                      # Entity type: card | action | guard | idea
title: Player Movement & Controller Core
status: active                  # User-set lifecycle: unstarted | active | vanquished
priority: high                  # Single-value component: critical | high | medium | low
disciplines:                    # Multi-value component (Validated against taxonomy.yaml)
  - Programming/Locomotion
  - Programming/Input
epics:                          # Multi-value component (Validated against taxonomy.yaml)
  - Combat_Engine/Locomotion
  - Core_Systems/Player
landmark: Landmark_01_Demo      # Validated against landmarks.yaml / Landmarks folder
parent: null
blocked_by: []
created_at: 2026-09-04T18:00:00Z
completed_at: null
tags:
  - player
  - locomotion
---

## System Context & Architectural Design
This card encompasses all locomotion mechanics, input buffer handling, and stamina consumption rules for the main player avatar.
```

#### B. Action Node (`Nodes/ACT-3X7P_dash_stamina_cost.md`)

```
---
id: ACT-3X7P
kind: action                    # Entity type: card | action | guard | idea
title: Deduct Stamina on Dash Trigger
status: active                  # Note: Dynamically evaluated as blocked if prerequisites unvanquished
priority: high
disciplines:
  - Programming/Locomotion
  - Programming/Systems
epics:
  - Combat_Engine/Locomotion
quest_points: 3                 # Fibonacci: 1, 2, 3, 5, 8, 13, 21
landmark: null                  # Inherits Landmark_01_Demo from CARD-K9F2
parent: CARD-K9F2
blocked_by:
  - ACT-8J3W
created_at: 2026-09-04T18:10:00Z
completed_at: null
tags:
  - mechanics
---

## Action Goal
Deduct 25 stamina points from `PlayerStaminaPool` immediately when the dash input is accepted.
```

#### C. Guard Node (`Nodes/GRD-7M2Q_dash_code_review.md`)

```
---
id: GRD-7M2Q
kind: guard                     # Entity type: card | action | guard | idea
title: Code Review: Dash Mechanics & Memory Leaks
status: unstarted
priority: medium
disciplines:
  - Programming/Optimization
  - QualityAssurance
epics:
  - Combat_Engine/Locomotion
quest_points: 1
landmark: null                  # Inherits Landmark_01_Demo from CARD-K9F2
parent: CARD-K9F2
blocked_by:
  - ACT-3X7P
created_at: 2026-09-04T18:15:00Z
completed_at: null
tags:
  - qa
  - code_review
---

## Quality Check Criteria
- Verify state machine transition cleanup.
- Ensure no memory allocations in update loop.
```

#### D. Idea Node (`Nodes/IDEA-5T2P_grappling_hook_mechanic.md`)

```
---
id: IDEA-5T2P
kind: idea                      # Entity type: card | action | guard | idea
title: Grappling Hook Physics & Momentum Retention
status: unstarted
priority: low
disciplines:
  - Programming/Locomotion
  - Art/Concept
epics:
  - Combat_Engine/Locomotion
quest_points: 5
landmark: null                  # Inherits Landmark_01_Demo from CARD-K9F2
parent: CARD-K9F2
blocked_by: []
created_at: 2026-09-05T09:00:00Z
completed_at: null
tags:
  - concept
  - pitch
---

## Idea Concept & Mechanics Exploration
Investigate physics-based cable simulation for momentum-preserving grappling locomotion in air combat.
```

## 4. Functional Requirements & Graph Operations

### 4.1. The "Wrap / Promote to Card" Insertion Operation

When an **Action**, **Guard**, or **Idea** node grows beyond its initial scope, the user can execute a quick **"Promote to Card"** or **"Insert Card Above"** action in the graph, inspector, command palette, or CLI.

- **Operation Algorithm:**
    
    1. Generate a randomized 4-character Base36 ID for a Card Node $C_{\text{new}}$ (e.g., `CARD-W4R9`).
        
    2. Inherit components from target node $N_{\text{target}}$ (copying `epics`, `landmark`, `priority`, and `disciplines`).
        
    3. Set $C_{\text{new}}.\text{parent} = N_{\text{target}}.\text{parent}$.
        
    4. Reparent target node: Set $N_{\text{target}}.\text{parent} = C_{\text{new}}.\text{id}$.
        
    5. Save the new card as a flat file in `Nodes/CARD-W4R9_[slug].md`.
        
    6. Shift UI focus or return updated CLI JSON for $C_{\text{new}}$, allowing immediate addition of sibling nodes beneath it.
        

### 4.2. Local File System & Persistence Contract

- **File Operations:** The system MUST parse, create, update, and delete node files directly inside `Nodes/` without modifying unmanaged Markdown text.
    
- **External Edits:** The system MUST watch `Nodes/` for external file changes (e.g., edits made in VS Code, Obsidian, or an AI agent) and update UI state and indexes in real time.
    
- **Deterministic Frontmatter:** YAML key ordering MUST be deterministic across reads and writes to maintain clean Git diffs.
    

### 4.3. Dependency & Node Resolution (DAG Rules, Card Completion & Cycle Prevention)

- **Computed Runtime Blocked State:**
    
    - To eliminate unnecessary disk writes and Git diff churn across downstream files whenever an upstream task is finished, `blocked` is a **computed runtime state** calculated dynamically during DAG evaluation.
        
    - An **Action**, **Guard**, or **Idea** node $N$ is dynamically evaluated as **`Blocked`** if any node listed in $N.\text{blocked\_by}$ has $\text{status} \neq \text{vanquished}$.
        
    - Frontmatter `status` represents the primary intended lifecycle state (`unstarted`, `active`, `vanquished`).
        
- **Card / Parent Blocked Logic:**
    
    - A parent Card or group node is dynamically evaluated as **`Blocked`** if **any of its direct active child nodes are currently blocked**.
        
- **Card Vanquishing & Forced Cascade Completion Rule:**
    
    - A Card node **cannot** be marked `status: vanquished` directly while any of its leaf Action, Guard, or Idea nodes remain unvanquished ($\text{status} \in \{\text{unstarted}, \text{active}\}$).
        
    - **UI Action:** Attempting to vanquish an incomplete Card prompts the user: _"Card contains N active tasks. Force complete all child tasks?"_
        
    - **CLI Action:** Executing `legend update [CARD_ID] --status vanquished` without completion will fail with exit status 1 and error: `Error: Cannot vanquish Card [ID]. N child tasks remaining. Use --cascade to force complete.`
        
    - **Forced Cascade Option (`--cascade` / UI Force Switch):** When forced completion is requested, the system recursively sets `status: vanquished` and populates `completed_at` timestamps on **all descendant leaf Action, Guard, and Idea nodes** in the Card's tree, then marks the Card as `vanquished`.
        
- **Percentage of Quest Points Locked:**
    
    - In addition to state flags, Cards calculate and display the exact percentage of their descendant Quest Points currently locked behind blocked prerequisites:
        
        $$\text{Locked } QP \% = \left( \frac{\sum_{n \in \text{BlockedLeafDescendants}(\text{Card})} QP(n)}{QP_{\text{Card}}} \right) \times 100$$
- **Strict Cycle Prevention & Validation:**
    
    - **Prohibition:** Circular dependency loops (e.g., $A \rightarrow B \rightarrow C \rightarrow A$) or parent-child container loops ($Card A \rightarrow Card B \rightarrow Card A$) are strictly prohibited during node creation, link creation, reparenting, or CLI mutations.
        
    - **Validation on Mutation:** Before writing dependency links (`blocked_by`) or parent references (`parent`), the engine performs depth-first traversal (DFS) in Rust to verify the change will not close a cycle. If a cycle is detected, the operation is aborted with error `CycleDetectedError`.
        
    - **Defensive Indexing for Manual File Edits:** If a user or external editor manually introduces a circular link directly in Markdown files, the indexer detects the loop during `.legend/index.json` generation, flags affected nodes with `validation_error: "Cycle detected"`, displays a diagnostic error in the UI / CLI (`legend index sync`), and safely isolates the circular nodes without causing infinite recursion or stack overflows.
        

### 4.4. Estimation & Rolling Velocity (Heroic Momentum)

- **Point Assignment:** Fibonacci points ($QP \in \{1, 2, 3, 5, 8, 13, 21\}$) are assigned to **Action**, **Guard**, and **Idea** nodes.
    
- **Multi-Level Card Point Aggregation:** Cards or group containers can nest inside other Cards. Aggregated Quest Points ($QP$) for any container node are calculated using **recursive post-order traversal over all leaf Action, Guard, and Idea nodes** in its descendant tree:
    
    $$QP_{\text{Card}} = \sum_{n \in \text{LeafDescendants}(\text{Card})} QP(n)$$
    
    This ensures intermediate container nodes do not double-count points when calculating total effort, progress rings, or percentage of locked Quest Points.
    
- **Rolling Velocity:** Calculated by summing completed $QP$ from vanquished leaf nodes over a rolling window ($W \in \{7\text{ days}, 30\text{ days}\}$):
    
    $$V_{W} = \sum_{n \in \text{VanquishedInWindow}(W)} QP(n)$$

### 4.5. Subgraph Deletion, Reference Integrity & Movement Rules

- **Recursive Subgraph Deletion Warning:** Attempting to delete a Card or parent container node $C$ triggers a warning modal displaying the total count of affected descendant nodes.
    
    - Confirming deletion **recursively deletes** $C$ and all descendant Cards, Actions, Guards, and Ideas from `Nodes/`.
        
    - Deleting any entity automatically strips its `id` from all downstream `blocked_by` prerequisite arrays across the remaining graph, preventing deadlocks or dangling prerequisite links.
        
    - Synchronously updates `.legend/index.json` and `.legend/INDEX.md`.
        
- **Intuitive Subgraph Movement & Re-parenting:**
    
    - **Drag-and-Drop Re-parenting:** Dragging a child node or entire sub-Card branch onto another Card in the graph view or tree navigator updates its `parent` component.
        
    - **Batch Re-parenting Action:** Selecting child nodes provides a "Move Subgraph To..." command in the inspector and command palette, allowing rapid selection of a new parent Card or promotion to root (`parent: null`).
        

### 4.6. Landmark Inheritance Hierarchy & Matrix Resolution

- **Hierarchical Inheritance Rule:** Any descendant node (Action, Guard, Idea, or nested child Card) inherits its effective `landmark` assignment from its parent Card by default across any level of nesting.
    
- **Explicit Local Override:** Setting an explicit `landmark` key on a child Action, Guard, Idea, or nested Card overrides any inherited landmark value from ancestor Cards.
    
- **Effective Landmark Calculation:** Views (such as the Landmark Matrix Kanban) and CLI queries resolve a node's effective landmark according to recursive parent lookup:
    
    $$\text{Landmark}_{\text{effective}}(N) = \begin{cases} N.\text{landmark} & \text{if } N.\text{landmark} \neq \text{null} \\ \text{Landmark}_{\text{effective}}(N.\text{parent}) & \text{if } N.\text{parent} \neq \text{null} \\ \text{null} & \text{otherwise} \end{cases}$$

## 5. User Interface & View Systems

### 5.1. Focused Node Workspace & Interactive Graph

Selecting any node opens a split workspace designed for focused context and local graph navigation.

```
+------------------------------------+------------------------------------+
|  SELECTED NODE WORKSPACE           |  LOCAL NODE GRAPH VIEW             |
|                                    |                                    |
|  [ACT-3X7P] Deduct Stamina on...   |             ┌──────────┐           |
|  ================================= |             │CARD-K9F2 │           |
|  Markdown Title & Content:         |             │(Parent)  │           |
|  --------------------------------- |             └────┬─────┘           |
|  ## Action Goal                    |                  │                 |
|  Deduct 25 stamina points from     |                  ▼                 |
|  `PlayerStaminaPool` immediately   |            [ ACT-8J3W ]            |
|  when dash trigger is executed.    |            (Prereq Pill)           |
|                                    |                  │                 |
|  --------------------------------- |                  ▼                 |
|  METADATA & EDIT PANEL (SIDE)      |           [ ACT-3X7P ]             |
|  Kind:       [ Action v ]          |          (Focused Pill)            |
|  QP:         [ 3 v ]               |             /        \             |
|  Status:     [ Active v ]          |            ▼          ▼            |
|  Priority:   [ High v ]            |     ◇ <GRD-7M2Q>  ( IDEA-5T2P )   |
|  Disciplines:[ Prog/Locomotion v ] |    {Guard Diamond} {Dashed Idea}  |
|  Epics:      [ Combat_Engine v ]   |                                    |
|  Landmark:   [ Landmark 01 Demo v ]|  COLOR OVERLAY MODE:               |
|  Blocked By: [ ACT-8J3W v ]        |  (o) Epic  ( ) Discipline          |
|  Parent Card:[ CARD-K9F2 v ]       |  ( ) Priority  ( ) Lifecycle       |
|  [ + Wrap Node with Card Group ]   |                                    |
|  [ Move Subgraph to Parent... ]    |                                    |
+------------------------------------+------------------------------------+
```

- **Left Panel (Focused Node Workspace):** Renders the item title at the top, followed directly by the editable Markdown body context and goal notes. The side metadata panel provides dedicated controls to edit all attached ECS components.
    
- **Right Panel (Interactive Local Graph View):** Focuses on the selected node as the active center point within its immediate graph.
    

### 5.2. Graph Node Shapes & Visual Hierarchy Rules

- **Actions (Compact Horizontal Pills / Capsules):** Executable Action nodes render as **solid-bordered compact horizontal pill or capsule shapes** for optimal title legibility and clean link routing.
    
- **Guards (Small Diamonds):** Quality gate Guard nodes render as **compact diamonds (45-degree tilted squares)** with distinct high-contrast borders.
    
- **Ideas (Dashed-Border Pills / Soft Capsules):** Speculative Idea nodes render as **dashed-border pills or soft-accent capsules** with subtle pitch styling to distinguish creative pitches from executable task items.
    
- **Cards (Symmetrical Squares):** Container Card nodes render as **symmetrical squares**. Squares display the Card ID, aggregated Quest Points ($QP_{\text{Card}}$), a progress ring representing completed child actions, and a visual dependency indicator showing the percentage of $QP$ currently locked behind blocked prerequisites.
    

### 5.3. Visual Emphasis & Dynamic Highlight Engine

1. **Depth-Based Scale & Emphasis (Default Hierarchy):** Top-level Cards and root-level nodes receive greater visual scale and stroke weight. Deeper sub-actions scale down proportionally.
    
2. **Toggleable Color & Highlight Overlay Modes (`C E`, `C D`, `C P`, `C S`):**
    
    - **Color by Epic:** Colors nodes based on associated Epic branches.
        
    - **Color by Discipline:** Color-codes nodes according to Discipline branches.
        
    - **Color by Priority:** Scales visual weight and warmth (`critical` > `high` > `medium` > `low`).
        
    - **Color by Lifecycle / Status:** Highlights runtime-blocked tasks in warning red/amber, active tasks in blue, and vanquished tasks in muted grays.
        
    - **Effort Mode:** Heatmaps node contrast according to estimation scores ($QP$).
        

### 5.4. Discipline & Epic Tree Navigators

Visualizes disciplines and epics as collapsible hierarchies validated against `.legend/taxonomy.yaml`. Selecting any branch filters the graph to include entities attached to that branch or sub-branches.

### 5.5. The Realm Map (Global Interactive Graph)

```
                   +-----------------------+
                   |  [ Realm Origin ]     |
                   |  Placeholder Node     |
                   +-----------+-----------+
                               |
            +------------------+------------------+
            |                                     |
            v                                     v
     +--------------+                      +--------------+
     | CARD-K9F2    |                      | CARD-4M1P    |
     | Combat Core  |                      | UI & HUD     |
     +------+-------+                      +------+-------+
            |                                     |
            +----------+                          v
            |          |                    [ ACT-2N8V ]
            v          v                    (Pill Node)
       [ ACT-3X7P ]  ◇ <GRD-7M2Q>
       (Pill Node)   (Diamond Node)
```

#### A. Single Origin Node & Canvas Topology

All top-level Card nodes branch directly from a single, neutral **`[ Realm Origin ]`** placeholder node. Users switch **Color Overlays** to analyze relationships visually without repositioning nodes.

#### B. Granular Completed Task Visibility Engine (3-Level Toggle)

1. **`Hide All Completed` (Level 0):** Hides every vanquished Action, Guard, Idea, and Card across the canvas.
    
2. **`Hide Completed Subgraphs Only` (Level 1):** Hides top-level Cards ONLY if **100% of their descendant nodes are vanquished**. Partially completed Card subgraphs remain visible.
    
3. **`Show All Completed` (Level 2):** Displays all vanquished tasks, completed Card trees, and active tasks across the entire graph.
    

### 5.6. The Landmark Matrix (Kanban & Domain View)

- **Kanban Mode:** Status columns (`Unstarted`, `Active`, `Blocked`, `Vanquished`) displaying Action, Guard, and Idea nodes grouped by their effective (explicit or inherited) Landmark.
    
- **Domain Swimlanes:** Matrix view crossing Disciplines or Epics with Status columns.
    

### 5.7. Command Palette & Keyboard-First Workflows

Modal Command Palette accessible globally via `Cmd+K` / `Ctrl+K` or `/`.

- **Fuzzy Search:** Real-time search matching Node IDs (`ACT-3X7P`, `IDEA-5T2P`), titles, tags, Epics, Landmarks, and Discipline string paths.
    
- **Contextual Graph Actions:** `Ctrl+Shift+W` (wrap/promote), `Ctrl+M` (re-parent), `Ctrl+Enter` (vanquish), `Ctrl+L` (link dependency), `Ctrl+Q` (set QP).
    
- **Overlay Toggles:** `C E` (Epic), `C D` (Discipline), `C P` (Priority), `C S` (Status).
    

## 6. Agent Interoperability & Filesystem Strategy

### 6.1. Manifest & Index Pattern

- **Central Taxonomy & Landmarks (`.legend/taxonomy.yaml` & `.legend/landmarks.yaml`):** Provides AI agents with declarative registries of valid Disciplines, Epics, and Landmarks.
    
- **Auto-Generated Index (`.legend/index.json`):** On every save, UI edit, or CLI write mutation, the system **synchronously updates** a JSON document containing all node metadata, effective landmarks, dynamic computed blocked states, and relative paths.
    
- **Agent Readme (`.legend/INDEX.md`):** Human- and agent-readable Markdown table listing active Actions, blocked Guards, speculative Ideas, and high-level Cards, updated synchronously alongside `index.json`.
    

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

## 7. Tech Stack & Implementation Architecture

To guarantee maximum execution speed, zero lag on large graphs, and instant index synchronizations, all performance-critical computational workloads are implemented in **Rust** and shared across both the Tauri desktop shell and the native CLI binary.

- **Core Engine Crate (`legend_core` - Rust):** A shared, high-performance Rust library responsible for:
    
    - **DAG Graph Traversal & Cycle Detection:** High-speed Depth-First Search (DFS) cycle checks and dynamic `blocked` state evaluations executed in single-digit milliseconds.
        
    - **Multi-Level Traversal & Aggregation:** Recursive post-order Quest Point ($QP$) aggregation, locked $QP\%$ metrics, and effective landmark inheritance resolution.
        
    - **Synchronous Indexing & Parsing:** Ultra-fast YAML frontmatter parsing, filesystem indexing, and synchronous generation of `.legend/index.json` and `.legend/INDEX.md`.
        
- **Force-Directed Graph Engine (Rust / Wasm Acceleration):**
    
    - Layout physics calculations (Barnes-Hut spatial quad-tree partitioning, Hooke attraction springs, and Coulomb repulsion forces) are calculated in **Rust** (compiled directly to WebAssembly or executed in a dedicated Tauri worker thread).
        
    - Smooth 60+ FPS layout calculations across thousands of visual nodes and edges, passing calculated $(x, y)$ position buffer arrays directly to the rendering pipeline.
        
- **Desktop Shell & Native Runtime:** **Tauri (Rust)**
    
    - **Lightweight Footprint:** Utilizes the native OS webview (WebView2 on Windows, WebKit on macOS/Linux), yielding tiny binaries (~10–15 MB) and low RAM usage (~30–50 MB).
        
    - **System Capabilities & File Watching:** Rust native file watcher (`notify` crate) with write debouncing to observe external filesystem edits without index-thrashing loops.
        
- **Headless Command Line Interface (`bin/legend`):**
    
    - Standalone CLI binary written in **Rust** using `clap` and `legend_core`, ensuring terminal and AI agent queries run with zero runtime startup overhead (instant startup, $< 5\text{ms}$ execution).
        
- **Frontend Framework:** **Vue 3 (Composition API + TypeScript)**
    
    - Fine-grained reactivity for UI inspectors, markdown context editing, and visual graph presentation (using Canvas/SVG and Vue Flow).