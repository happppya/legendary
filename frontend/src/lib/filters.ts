// Filter model + row vocabulary for the node index. Realm-agnostic: both the
// demo scene (lib/mockRealm) and realms opened from disk feed `NodeView`
// lists through these builders, so the same checklist rows, taxonomy trees
// and `matchesFilters` semantics drive the table, board and explorer panes.

import type { NodeView } from '../types'
import { KIND_LABEL, KINDS } from './kind'
import { STATUS_LABEL, STATUS_KEYS } from './status'
import { PRIORITY_LABEL, PRIORITY_KEYS } from './priority'

/* ---- filter model ----------------------------------------------------- */

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

/* ---- row builders ------------------------------------------------------ */

export function kindRows(nodes: readonly NodeView[]): CheckRow[] {
  return KINDS.map((k) => ({
    key: k,
    label: KIND_LABEL[k],
    count: nodes.filter((n) => n.kind === k).length,
  }))
}

export function statusRows(nodes: readonly NodeView[]): CheckRow[] {
  return STATUS_KEYS.map((s) => ({
    key: s,
    label: STATUS_LABEL[s],
    count: nodes.filter((n) => n.effectiveStatus === s).length,
  }))
}

export function priorityRows(nodes: readonly NodeView[]): CheckRow[] {
  return PRIORITY_KEYS.map((p) => ({
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
function branchRows(field: 'epics' | 'disciplines', nodes: readonly NodeView[]): CheckRow[] {
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

export function epicRows(nodes: readonly NodeView[]): CheckRow[] {
  return branchRows('epics', nodes)
}

export function disciplineRows(nodes: readonly NodeView[]): CheckRow[] {
  return branchRows('disciplines', nodes)
}

export function landmarkRows(nodes: readonly NodeView[]): CheckRow[] {
  const counts = new Map<string, number>()
  for (const n of nodes) {
    if (!n.landmark) continue
    counts.set(n.landmark, (counts.get(n.landmark) ?? 0) + 1)
  }
  return [...counts.entries()]
    .map(([key, count]) => ({ key, label: key, count }))
    .sort((a, b) => b.count - a.count || a.label.localeCompare(b.label))
}

/* ---- taxonomy trees ---------------------------------------------------- */

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
export function taxTree(field: 'epics' | 'disciplines', nodes: readonly NodeView[]): TaxRow[] {
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

/* ---- matching ---------------------------------------------------------- */

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

/** Copy `set` with `value` added/removed (never mutates the input). */
export function toggle<T>(set: Set<T>, value: T): Set<T> {
  const next = new Set(set)
  if (next.has(value)) next.delete(value)
  else next.add(value)
  return next
}
