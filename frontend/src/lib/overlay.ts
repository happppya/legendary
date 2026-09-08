// Overlay color engine for the graph canvas (doc 05 §5.3). Maps a node's
// components onto stroke colors for each toggleable overlay mode:
// Epic / Discipline (categorical palettes), Priority (warmth scale),
// Status (lifecycle colors), Effort (QP heatmap). `null` means "no overlay
// value" — the shape keeps its kind color.

import type { NodeView } from '../types'
import { PRIORITY_RANK } from './priority'
import { isContainerKind } from './kind'

export type OverlayMode = 'none' | 'epic' | 'discipline' | 'priority' | 'status' | 'effort'

/** Ordered overlay modes for cycling and menus. */
export const OVERLAY_MODES: readonly OverlayMode[] = [
  'none',
  'epic',
  'discipline',
  'priority',
  'status',
  'effort',
]

export const OVERLAY_LABEL: Record<OverlayMode, string> = {
  none: 'Kind (default)',
  epic: 'Color by Epic',
  discipline: 'Color by Discipline',
  priority: 'Color by Priority',
  status: 'Color by Status',
  effort: 'Effort (QP heatmap)',
}

/** Categorical palette for epic/discipline branches. Warm, distinguishable
 * hues that also read as different lightness levels. */
const CATEGORY_COLORS = [
  '#e0a458', // amber
  '#7fb069', // green
  '#5b8db8', // blue
  '#c56c86', // rose
  '#9b7fc0', // violet
  '#5fb3a1', // teal
  '#d08770', // coral
  '#8fa860', // olive
  '#b8845f', // ochre
  '#7c9ec9', // steel
]

/** Deterministically assign each branch path a palette color. */
export function categoryColorMap(paths: string[]): Map<string, string> {
  const map = new Map<string, string>()
  const sorted = [...paths].sort()
  sorted.forEach((p, i) => map.set(p, CATEGORY_COLORS[i % CATEGORY_COLORS.length] as string))
  return map
}

/** Status overlay: lifecycle colors (doc 05 §5.3 — blocked warm warning,
 * active blue, vanquished muted gray, unstarted neutral). */
function statusColor(n: NodeView): string {
  if (n.blocked) return '#d67866'
  switch (n.effectiveStatus) {
    case 'active':
      return '#5b8db8'
    case 'vanquished':
      return '#6b7280'
    default:
      return '#9aa3ad'
  }
}

/** Priority overlay: warmth scale critical → low. */
function priorityColor(n: NodeView): string {
  switch (PRIORITY_RANK[n.priority] ?? 3) {
    case 0:
      return '#d64545'
    case 1:
      return '#e08a3c'
    case 2:
      return '#d0b845'
    default:
      return '#7d8590'
  }
}

/** Effort overlay: QP heatmap (small = cool, large = hot). Fibonacci scale. */
function effortColor(n: NodeView): string | null {
  const qp = isContainerKind(n.kind) ? (n.totalQp > 0 ? n.totalQp : null) : n.questPoints
  if (qp === null || qp <= 0) return null
  if (qp <= 2) return '#5b8db8'
  if (qp <= 5) return '#5fb3a1'
  if (qp <= 8) return '#d0b845'
  if (qp <= 13) return '#e08a3c'
  return '#d64545'
}

/** The branch of `n` that owns a color in the given map: the first of the
 * node's paths (for epics/disciplines) with a mapping, falling back to the
 * node's first path (roots own their subtree's color via prefix keys). */
function branchColor(n: NodeView, field: 'epics' | 'disciplines', colors: Map<string, string>): string | null {
  const paths = n[field]
  if (!paths.length) return null
  for (const p of paths) {
    const direct = colors.get(p)
    if (direct) return direct
  }
  // walk up the path segments to find the nearest colored ancestor branch
  for (const p of paths) {
    const parts = p.split('/')
    for (let i = parts.length - 1; i > 0; i--) {
      const prefix = parts.slice(0, i).join('/')
      const c = colors.get(prefix)
      if (c) return c
    }
  }
  return null
}

/** Stroke color for a node under an overlay mode, or null to keep the kind
 * color. `epicColors`/`discColors` come from `categoryColorMap` over the
 * realm's declared branch paths. */
export function overlayColor(
  mode: OverlayMode,
  n: NodeView,
  epicColors: Map<string, string>,
  discColors: Map<string, string>,
): string | null {
  switch (mode) {
    case 'epic':
      return branchColor(n, 'epics', epicColors)
    case 'discipline':
      return branchColor(n, 'disciplines', discColors)
    case 'priority':
      return priorityColor(n)
    case 'status':
      return statusColor(n)
    case 'effort':
      return effortColor(n)
    default:
      return null
  }
}

/** Legend entries for an overlay mode: { label, color } pairs. */
export function overlayLegend(
  mode: OverlayMode,
  epicColors: Map<string, string>,
  discColors: Map<string, string>,
): { label: string; color: string }[] {
  switch (mode) {
    case 'epic':
      return [...epicColors.entries()].map(([label, color]) => ({ label, color }))
    case 'discipline':
      return [...discColors.entries()].map(([label, color]) => ({ label, color }))
    case 'priority':
      return [
        { label: 'Critical', color: '#d64545' },
        { label: 'High', color: '#e08a3c' },
        { label: 'Medium', color: '#d0b845' },
        { label: 'Low', color: '#7d8590' },
      ]
    case 'status':
      return [
        { label: 'Blocked', color: '#d67866' },
        { label: 'Active', color: '#5b8db8' },
        { label: 'Unstarted', color: '#9aa3ad' },
        { label: 'Vanquished', color: '#6b7280' },
      ]
    case 'effort':
      return [
        { label: 'QP 1–2', color: '#5b8db8' },
        { label: 'QP 3–5', color: '#5fb3a1' },
        { label: 'QP 8', color: '#d0b845' },
        { label: 'QP 13', color: '#e08a3c' },
        { label: 'QP 21+', color: '#d64545' },
      ]
    default:
      return []
  }
}
