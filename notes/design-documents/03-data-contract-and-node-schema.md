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

- [x] **Validation & Auto-Completion:** Frontmatter fields for `disciplines`, `epics`, and `landmark` are validated against these registries. UI and CLI inputs auto-complete using these trees.

- [x] **Alias Resolution:** If an alias is mapped, queries or frontmatter using legacy strings resolve transparently to the canonical target without triggering mass search-and-replace edits across Markdown files.

### 3.3. Node ID System, Base36 Hash Strategy & Slug Stability

- [x] **Randomized Base36 Allocation:** Node IDs are generated as randomized 4-character Base36 hash strings (`0-9`, `A-Z`).

- [x] **Format:** `[KIND_PREFIX]-[RANDOM_BASE36_4]` (e.g., `CARD-K9F2`, `ACT-3X7P`, `GRD-7M2Q`, `IDEA-5T2P`).

- [x] **Collision Resistance:** A 4-character Base36 string yields $36^4 = 1,679,616$ unique combinations per node prefix. If a collision occurs during creation, the system re-rolls or scales the ID length (`ACT-3X7PA`).

- [ ] **Node ID Immutability & File Slug Stability:**

    - [x] The Node ID (`ACT-3X7P`) is the **immutable primary key** across all links, dependency arrays, and filesystem indexes.

    - [x] Filename title slugs (e.g., `Nodes/ACT-3X7P_dash_stamina_cost.md`) are generated upon creation and **remain fixed**. Editing a title updates the `title` frontmatter key and markdown content without renaming the file on disk. File renames occur **ONLY upon explicit user/agent request** (`legend rename [ID]`).

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
