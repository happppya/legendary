// Generic helpers over NodeView realm data, shared by panes that render
// nodes (table, board, entities, detail, graph breadcrumbs). No demo data
// lives here.

import type { NodeView } from '../types'

export function capitalize(s: string): string {
  return s ? s[0]!.toUpperCase() + s.slice(1) : s
}

/**
 * Quest points a view should display for a node: cards show their aggregate
 * total only when it is non-zero (a card's own `questPoints` is always
 * null), everything else shows its authored points. `null` means "no QP to
 * show" (an empty card, or an action without a point value).
 */
export function qpOf(n: NodeView): number | null {
  if (n.kind === 'card') return n.totalQp > 0 ? n.totalQp : null
  return n.questPoints
}

/**
 * Parent chain from the realm root down to `id` (inclusive), oldest first.
 * Walks `parent` links; a missing node or unknown id yields an empty chain.
 * Used by the detail breadcrumb and the graph trail.
 */
export function ancestorChain(
  nodesById: ReadonlyMap<string, NodeView>,
  id: string | null,
): NodeView[] {
  if (!id) return []
  const out: NodeView[] = []
  let cur = nodesById.get(id) ?? null
  const seen = new Set<string>()
  while (cur) {
    if (seen.has(cur.id)) break
    seen.add(cur.id)
    out.unshift(cur)
    cur = cur.parent ? (nodesById.get(cur.parent) ?? null) : null
  }
  return out
}

/** Edges across a node list (parent links + blocked_by dependencies). */
export function linkCount(nodes: readonly NodeView[]): number {
  let n = 0
  for (const node of nodes) {
    if (node.parent) n += 1
    n += node.blockedBy.length
  }
  return n
}

export function statusTotals(nodes: readonly NodeView[]): { nodes: number; links: number } {
  return { nodes: nodes.length, links: linkCount(nodes) }
}
