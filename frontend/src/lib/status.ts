// Status vocabulary for nodes (doc 03). `unstarted | active | vanquished`
// are authored; `blocked` is computed at runtime (never written to
// frontmatter). Single source of truth for the label + ordering used by the
// table, board, chips, pills and filter rows.

/** Effective statuses in canonical display order. */
export const STATUS_KEYS = ['unstarted', 'active', 'blocked', 'vanquished'] as const

export const STATUS_LABEL: Record<string, string> = {
  unstarted: 'Unstarted',
  active: 'Active',
  blocked: 'Blocked',
  vanquished: 'Vanquished',
}

/** Sort rank: unstarted → active → blocked → vanquished. */
export const STATUS_RANK: Record<string, number> = {
  unstarted: 0,
  active: 1,
  blocked: 2,
  vanquished: 3,
}

/** Ordered { key, label } rows for column builders / board columns. */
export const STATUSES: readonly { key: string; label: string }[] = STATUS_KEYS.map((k) => ({
  key: k,
  label: STATUS_LABEL[k],
}))
