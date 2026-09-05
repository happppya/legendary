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

- [ ] **Left Panel (Focused Node Workspace):** Renders the item title at the top, followed directly by the editable Markdown body context and goal notes. The side metadata panel provides dedicated controls to edit all attached ECS components.

- [ ] **Right Panel (Interactive Local Graph View):** Focuses on the selected node as the active center point within its immediate graph.

### 5.2. Graph Node Shapes & Visual Hierarchy Rules

- [ ] **Actions (Compact Horizontal Pills / Capsules):** Executable Action nodes render as **solid-bordered compact horizontal pill or capsule shapes** for optimal title legibility and clean link routing.

- [ ] **Guards (Small Diamonds):** Quality gate Guard nodes render as **compact diamonds (45-degree tilted squares)** with distinct high-contrast borders.

- [ ] **Ideas (Dashed-Border Pills / Soft Capsules):** Speculative Idea nodes render as **dashed-border pills or soft-accent capsules** with subtle pitch styling to distinguish creative pitches from executable task items.

- [ ] **Cards (Symmetrical Squares):** Container Card nodes render as **symmetrical squares**. Squares display the Card ID, aggregated Quest Points ($QP_{\text{Card}}$), a progress ring representing completed child actions, and a visual dependency indicator showing the percentage of $QP$ currently locked behind blocked prerequisites.

### 5.3. Visual Emphasis & Dynamic Highlight Engine

- [ ] **Depth-Based Scale & Emphasis (Default Hierarchy):** Top-level Cards and root-level nodes receive greater visual scale and stroke weight. Deeper sub-actions scale down proportionally.

- [ ] **Toggleable Color & Highlight Overlay Modes (`C E`, `C D`, `C P`, `C S`):**

    - [ ] **Color by Epic:** Colors nodes based on associated Epic branches.

    - [ ] **Color by Discipline:** Color-codes nodes according to Discipline branches.

    - [ ] **Color by Priority:** Scales visual weight and warmth (`critical` > `high` > `medium` > `low`).

    - [ ] **Color by Lifecycle / Status:** Highlights runtime-blocked tasks in warning red/amber, active tasks in blue, and vanquished tasks in muted grays.

    - [ ] **Effort Mode:** Heatmaps node contrast according to estimation scores ($QP$).

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

- [ ] **`Hide All Completed` (Level 0):** Hides every vanquished Action, Guard, Idea, and Card across the canvas.

- [ ] **`Hide Completed Subgraphs Only` (Level 1):** Hides top-level Cards ONLY if **100% of their descendant nodes are vanquished**. Partially completed Card subgraphs remain visible.

- [ ] **`Show All Completed` (Level 2):** Displays all vanquished tasks, completed Card trees, and active tasks across the entire graph.

### 5.6. The Landmark Matrix (Kanban & Domain View)

- [ ] **Kanban Mode:** Status columns (`Unstarted`, `Active`, `Blocked`, `Vanquished`) displaying Action, Guard, and Idea nodes grouped by their effective (explicit or inherited) Landmark.

- [ ] **Domain Swimlanes:** Matrix view crossing Disciplines or Epics with Status columns.

### 5.7. Command Palette & Keyboard-First Workflows

Modal Command Palette accessible globally via `Cmd+K` / `Ctrl+K` or `/`.

- [ ] **Fuzzy Search:** Real-time search matching Node IDs (`ACT-3X7P`, `IDEA-5T2P`), titles, tags, Epics, Landmarks, and Discipline string paths.

- [ ] **Contextual Graph Actions:** `Ctrl+Shift+W` (wrap/promote), `Ctrl+M` (re-parent), `Ctrl+Enter` (vanquish), `Ctrl+L` (link dependency), `Ctrl+Q` (set QP).

- [ ] **Overlay Toggles:** `C E` (Epic), `C D` (Discipline), `C P` (Priority), `C S` (Status).
