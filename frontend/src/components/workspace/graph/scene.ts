// Scene construction for the graph pane: sizes per node kind, deterministic
// pseudo-random seeding, and the "auto-layout scatter" used to restart the
// force simulator. Pure geometry — no Vue state here (GraphPane owns the
// `live` ref and the rAF loop).

import type { NodeView } from '../../../types'
import { MOCK_BY_KEY, SCENE_HEIGHT, SCENE_SPOTS, SCENE_WIDTH } from '../../../lib/mockRealm'

/** Sizes per kind (centred at x,y). */
export const KIND_SIZE: Record<string, { w: number; h: number }> = {
  card: { w: 190, h: 56 },
  action: { w: 172, h: 30 },
  idea: { w: 172, h: 30 },
  guard: { w: 46, h: 46 },
}

/** Whole-realm scenes above this node count wait for the Wasm physics
 * engine (doc 07 / Milestone 5) instead of the prototype simulator. */
export const MAX_SCENE_NODES = 200

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
}

/** Fresh scene items: the curated mock layout, or a force-laid ring over
 * the whole realm (seedN-driven so a re-seed settles identically). */
export function buildSceneItems(src: SceneSource): SceneItem[] {
  const out: SceneItem[] = []
  if (src.isMockScene) {
    // Curated local scene around Character Movement Core.
    for (const spot of SCENE_SPOTS) {
      const id = MOCK_BY_KEY.get(spot.key)?.id ?? spot.key
      const node = src.nodesById.get(id)
      if (!node) continue
      const size = KIND_SIZE[node.kind] ?? KIND_SIZE.action
      // deterministic tiny jitter so the first auto-layout visibly settles
      const jitter = (seedN(spot.key) - 0.5) * 14
      out.push({
        key: spot.key,
        node,
        x: spot.x,
        y: spot.y + jitter,
        w: size.w,
        h: size.h,
        vx: (seedN(spot.key + 'x') - 0.5) * 1.6,
        vy: (seedN(spot.key + 'y') - 0.5) * 1.6,
      })
    }
    return out
  }
  // Real realm: force-lay the whole realm tree, seeded on a ring so the
  // simulator visibly settles into the dependency structure.
  const size = src.nodesById.size
  if (size === 0 || size > MAX_SCENE_NODES) return out
  let i = 0
  for (const node of src.nodesById.values()) {
    const dim = KIND_SIZE[node.kind] ?? KIND_SIZE.action
    const p = scatterPosition(node.id, i, size)
    out.push({
      key: node.id,
      node,
      x: p.x,
      y: p.y,
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
 * open/reload, or the mock/local scene swaps). Dragging and auto-layout do
 * not touch it, so a settled layout survives navigation. */
export function sceneSignature(src: SceneSource): string {
  if (src.isMockScene) {
    return 'm|' + SCENE_SPOTS.map((s) => s.key).sort().join('|')
  }
  return 'r|' + [...src.nodesById.keys()].sort().join('|')
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
