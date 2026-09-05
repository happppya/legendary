// The demo realm shown by default in the workspace mockup.
//
// This is a hand-authored "slice" of the game project the reference mockup
// depicts (Combat Engine / Locomotion focus). The real engine does not yet
// ship graph payloads, so this module stands in as the single source of
// truth for the mock: it produces the same `NodeView` shape the desktop IPC
// will send, and it recomputes the runtime state the engine owns today
// (effective landmark inheritance, blocked cascade, QP aggregation, locked
// QP %), so the UI logic is data-driven instead of full of hard-coded
// numbers. Swapping `openRealm`'s result in for `MOCK_NODES` later keeps
// every pane working.
//
// Design-truth notes (design docs 02/03/04):
//   - ids are immutable `KIND-XXXX`; every file lives in `Nodes/`.
//   - status is authored (unstarted | active | vanquished); `blocked` is
//     computed. A non-vanquished node is blocked while any `blockedBy`
//     dependency is unfinished; a Card is also blocked while any descendant
//     is blocked.
//   - a Card's quest_points is null; its total aggregates descendant QP.
//   - landmarks inherit down the parent chain unless overridden.
//   - QP values come from the Fibonacci set.

import type { NodeView } from '../types'

export const MOCK_REALM = {
  /** Shown in the explorer dropdown + graph breadcrumb root. */
  name: 'Mythic Realms RPG Adventure',
  path: 'realm/mythic-realms',
}

/* ------------------------------------------------------------------ */
/* Raw authored entries                                                */
/* ------------------------------------------------------------------ */

type Status = 'unstarted' | 'active' | 'vanquished'
type Priority = 'critical' | 'high' | 'medium' | 'low'

interface Entry {
  key: string
  kind: 'card' | 'action' | 'guard' | 'idea'
  title: string
  status: Status
  priority: Priority
  qp?: number
  parent?: string
  blockedBy?: string[]
  epics: string[]
  disciplines: string[]
  landmark?: string
  tags?: string[]
  createdAt: string
  completedAt?: string
  body: string
}

const E = {
  combat: 'Combat Engine',
  loco: 'Combat Engine/Locomotion',
  abil: 'Combat Engine/Abilities',
  player: 'Core Systems/Player',
  ui: 'UI/HUD',
  prog: 'Progression',
  inv: 'Inventory',
  world: 'World',
  tools: 'Tools',
} as const

const D = {
  prog: 'Programming',
  gameplay: 'Gameplay',
  design: 'Design',
  art: 'Art',
  qa: 'Quality Assurance',
} as const

const PLAYTEST = 'Steam Playtest Demo'
const PUBLISHER = 'Publisher Vertical Slice'

const RAW: Entry[] = [
  /* ---- Combat Engine root ------------------------------------------ */
  {
    key: 'ce', kind: 'card', title: 'Combat Engine', status: 'active', priority: 'high',
    epics: [E.combat], disciplines: [D.prog, D.gameplay], landmark: PLAYTEST,
    createdAt: '2025-05-05T09:00:00Z',
    body: `## System Context

The combat engine owns every moment-to-moment gameplay system in the
vertical slice: locomotion, abilities, stamina and camera feel.

Child cards track each feature domain.`,
  },
  /* ---- Locomotion System (focused subgraph root) -------------------- */
  {
    key: 'loc', kind: 'card', title: 'Locomotion System', status: 'active', priority: 'high',
    parent: 'ce', epics: [E.loco], disciplines: [D.prog, D.gameplay], landmark: PLAYTEST,
    createdAt: '2025-05-05T09:05:00Z',
    body: `## Locomotion System

Everything that moves the player character. Ships as the core of the
Steam Playtest Demo slice.

- The movement core card is the single place character physics live.
- Camera & feel owns juice, not control.`,
  },
  {
    key: 'cmc', kind: 'card', title: 'Character Movement Core', status: 'active',
    priority: 'critical', parent: 'loc', epics: [E.loco], disciplines: [D.prog, D.gameplay],
    landmark: PLAYTEST, tags: ['movement core'],
    createdAt: '2025-05-05T10:21:00Z',
    body: `Core character movement logic: acceleration, deceleration, air control, gravity, and collision resolution. Should be engine-agnostic and extensible for future abilities.`,
  },
  {
    key: 'dash', kind: 'action', title: 'Implement Dash Input Hook', status: 'active',
    priority: 'critical', parent: 'cmc', qp: 5, epics: [E.loco], disciplines: [D.prog, D.gameplay],
    landmark: PLAYTEST, tags: ['input', 'dash'],
    createdAt: '2025-05-06T08:30:00Z',
    body: `## Action Goal

Hook the dash input in the player controller and expose an \`OnDashRequested\`
event so gameplay systems can react without coupling to the input layer.

## Acceptance

- Buffer the input for 120 ms so the dash never feels dropped.
- Log \`dash_requested\` with the buffered timestamp for later tuning analysis.`,
  },
  {
    key: 'stam', kind: 'action', title: 'Integrate Stamina Consumption', status: 'active',
    priority: 'high', parent: 'cmc', qp: 3, epics: [E.loco], disciplines: [D.prog, D.gameplay],
    blockedBy: ['stamui'], landmark: PLAYTEST, tags: ['stamina'],
    createdAt: '2025-05-06T09:00:00Z',
    body: `## Action Goal

Deduct 25 stamina from \`PlayerStaminaPool\` when the dash trigger is accepted.
Clamp at zero and refuse the dash while depleted.

Depends on the stamina bar UI contract landing first.`,
  },
  {
    key: 'pol', kind: 'action', title: 'Movement System Polish & Tuning', status: 'unstarted',
    priority: 'critical', parent: 'cmc', qp: 8, epics: [E.loco], disciplines: [D.prog],
    landmark: PLAYTEST, createdAt: '2025-05-07T11:00:00Z',
    body: `## Tuning Pass

Clean up acceleration curves, friction and air control feel. Compare against
the reference feel passes in \`notes/feel\` before locking values.

QA opens once the first tuning pass is in a build.`,
  },
  {
    key: 'coy', kind: 'action', title: 'Coyote Time & Jump Buffering', status: 'unstarted',
    priority: 'low', parent: 'cmc', qp: 3, epics: [E.loco], disciplines: [D.prog],
    landmark: PLAYTEST, createdAt: '2025-05-10T14:00:00Z',
    body: 'Give the jump input the forgiving feel window that shipped in the previous slice.',
  },
  {
    key: 'fov', kind: 'action', title: 'Sprint FOV & Camera Kick', status: 'unstarted',
    priority: 'low', parent: 'cmc', qp: 2, epics: [E.loco], disciplines: [D.prog, D.art],
    landmark: PLAYTEST, createdAt: '2025-05-12T16:00:00Z',
    body: 'Sprint ramps the camera FOV +2.5 and adds a subtle kick on dash start.',
  },
  {
    key: 'wall', kind: 'idea', title: 'Wall Run (Exploratory)', status: 'unstarted',
    priority: 'low', parent: 'cmc', qp: 5, epics: [E.loco], disciplines: [D.design, D.gameplay],
    landmark: PLAYTEST, tags: ['pitch'], createdAt: '2025-05-08T10:00:00Z',
    body: `## Exploratory Pitch

Wall running that requires holding the jump key against a surface. Open
question: momentum retention and whether it belongs to core locomotion or
to the mobility ability kit.

> Speculative — not scheduled until the pitch is validated.`,
  },
  {
    key: 'slide', kind: 'idea', title: 'Add Sliding Mechanic', status: 'unstarted',
    priority: 'low', parent: 'cmc', qp: 3, epics: [E.loco], disciplines: [D.design],
    landmark: PLAYTEST, createdAt: '2025-05-08T10:15:00Z',
    body: `## Exploratory Pitch

Add a slide that cancels sprint velocity into a low-profile state. Candidate
for the Abilities epic if it needs a resource cost.`,
  },
  {
    key: 'crdash', kind: 'guard', title: 'Code Review: Dash Input Hook', status: 'unstarted',
    priority: 'medium', parent: 'cmc', qp: 1, epics: [E.loco], disciplines: [D.prog, D.qa],
    landmark: PLAYTEST, tags: ['code review'],
    createdAt: '2025-05-11T09:00:00Z',
    body: `## Quality Check Criteria

- [ ] Input buffering behaves on low FPS spikes.
- [ ] Dash state machine has no unreachable states.
- [ ] No allocs in the per-frame update path.`,
  },
  {
    key: 'qamove', kind: 'guard', title: 'QA Pass: Movement Core', status: 'unstarted',
    priority: 'medium', parent: 'cmc', qp: 2, epics: [E.loco], disciplines: [D.qa, D.gameplay],
    blockedBy: ['pol'], landmark: PLAYTEST, createdAt: '2025-05-12T09:00:00Z',
    body: `## Quality Check Criteria

- [ ] Feel review pass on acceleration / deceleration.
- [ ] No regressions on stairs or slopes in the vertical slice.
- [ ] Frame budget: movement update under 0.2 ms.`,
  },
  {
    key: 'qawall', kind: 'guard', title: 'QA Pass: Wall Run Prototype', status: 'vanquished',
    priority: 'low', parent: 'cmc', qp: 1, epics: [E.loco], disciplines: [D.qa],
    landmark: PLAYTEST, createdAt: '2025-05-13T10:00:00Z', completedAt: '2025-05-13T16:30:00Z',
    body: 'Prototype validated on the greybox arena; findings folded into the wall run pitch.',
  },
  /* ---- Camera & Feel card ------------------------------------------- */
  {
    key: 'cam', kind: 'card', title: 'Camera & Feel', status: 'active', priority: 'high',
    parent: 'loc', epics: [E.loco], disciplines: [D.prog, D.art], landmark: PLAYTEST,
    createdAt: '2025-05-15T09:00:00Z',
    body: 'Third-person camera response, collision and the "feel" budget shared with combat.',
  },
  {
    key: 'camcoll', kind: 'action', title: 'Camera Collision & Pushback', status: 'active',
    priority: 'high', parent: 'cam', qp: 5, epics: [E.loco], disciplines: [D.prog],
    landmark: PLAYTEST, createdAt: '2025-05-16T09:00:00Z',
    body: 'Slide the camera along occluders instead of snapping; keep the player visible in tight corridors.',
  },
  {
    key: 'shake', kind: 'idea', title: 'Camera Shake Presets', status: 'unstarted',
    priority: 'low', parent: 'cam', qp: 2, epics: [E.loco], disciplines: [D.design],
    landmark: PLAYTEST, createdAt: '2025-05-18T11:00:00Z',
    body: 'Authorable shake presets (impact / dash / landing) reusable from any gameplay event.',
  },
  /* ---- Ability System card ------------------------------------------ */
  {
    key: 'absys', kind: 'card', title: 'Ability System', status: 'active', priority: 'high',
    parent: 'ce', epics: [E.abil], disciplines: [D.prog, D.design], landmark: PLAYTEST,
    createdAt: '2025-06-01T09:00:00Z',
    body: 'Generic runtime for equipping, queuing and cooldown-tracking abilities on the player.',
  },
  {
    key: 'abqueue', kind: 'action', title: 'Implement Ability Queue', status: 'active',
    priority: 'critical', parent: 'absys', qp: 8, epics: [E.abil], disciplines: [D.prog],
    landmark: PLAYTEST, createdAt: '2025-06-02T09:00:00Z',
    body: 'Buffer the next queued ability while the current one animates; drop the queue on damage.',
  },
  {
    key: 'abcooldown', kind: 'action', title: 'Cooldown & Resource Ticker', status: 'active',
    priority: 'high', parent: 'absys', qp: 3, epics: [E.abil], disciplines: [D.prog],
    landmark: PLAYTEST, createdAt: '2025-06-03T09:00:00Z',
    body: 'Central ticker driving ability cooldowns and stamina/energy regen so the UI only reads state.',
  },
  {
    key: 'abint', kind: 'guard', title: 'Integration Review: Abilities', status: 'unstarted',
    priority: 'medium', parent: 'absys', qp: 2, epics: [E.abil], disciplines: [D.qa],
    blockedBy: ['abqueue'], landmark: PLAYTEST, createdAt: '2025-06-05T09:00:00Z',
    body: 'Verify the ability queue + cooldowns compose with the dash without double-spending stamina.',
  },
  /* ---- UI & HUD ------------------------------------------------------ */
  {
    key: 'uihud', kind: 'card', title: 'UI & HUD', status: 'active', priority: 'high',
    epics: [E.ui], disciplines: [D.art, D.prog], landmark: PLAYTEST,
    createdAt: '2025-04-20T09:00:00Z',
    body: 'HUD framework, screens and the shared UI design tokens for the whole game.',
  },
  {
    key: 'stamui', kind: 'action', title: 'Stamina Bar UI Rendering & Layout', status: 'unstarted',
    priority: 'high', parent: 'uihud', qp: 5, epics: [E.ui], disciplines: [D.art, D.prog],
    landmark: PLAYTEST, tags: ['hud'], createdAt: '2025-04-22T10:00:00Z',
    body: `\`PlayerStaminaPool\` drives the HUD bar; a low-stamina pulse at 25% and a lock flash when empty.`,
  },
  {
    key: 'qlog', kind: 'action', title: 'Quest Log Sheet', status: 'active',
    priority: 'medium', parent: 'uihud', qp: 3, epics: [E.ui], disciplines: [D.art],
    landmark: PLAYTEST, createdAt: '2025-04-24T10:00:00Z',
    body: 'Scrollable quest log grouped by active landmark with objective check states.',
  },
  {
    key: 'uiqa', kind: 'guard', title: 'UI QA: HUD Readability', status: 'unstarted',
    priority: 'medium', parent: 'uihud', qp: 1, epics: [E.ui], disciplines: [D.qa],
    landmark: PLAYTEST, createdAt: '2025-04-28T10:00:00Z',
    body: 'Legibility pass at 720p and under the colour-blind simulator.',
  },
  /* ---- Menus & Navigation card -------------------------------------- */
  {
    key: 'menu', kind: 'card', title: 'Menus & Navigation', status: 'unstarted',
    priority: 'medium', parent: 'uihud', epics: [E.ui], disciplines: [D.art],
    landmark: PLAYTEST, createdAt: '2025-04-21T09:00:00Z',
    body: 'Main menu, pause flow and settings screens. Reuses the HUD design tokens.',
  },
  {
    key: 'mainmenu', kind: 'action', title: 'Main Menu Rebuild', status: 'vanquished',
    priority: 'high', parent: 'menu', qp: 5, epics: [E.ui], disciplines: [D.art, D.prog],
    landmark: PLAYTEST, createdAt: '2025-03-02T09:00:00Z', completedAt: '2025-03-28T17:00:00Z',
    body: 'Ship the new title screen with the vertical-slice art. Done for the studio playtest.',
  },
  {
    key: 'pausemenu', kind: 'action', title: 'Pause Menu & Settings Sliders', status: 'unstarted',
    priority: 'medium', parent: 'menu', qp: 3, epics: [E.ui], disciplines: [D.art],
    landmark: PLAYTEST, createdAt: '2025-03-05T09:00:00Z',
    body: 'Pause menu with sensitivity, FOV and volume sliders persisted to the save file.',
  },
  {
    key: 'verify', kind: 'guard', title: 'Build Verification: Menu Flow', status: 'vanquished',
    priority: 'medium', parent: 'menu', qp: 1, epics: [E.ui], disciplines: [D.qa],
    landmark: PLAYTEST, createdAt: '2025-03-26T09:00:00Z', completedAt: '2025-03-28T18:00:00Z',
    body: 'Boot-to-main-menu and menu-to-game round trips verified on the playtest build.',
  },
  /* ---- Player -------------------------------------------------------- */
  {
    key: 'play', kind: 'card', title: 'Player Character', status: 'active', priority: 'high',
    epics: [E.player], disciplines: [D.prog, D.gameplay], landmark: PLAYTEST,
    createdAt: '2025-02-10T09:00:00Z',
    body: 'The playable character: stats, health, respawn and state exposure for the HUD and abilities.',
  },
  {
    key: 'hp', kind: 'action', title: 'Health & Damage Pipeline', status: 'active',
    priority: 'critical', parent: 'play', qp: 8, epics: [E.player], disciplines: [D.prog],
    landmark: PLAYTEST, createdAt: '2025-02-12T09:00:00Z',
    body: 'Damage funnels through a single pipeline: mitigation, invulnerability frames, then death.',
  },
  {
    key: 'respawn', kind: 'action', title: 'Respawn & Checkpoint Flow', status: 'active',
    priority: 'medium', parent: 'play', qp: 5, epics: [E.player], disciplines: [D.prog],
    landmark: PLAYTEST, createdAt: '2025-02-14T09:00:00Z',
    body: 'Checkpoint list plus a fade respawn that restores the correct ability loadout.',
  },
  {
    key: 'death', kind: 'idea', title: 'Death Cam & Slow-Mo', status: 'unstarted',
    priority: 'low', parent: 'play', qp: 3, epics: [E.player], disciplines: [D.design],
    landmark: PLAYTEST, createdAt: '2025-02-20T10:00:00Z',
    body: 'Short slow-mo on the killing blow before the fade-to-respawn. Pitch only.',
  },
  /* ---- Progression --------------------------------------------------- */
  {
    key: 'prog', kind: 'card', title: 'Progression', status: 'unstarted', priority: 'medium',
    epics: [E.prog], disciplines: [D.gameplay, D.design], landmark: PUBLISHER,
    createdAt: '2025-07-01T09:00:00Z',
    body: 'XP, levels and unlocks. Not part of the Steam Playtest scope.',
  },
  {
    key: 'xp', kind: 'action', title: 'XP & Level Curve', status: 'unstarted',
    priority: 'medium', parent: 'prog', qp: 5, epics: [E.prog], disciplines: [D.gameplay],
    landmark: PUBLISHER, createdAt: '2025-07-02T09:00:00Z',
    body: 'Author the level curve and expose milestones to the reward service.',
  },
  {
    key: 'skillui', kind: 'action', title: 'Skill Point UI', status: 'unstarted',
    priority: 'low', parent: 'prog', qp: 2, epics: [E.prog], disciplines: [D.art],
    landmark: PUBLISHER, createdAt: '2025-07-03T09:00:00Z',
    body: 'Spend screen for the skill points earned per level.',
  },
  /* ---- Inventory ----------------------------------------------------- */
  {
    key: 'inv', kind: 'card', title: 'Inventory', status: 'unstarted', priority: 'medium',
    epics: [E.inv], disciplines: [D.gameplay, D.prog], landmark: PUBLISHER,
    createdAt: '2025-07-05T09:00:00Z',
    body: 'Item data model, stacks and the grid UI. A later milestone.',
  },
  {
    key: 'invdata', kind: 'action', title: 'Inventory Data Model', status: 'unstarted',
    priority: 'medium', parent: 'inv', qp: 5, epics: [E.inv], disciplines: [D.prog],
    landmark: PUBLISHER, createdAt: '2025-07-06T09:00:00Z',
    body: 'Serializable item/stack model with idempotent add/remove semantics.',
  },
  {
    key: 'stack', kind: 'action', title: 'Pickup & Stacking', status: 'active',
    priority: 'medium', parent: 'inv', qp: 3, epics: [E.inv], disciplines: [D.prog],
    landmark: PUBLISHER, createdAt: '2025-07-07T09:00:00Z',
    body: 'World pickups merge into stacks up to the cap; overflow drops a new stack.',
  },
  /* ---- World --------------------------------------------------------- */
  {
    key: 'world', kind: 'card', title: 'World Systems', status: 'active', priority: 'medium',
    epics: [E.world], disciplines: [D.prog], landmark: PUBLISHER,
    createdAt: '2025-07-10T09:00:00Z',
    body: 'Time of day, weather and the streaming budget for the first real level.',
  },
  {
    key: 'daynight', kind: 'action', title: 'Day/Night Cycle', status: 'active',
    priority: 'low', parent: 'world', qp: 5, epics: [E.world], disciplines: [D.prog, D.art],
    landmark: PUBLISHER, createdAt: '2025-07-11T09:00:00Z',
    body: 'Authorable cycle length driving the directional light and fog.',
  },
  {
    key: 'weather', kind: 'action', title: 'Weather State Machine', status: 'active',
    priority: 'low', parent: 'world', qp: 2, epics: [E.world], disciplines: [D.prog],
    landmark: PUBLISHER, createdAt: '2025-07-12T09:00:00Z',
    body: 'Light-rain and clear states with cross-fade; placeholder art acceptable.',
  },
  /* ---- Tools & Editor ------------------------------------------------ */
  {
    key: 'tools', kind: 'card', title: 'Tools & Editor', status: 'unstarted', priority: 'low',
    epics: [E.tools], disciplines: [D.prog], landmark: PUBLISHER,
    createdAt: '2025-08-01T09:00:00Z',
    body: 'In-editor graph tools for the task engine itself. Empty card — scoped for later.',
  },
]

/* ------------------------------------------------------------------ */
/* Compile raw entries into computed NodeViews                         */
/* ------------------------------------------------------------------ */

const KIND_PREFIX: Record<string, string> = {
  card: 'CARD',
  action: 'ACT',
  guard: 'GRD',
  idea: 'IDEA',
}

function encode36(n: number): string {
  return n.toString(36).toUpperCase().padStart(4, '0')
}

function slugify(title: string): string {
  const slug = title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
  return slug || 'node'
}

/** Flagship nodes keep the ids the sample realm already uses. */
const VANITY_IDS: Record<string, string> = {
  cmc: 'CARD-K9F2',
  stam: 'ACT-3X7P',
  stamui: 'ACT-8J3W',
  crdash: 'GRD-7M2Q',
  uihud: 'CARD-4M1P',
}

export interface BuiltNode extends NodeView {
  _key: string
}

function compile(): { nodes: BuiltNode[]; byKey: Map<string, BuiltNode> } {
  const byKey = new Map<string, BuiltNode>()
  const idByKey = new Map<string, string>()
  const used = new Set<string>()

  // Claim vanity ids first so generated ids never collide with them.
  for (const e of RAW) {
    const vanity = VANITY_IDS[e.key]
    if (vanity) used.add(vanity)
  }
  let counter = 0
  for (const e of RAW) {
    if (VANITY_IDS[e.key]) {
      idByKey.set(e.key, VANITY_IDS[e.key]!)
    } else {
      let id = ''
      do {
        id = `${KIND_PREFIX[e.kind]}-${encode36(counter++)}`
      } while (used.has(id))
      used.add(id)
      idByKey.set(e.key, id)
    }
  }

  for (const e of RAW) {
    const id = idByKey.get(e.key)!
    const node: BuiltNode = {
      _key: e.key,
      id,
      kind: e.kind,
      title: e.title,
      status: e.status,
      effectiveStatus: e.status,
      blocked: false,
      priority: e.priority,
      disciplines: [...e.disciplines],
      epics: [...e.epics],
      questPoints: e.kind === 'card' ? null : (e.qp ?? null),
      landmark: null,
      explicitLandmark: e.landmark ?? null,
      parent: e.parent ? idByKey.get(e.parent) ?? null : null,
      blockedBy: (e.blockedBy ?? []).map((k) => idByKey.get(k) ?? k),
      totalQp: 0,
      lockedQpPercent: null,
      completedAt: e.completedAt ?? null,
      createdAt: e.createdAt,
      tags: [...(e.tags ?? [])],
      path: `Nodes/${id}_${slugify(e.title)}.md`,
      validationError: null,
      body: e.body,
    }
    byKey.set(e.key, node)
  }

  const nodes = [...byKey.values()]
  const byId = new Map(nodes.map((n) => [n.id, n]))
  const childrenOf = (id: string): NodeView[] => nodes.filter((n) => n.parent === id)

  // Landmark inheritance: walk up until an explicit landmark exists.
  const landmarkOf = (n: NodeView): string | null => {
    if (n.explicitLandmark) return n.explicitLandmark
    if (!n.parent) return null
    const p = byId.get(n.parent)
    return p ? landmarkOf(p) : null
  }

  // One memoised traversal computes done/blocked state, descendant QP and
  // locked QP (QP stranded behind blocked prerequisites).
  interface NodeInfo {
    done: boolean
    blocked: boolean
    totalQp: number
    lockedQp: number
  }
  const memo = new Map<string, NodeInfo>()
  const compute = (n: NodeView): NodeInfo => {
    const hit = memo.get(n.id)
    if (hit) return hit

    const kids = childrenOf(n.id)
    const kidInfos = kids.map(compute)
    const done = n.status === 'vanquished'

    const prereqOpen = n.blockedBy.some((bid) => {
      const dep = byId.get(bid)
      return dep ? !compute(dep).done : true
    })
    const descBlocked = kidInfos.some((k) => k.blocked)
    const blocked = !done && (prereqOpen || descBlocked)

    const totalQp = (n.questPoints ?? 0) + kidInfos.reduce((s, k) => s + k.totalQp, 0)
    // Locked QP = this node's own blocked children (their own QP) plus whatever
    // is locked deeper in each child's subtree.
    const lockedQp = kidInfos.reduce((s, info, i) => {
      const own = (kids[i]?.questPoints ?? 0)
      return s + (info.blocked ? own + info.lockedQp : 0)
    }, 0)

    const info: NodeInfo = { done, blocked, totalQp, lockedQp }
    memo.set(n.id, info)
    return info
  }

  for (const n of nodes) {
    const info = compute(n)
    n.blocked = info.blocked
    n.effectiveStatus = info.done ? 'vanquished' : info.blocked ? 'blocked' : n.status
    n.landmark = landmarkOf(n)
    n.totalQp = info.totalQp
    if (info.totalQp > 0 && info.lockedQp > 0) {
      n.lockedQpPercent = (info.lockedQp / info.totalQp) * 100
    }
  }

  return { nodes, byKey }
}

const COMPILED = compile()
export const MOCK_NODES: BuiltNode[] = COMPILED.nodes
export const MOCK_BY_ID: Map<string, BuiltNode> = new Map(MOCK_NODES.map((n) => [n.id, n]))
export const MOCK_BY_KEY: Map<string, BuiltNode> = COMPILED.byKey

/** Direct entry for the graph scene (key → positioned node). */
export function mockNode(key: string): BuiltNode {
  const n = MOCK_BY_KEY.get(key)
  if (!n) throw new Error(`mock key not found: ${key}`)
  return n
}

/* ------------------------------------------------------------------ */
/* Derived counts + filter row vocabulary                              */
/* ------------------------------------------------------------------ */

export interface FilterModel {
  search: string
  kinds: Set<string>
  statuses: Set<string>
  priorities: Set<string>
  epics: Set<string>
  disciplines: Set<string>
  landmarks: Set<string>
}

export function emptyFilters(): FilterModel {
  return {
    search: '',
    kinds: new Set<string>(),
    statuses: new Set<string>(),
    priorities: new Set<string>(),
    epics: new Set<string>(),
    disciplines: new Set<string>(),
    landmarks: new Set<string>(),
  }
}

export interface CheckRow {
  key: string
  label: string
  count: number
}

export const STATUS_LABEL: Record<string, string> = {
  active: 'Active',
  blocked: 'Blocked',
  unstarted: 'Unstarted',
  vanquished: 'Vanquished',
}
export const KIND_LABEL: Record<string, string> = {
  card: 'Card',
  action: 'Action',
  guard: 'Guard',
  idea: 'Idea',
}
export const PRIORITY_LABEL: Record<string, string> = {
  critical: 'Critical',
  high: 'High',
  medium: 'Medium',
  low: 'Low',
}

export function kindRows(nodes: readonly NodeView[] = MOCK_NODES): CheckRow[] {
  return (['card', 'action', 'guard', 'idea'] as const).map((k) => ({
    key: k,
    label: KIND_LABEL[k],
    count: nodes.filter((n) => n.kind === k).length,
  }))
}

export function statusRows(nodes: readonly NodeView[] = MOCK_NODES): CheckRow[] {
  return (['unstarted', 'active', 'blocked', 'vanquished'] as const).map((s) => ({
    key: s,
    label: STATUS_LABEL[s],
    count: nodes.filter((n) => n.effectiveStatus === s).length,
  }))
}

export function priorityRows(nodes: readonly NodeView[] = MOCK_NODES): CheckRow[] {
  return (['critical', 'high', 'medium', 'low'] as const).map((p) => ({
    key: p,
    label: PRIORITY_LABEL[p],
    count: nodes.filter((n) => n.priority === p).length,
  }))
}

/**
 * Hierarchical path vocabulary presented as a flat checklist. A deep path
 * like `Combat Engine/Locomotion` contributes its leaf (`Locomotion`) so the
 * list stays readable; matching treats a label as addressable anywhere it
 * appears: as a whole path, as a path prefix, or as a segment of a path.
 */
function branchRows(
  field: 'epics' | 'disciplines',
  nodes: readonly NodeView[] = MOCK_NODES,
): CheckRow[] {
  const labels = new Set<string>()
  for (const n of nodes) {
    for (const path of n[field]) {
      const parts = path.split('/')
      labels.add(parts.length > 1 ? (parts[parts.length - 1] as string) : path)
    }
  }
  const rows: CheckRow[] = []
  for (const label of labels) {
    const count = nodes.filter((n) => n[field].some((p) => labelMatches(p, label))).length
    rows.push({ key: label, label, count })
  }
  return rows.sort((a, b) => b.count - a.count || a.label.localeCompare(b.label))
}

function labelMatches(path: string, label: string): boolean {
  return path === label || path.startsWith(`${label}/`) || path.split('/').includes(label)
}

export function epicRows(nodes?: readonly NodeView[]): CheckRow[] {
  return branchRows('epics', nodes)
}

export function disciplineRows(nodes?: readonly NodeView[]): CheckRow[] {
  return branchRows('disciplines', nodes)
}

export function landmarkRows(nodes: readonly NodeView[] = MOCK_NODES): CheckRow[] {
  const counts = new Map<string, number>()
  for (const n of nodes) {
    if (!n.landmark) continue
    counts.set(n.landmark, (counts.get(n.landmark) ?? 0) + 1)
  }
  return [...counts.entries()]
    .map(([key, count]) => ({ key, label: key, count }))
    .sort((a, b) => b.count - a.count || a.label.localeCompare(b.label))
}

/** One node in a collapsible taxonomy hierarchy. `children` present when
 * slash-path values ("Programming/Locomotion") exist under this root. */
export interface TaxRow {
  /** Filter address: the value nodes must match (root prefixes match below). */
  key: string
  label: string
  count: number
  children?: TaxRow[]
}

/**
 * Taxonomy navigator rows for an epic or discipline vocabulary (doc 05 §5.4).
 * Slash-path values group under their root category ("Combat_Engine" owns
 * "Combat_Engine/Locomotion"…); flat values stay top-level leaves, and a root
 * used bare (e.g. epic on the category card itself) is folded into the root
 * row's count. Row counts are unique nodes attached to that branch or any
 * sub-branch, so checking a root filters its whole subtree via the existing
 * prefix matching in `matchesFilters`.
 */
export function taxTree(
  field: 'epics' | 'disciplines',
  nodes: readonly NodeView[] = MOCK_NODES,
): TaxRow[] {
  const byValue = new Map<string, number>()
  for (const n of nodes) {
    for (const v of n[field]) {
      byValue.set(v, (byValue.get(v) ?? 0) + 1)
    }
  }
  if (![...byValue.keys()].some((v) => v.includes('/'))) {
    return [...byValue.entries()]
      .map(([key, count]) => ({ key, label: key, count }))
      .sort((a, b) => b.count - a.count || a.label.localeCompare(b.label))
  }

  const byRoot = new Map<string, { key: string; label: string; count: number }[]>()
  const bare: { key: string; label: string; count: number }[] = []
  for (const [value, count] of byValue) {
    const idx = value.indexOf('/')
    if (idx === -1) {
      bare.push({ key: value, label: value, count })
      continue
    }
    const root = value.slice(0, idx)
    const kids = byRoot.get(root) ?? []
    kids.push({ key: value, label: value.slice(idx + 1), count })
    byRoot.set(root, kids)
  }

  const rows: TaxRow[] = []
  for (const [root, kids] of byRoot) {
    const attached = new Set<string>()
    for (const n of nodes) {
      if (n[field].some((p) => p === root || p.startsWith(`${root}/`))) attached.add(n.id)
    }
    kids.sort((a, b) => b.count - a.count || a.label.localeCompare(b.label))
    rows.push({ key: root, label: root, count: attached.size, children: kids })
  }
  for (const b of bare) rows.push(b)
  return rows.sort((a, b) => b.count - a.count || a.label.localeCompare(b.label))
}

/* ------------------------------------------------------------------ */
/* Filter matching                                                     */
/* ------------------------------------------------------------------ */

export function matchesFilters(n: NodeView, f: FilterModel): boolean {
  if (f.kinds.size && !f.kinds.has(n.kind)) return false
  if (f.statuses.size && !f.statuses.has(n.effectiveStatus)) return false
  if (f.priorities.size && !f.priorities.has(n.priority)) return false
  if (f.epics.size && !pathAny(n.epics, f.epics)) return false
  if (f.disciplines.size && !pathAny(n.disciplines, f.disciplines)) return false
  if (f.landmarks.size && !(n.landmark !== null && f.landmarks.has(n.landmark))) return false

  const q = f.search.trim().toLowerCase()
  if (q) {
    const hay = [
      n.title, n.id, n.kind, n.effectiveStatus, n.status, n.priority,
      ...n.tags, ...n.epics, ...n.disciplines,
      n.landmark ?? '', n.parent ?? '', ...n.blockedBy,
    ]
      .join(' ')
      .toLowerCase()
    if (!hay.includes(q)) return false
  }
  return true
}

function pathAny(paths: string[], selected: Set<string>): boolean {
  return paths.some((p) => {
    for (const s of selected) {
      if (labelMatches(p, s)) return true
    }
    return false
  })
}

export function toggle<T>(set: Set<T>, value: T): Set<T> {
  const next = new Set(set)
  if (next.has(value)) next.delete(value)
  else next.add(value)
  return next
}

/** Edges across the whole realm (parent links + dependencies). */
export function linkCount(nodes: readonly NodeView[] = MOCK_NODES): number {
  let n = 0
  for (const node of nodes) {
    if (node.parent) n += 1
    n += node.blockedBy.length
  }
  return n
}

export function statusTotals(nodes: readonly NodeView[] = MOCK_NODES): { nodes: number; links: number } {
  return { nodes: nodes.length, links: linkCount(nodes) }
}

/* ------------------------------------------------------------------ */
/* Local graph scene (the workspace mock)                              */
/* ------------------------------------------------------------------ */

export interface SceneSpot {
  key: string
  x: number
  y: number
}

export interface SceneLink {
  from: string
  to: string
  dashed?: boolean
}export const SCENE_WIDTH = 860
export const SCENE_HEIGHT = 500

/** Layout of the local graph centred on Character Movement Core. */
export const SCENE_SPOTS: SceneSpot[] = [
  { key: 'loc', x: 430, y: 64 },
  { key: 'cmc', x: 430, y: 200 },
  { key: 'dash', x: 120, y: 348 },
  { key: 'stam', x: 320, y: 348 },
  { key: 'pol', x: 520, y: 348 },
  { key: 'wall', x: 720, y: 348 },
  { key: 'crdash', x: 120, y: 452 },
  { key: 'slide', x: 320, y: 452 },
  { key: 'qamove', x: 520, y: 452 },
]

/** Scene edges: parent edges (thin) + dependency edges (arrowed). */
export const SCENE_LINKS: SceneLink[] = [
  { from: 'loc', to: 'cmc' },
  { from: 'cmc', to: 'dash' },
  { from: 'cmc', to: 'stam' },
  { from: 'cmc', to: 'pol' },
  { from: 'cmc', to: 'wall', dashed: true },
  { from: 'dash', to: 'crdash' },
  { from: 'pol', to: 'qamove' },
  { from: 'pol', to: 'slide', dashed: true },
]
