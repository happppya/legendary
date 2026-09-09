// Directed edges for the graph scene: the curated mock links, or — for a
// real realm — every parent edge plus every blocked_by dependency edge that
// points at a node present in the payload. Also owns the geometry that turns
// a scene + edge defs into SVG lines with per-edge style markers.

import type { NodeView } from '../../../types'
import { MOCK_BY_KEY, SCENE_LINKS } from '../../../lib/mockRealm'
import type { SceneItem } from './scene'

export interface SceneLinkDef {
  from: string
  to: string
  dashed?: boolean
  /** A true prerequisite edge (blocked_by), drawn with a dependency arrow. */
  dep?: boolean
}

/** Edges restricted to the nodes currently on the scene. `byKey` holds the
 * scene members (node ids — scene items are keyed by node id in every
 * scope). */
export function sceneEdges(
  isMockScene: boolean,
  nodesById: Map<string, NodeView>,
  byKey?: Map<string, unknown>,
): SceneLinkDef[] {
  if (isMockScene) {
    // SCENE_LINKS are authored with curated spot keys (`loc`, `cmc`, …);
    // the live scene keys items by node id, so normalize each endpoint to
    // its node id before restricting to the visible members.
    const idOf = (k: string): string | null => MOCK_BY_KEY.get(k)?.id ?? k
    const curated: SceneLinkDef[] = []
    const seen = new Set<string>()
    for (const l of SCENE_LINKS) {
      const from = idOf(l.from)
      const to = idOf(l.to)
      if (!from || !to) continue
      if (byKey && (!byKey.has(from) || !byKey.has(to))) continue
      seen.add(`${from}→${to}`)
      curated.push({ from, to, dashed: l.dashed })
    }
    // Beyond the local scene, fall back to the same derivation the real
    // realm uses (parent + blocked_by) so non-local scopes are not blank:
    // curated links stay (they carry the demo's dashed styling) and every
    // other structural edge is added once, deduped against them.
    for (const [id, node] of nodesById) {
      const to = idOf(id)
      if (!to || (byKey && !byKey.has(to))) continue
      const from = node.parent ? idOf(node.parent) : null
      if (from && (!byKey || byKey.has(from)) && !seen.has(`${from}→${to}`)) {
        seen.add(`${from}→${to}`)
        curated.push({ from, to })
      }
      for (const b of node.blockedBy) {
        const dep = idOf(b)
        if (dep && (!byKey || byKey.has(dep)) && !seen.has(`${dep}→${to}`)) {
          seen.add(`${dep}→${to}`)
          curated.push({ from: dep, to, dep: true })
        }
      }
    }
    return curated
  }
  const present = byKey ?? nodesById
  const keyOf = (id: string): string | null => {
    if (present.has(id)) return id
    if (byKey && MOCK_BY_KEY.size) {
      for (const [k, n] of MOCK_BY_KEY) if (n.id === id && byKey.has(k)) return k
    }
    return null
  }
  const out: SceneLinkDef[] = []
  for (const [id, node] of nodesById) {
    const to = keyOf(id)
    if (!to) continue
    const from = node.parent ? keyOf(node.parent) : null
    if (from) out.push({ from, to })
    for (const b of node.blockedBy) {
      const dep = keyOf(b)
      if (dep) out.push({ from: dep, to, dep: true })
    }
  }
  return out
}

/** Linked pairs (both directions normalised) — drives the edge springs. */
export function linkedPairs(defs: SceneLinkDef[]): Set<string> {
  const s = new Set<string>()
  for (const l of defs) s.add(`${l.from}␟${l.to}`)
  return s
}

export interface DrawnEdge {
  dashed: boolean
  dep: boolean
  x1: number
  y1: number
  x2: number
  y2: number
}

/** Clamp each edge to the shape boundary of its endpoints. */
export function drawEdges(items: SceneItem[], defs: SceneLinkDef[]): DrawnEdge[] {
  const byKey = new Map(items.map((s) => [s.key, s]))
  const out: DrawnEdge[] = []
  for (const link of defs) {
    const a = byKey.get(link.from)
    const b = byKey.get(link.to)
    if (!a || !b) continue
    const ax = a.x + (a.x < b.x ? a.w / 2 : a.x > b.x ? -a.w / 2 : 0)
    const ay = a.y + (a.y < b.y ? a.h / 2 : a.y > b.y ? -a.h / 2 : 0)
    const bx = b.x + (b.x > a.x ? -b.w / 2 : b.x < a.x ? b.w / 2 : 0)
    const by = b.y + (b.y > a.y ? -b.h / 2 : b.y < a.y ? b.h / 2 : 0)
    out.push({
      dashed: !!link.dashed,
      dep: !!link.dep,
      x1: ax,
      y1: ay,
      x2: bx,
      y2: by,
    })
  }
  return out
}
