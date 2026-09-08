// Scene construction for the graph pane: sizes per node kind, deterministic
// pseudo-random seeding, and the "auto-layout scatter" used to restart the
// force simulator. Pure geometry — no Vue state here (GraphPane owns the
// `live` ref and the rAF loop).
//
// Three graph scopes drive which nodes a scene holds (test-feedback A-1):
// - `local`:  the selected node's neighbourhood (mock: curated spots;
//             realm: focused subgraph re-centred on the selection).
// - `global`: every node in the realm (Feature A-a).
// - `overview`: Realm Origin → genre roots, recursing only through genre
//             nodes and terminating at the first non-genre descendant
//             (Feature A-b).

import type { NodeView } from '../../../types'
import { MOCK_BY_KEY, SCENE_HEIGHT, SCENE_SPOTS, SCENE_WIDTH } from '../../../lib/mockRealm'

/** Sizes per kind (centred at x,y). */
export const KIND_SIZE: Record<string, { w: number; h: number }> = {
  card: { w: 190, h: 56 },
  genre: { w: 190, h: 56 },
  action: { w: 172, h: 30 },
  idea: { w: 172, h: 30 },
  guard: { w: 46, h: 46 },
}

/** Whole-realm scenes above this node count wait for the Wasm physics
 * engine (doc 07 / Milestone 5) instead of the prototype simulator. */
export const MAX_SCENE_NODES = 200

/** Graph scope: local neighbourhood, whole realm, or genre overview. */
export type GraphScope = 'local' | 'global' | 'overview'

/** Node metadata + live physics state, keyed by scene spot key. */
export interface SceneItem {
  key: string
  node: NodeView
  x: number
  y: number
  w: number
  h: number
  vx: number
  vy: number
}

export function sceneCenter(): { x: number; y: number } {
  return { x: SCENE_WIDTH / 2, y: SCENE_HEIGHT / 2 }
}

/** Cheap deterministic pseudo-random for stable-but-varied starts. */
export function seedN(s: string): number {
  let h = 2166136261
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return ((h >>> 0) % 1000) / 1000
}

/** Deterministic scatter for realm-wide scenes (ring around the centre). */
function scatterPosition(key: string, index: number, total: number) {
  const { x: cx, y: cy } = sceneCenter()
  const base = Math.min(SCENE_WIDTH, SCENE_HEIGHT) * 0.42
  const ang = (index / total) * Math.PI * 2 + seedN(key) * 1.4
  const rad = base * (0.5 + 0.9 * seedN(key + 'r'))
  return {
    x: cx + Math.cos(ang) * rad,
    y: cy + Math.sin(ang) * rad,
    vx: (seedN(key + 'vx') - 0.5) * 2.4,
    vy: (seedN(key + 'vy') - 0.5) * 2.4,
  }
}

export interface SceneSource {
  isMockScene: boolean
  nodesById: Map<string, NodeView>
  /** Selected node id (drives the local scope's centre). */
  selectedId: string | null
  scope: GraphScope
}

/** Ids held by a scope: everything (global), the genre overview frontier,
 * or the local neighbourhood around the selection. */
export function scopeIds(src: SceneSource): Set<string> | null {
  const byId = src.nodesById
  if (src.scope === 'global') return null // null = all nodes
  if (src.scope === 'overview') {
    // Realm Origin → every top-level node; recurse only through genre
    // nodes. The scene shows genres plus their immediate non-genre children
    // (the frontier), wherever those children sit in the tree.
    const out = new Set<string>()
    const roots = [...byId.values()].filter((n) => n.parent === null)
    const walk = (id: string) => {
      for (const n of byId.values()) {
        if (n.parent !== id) continue
        out.add(n.id)
        if (n.kind === 'genre') walk(n.id)
      }
    }
    for (const r of roots) {
      out.add(r.id)
      if (r.kind === 'genre') walk(r.id)
    }
    return out
  }
  // local: the selection plus its ancestors, siblings and direct children
  // (empty → fall back to the whole realm so the canvas is never blank).
  const focus = src.selectedId ? byId.get(src.selectedId) : null
  if (!focus) return null
  const out = new Set<string>([focus.id])
  for (const n of byId.values()) {
    if (n.parent === focus.id) out.add(n.id)
    if (focus.parent && n.id === focus.parent) out.add(n.id)
    if (n.parent && focus.parent && n.parent === focus.parent) out.add(n.id)
  }
  return out.size > 1 ? out : null
}

/** Fresh scene items for the scope: the local mock scope seeds members onto
 * their curated demo spots (when they have one), everything else is a
 * force-laid ring over the scope's members. */
export function buildSceneItems(src: SceneSource): SceneItem[] {
  const out: SceneItem[] = []
  const keep = scopeIds(src)
  const members = keep
    ? [...src.nodesById.values()].filter((n) => keep.has(n.id))
    : [...src.nodesById.values()]
  const size = members.length
  if (size === 0 || size > MAX_SCENE_NODES) return out
  // Reverse lookup: mock node id → curated spot key (demo scene only).
  const spotById = new Map<string, string>()
  if (src.isMockScene) {
    for (const [key, node] of MOCK_BY_KEY) {
      if (SCENE_SPOTS.some((s) => s.key === key)) spotById.set(node.id, key)
    }
  }
  let i = 0
  for (const node of members) {
    const dim = KIND_SIZE[node.kind] ?? KIND_SIZE.action
    const spotKey = spotById.get(node.id)
    const spot = spotKey ? SCENE_SPOTS.find((s) => s.key === spotKey) : undefined
    const p = scatterPosition(node.id, i, size)
    // deterministic tiny jitter so the first auto-layout visibly settles
    const jitter = (seedN(spotKey ?? node.id) - 0.5) * 14
    out.push({
      key: node.id,
      node,
      x: spot ? spot.x : p.x,
      y: spot ? spot.y + jitter : p.y,
      w: dim.w,
      h: dim.h,
      vx: p.vx,
      vy: p.vy,
    })
    i += 1
  }
  return out
}

/** Scene signature — re-seed only when the member set really changes (realm
 * open/reload, scope swap, or a local-scope selection change that reshapes
 * the neighbourhood). Dragging and auto-layout do not touch it, so a
 * settled layout survives navigation. */
export function sceneSignature(src: SceneSource): string {
  const focus = src.scope === 'local' ? (src.selectedId ?? '') : ''
  const keep = scopeIds(src)
  const ids = keep
    ? [...src.nodesById.keys()].filter((id) => keep.has(id)).sort()
    : [...src.nodesById.keys()].sort()
  return `${src.scope[0]}|${focus}|${ids.join('|')}`
}

/** Scatter every item onto a ring and re-kick its velocity; the simulator
 * then re-settles the cluster. Mutates the passed items in place. */
export function scatterAll(items: SceneItem[], velocity = 3): void {
  const { x: cx, y: cy } = sceneCenter()
  const base = Math.min(SCENE_WIDTH, SCENE_HEIGHT) * 0.42
  for (let i = 0; i < items.length; i++) {
    const ang = (i / items.length) * Math.PI * 2 + 0.6
    const rad = base * (0.55 + 0.9 * seedN(items[i]!.key + 'r'))
    items[i]!.x = cx + Math.cos(ang) * rad
    items[i]!.y = cy + Math.sin(ang) * rad
    items[i]!.vx = (seedN(items[i]!.key + 'vx') - 0.5) * velocity
    items[i]!.vy = (seedN(items[i]!.key + 'vy') - 0.5) * velocity
  }
}
