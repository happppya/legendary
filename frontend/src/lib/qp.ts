// Single source of truth for quest-point estimation (test-feedback Bug3):
// Fibonacci amounts are the *recommended* one-click options, but any whole
// number in 0–9999 is accepted by the engine (Fibonacci is a recommendation,
// not a rule).
export const QP_CHOICES = [1, 2, 3, 5, 8, 13, 21] as const

export const QP_MAX = 9999

/** Validate a raw QP string: '' means "no estimate". */
export function parseQp(raw: string): number | null | 'invalid' {
  const s = raw.trim()
  if (s === '') return null
  const n = Number(s)
  if (!Number.isInteger(n) || n < 0 || n > QP_MAX) return 'invalid'
  return n
}
