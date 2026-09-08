// Generic helpers over NodeView realm data, shared by panes that render
// nodes (table, board, entities, detail, graph breadcrumbs). No demo data
// lives here.

import type { NodeView } from '../types'
import { isContainerKind } from './kind'

export function capitalize(s: string): string {
  return s ? s[0]!.toUpperCase() + s.slice(1) : s
}

/**
 * Quest points a view should display for a node: containers (cards, genres)
 * show their aggregate total only when it is non-zero (a container's own
 * `questPoints` is always null), everything else shows its authored points.
 * `null` means "no QP to show" (an empty card, or an action without a
 * point value).
 */
export function qpOf(n: NodeView): number | null {
  if (isContainerKind(n.kind)) return n.totalQp > 0 ? n.totalQp : null
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

/**
 * Realm Map completed-task visibility levels (doc 05 §5.5 B).
 * Level 0 hides every vanquished node; level 1 hides only top-level Card
 * subgraphs whose descendants are 100% vanquished; level 2 shows all.
 */
export type CompletionLevel = 0 | 1 | 2

/** Is a Card's whole descendant tree vanquished? (vanquished nodes have no
 * visible children in the payload that are not themselves vanquished). */
export function fullyVanquished(nodesById: ReadonlyMap<string, NodeView>, root: NodeView): boolean {
  const stack = [root.id]
  while (stack.length) {
    const cur = stack.pop()
    for (const n of nodesById.values()) {
      if (n.parent === cur) {
        if (n.effectiveStatus !== 'vanquished') return false
        stack.push(n.id)
      }
    }
  }
  return root.effectiveStatus === 'vanquished'
}

/**
 * Filter a node list by completed-task visibility level. Level 1 keeps
 * partially-completed top-level Card subgraphs intact (their vanquished
 * members stay visible inside a kept Card).
 */
export function filterByCompletionLevel(
  nodes: readonly NodeView[],
  level: CompletionLevel,
): NodeView[] {
  if (level === 2) return [...nodes]
  const byId = new Map(nodes.map((n) => [n.id, n]))
  return nodes.filter((n) => {
    if (level === 0) return n.effectiveStatus !== 'vanquished'
    // level 1: hide vanquished nodes, but keep any node inside a kept
    // (partially-completed) top-level Card subgraph
    if (n.effectiveStatus !== 'vanquished') return true
    // walk up to the top-level ancestor
    let cur = n
    const seen = new Set<string>()
    while (cur.parent && !seen.has(cur.id)) {
      seen.add(cur.id)
      const p = byId.get(cur.parent)
      if (!p) break
      cur = p
    }
    if (!isContainerKind(cur.kind)) return false
    return !fullyVanquished(byId, cur)
  })
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
