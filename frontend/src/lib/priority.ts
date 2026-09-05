// Priority vocabulary for nodes (doc 04). Single source of truth for the
// label + ordering used by the table, chips and filter rows.

export const PRIORITY_KEYS = ['critical', 'high', 'medium', 'low'] as const

export const PRIORITY_LABEL: Record<string, string> = {
  critical: 'Critical',
  high: 'High',
  medium: 'Medium',
  low: 'Low',
}

/** Sort rank: most to least urgent. */
export const PRIORITY_RANK: Record<string, number> = {
  critical: 0,
  high: 1,
  medium: 2,
  low: 3,
}

/** Ordered { key, label } rows for filter rows / menu lists. */
export const PRIORITIES: readonly { key: string; label: string }[] = PRIORITY_KEYS.map((k) => ({
  key: k,
  label: PRIORITY_LABEL[k],
}))
