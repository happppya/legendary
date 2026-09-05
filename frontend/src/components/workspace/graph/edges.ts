// Directed edges for the graph scene: the curated mock links, or — for a
// real realm — every parent edge plus every blocked_by dependency edge that
// points at a node present in the payload. Also owns the geometry that turns
// a scene + edge defs into SVG lines with per-edge style markers.

import type { NodeView } from '../../../types'
import { SCENE_LINKS } from '../../../lib/mockRealm'
import type { SceneItem } from './scene'

export interface SceneLinkDef {
  from: string
  to: string
  dashed?: boolean
  /** A true prerequisite edge (blocked_by), drawn with a dependency arrow. */
  dep?: boolean
}

export function sceneEdges(
  isMockScene: boolean,
  nodesById: Map<string, NodeView>,
): SceneLinkDef[] {
  if (isMockScene) return SCENE_LINKS
  const out: SceneLinkDef[] = []
  for (const [id, node] of nodesById) {
    if (node.parent && nodesById.has(node.parent)) out.push({ from: node.parent, to: id })
    for (const b of node.blockedBy) {
      if (nodesById.has(b)) out.push({ from: b, to: id, dep: true })
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
