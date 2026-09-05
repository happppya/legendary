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
