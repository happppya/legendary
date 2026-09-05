// Generates `frontend/public/realm-demo.json` — a static snapshot of
// `examples/realm-demo` in the exact shape the desktop shell returns over
// IPC (see desktop/src/ipc.rs). Plain-browser dev/preview sessions fetch this
// fixture so the renderer is identical in web and desktop.
//
// Usage (repo root): node frontend/scripts/make-demo-fixture.mjs
// or: npm run fixture --prefix frontend
//
// Regenerate whenever the sample realm changes. The desktop shell never uses
// this file — it reads the realm live.

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const repoRoot = join(here, '..', '..')
const realmRoot = join(repoRoot, 'examples', 'realm-demo')
const outFile = join(here, '..', 'public', 'realm-demo.json')

const index = JSON.parse(readFileSync(join(realmRoot, '.legend', 'index.json'), 'utf8'))

// snake_case index.json entry → camelCase IPC NodeView, plus raw Markdown body
const RENAME = {
  effective_status: 'effectiveStatus',
  quest_points: 'questPoints',
  explicit_landmark: 'explicitLandmark',
  blocked_by: 'blockedBy',
  total_qp: 'totalQp',
  locked_qp_percent: 'lockedQpPercent',
  completed_at: 'completedAt',
  created_at: 'createdAt',
  validation_error: 'validationError',
}

const nodes = Object.entries(index.nodes).map(([id, entry]) => {
  const view = { ...entry }
  for (const [from, to] of Object.entries(RENAME)) {
    if (from in view) {
      view[to] = view[from]
      delete view[from]
    }
  }
  const file = readFileSync(join(realmRoot, view.path), 'utf8')
  const body = file.replace(/^---\n[\s\S]*?\n---\n/, '')
  view.body = body
  if (view.id !== id) throw new Error(`id mismatch for ${id}`)
  return view
})

const payload = {
  root: 'examples/realm-demo',
  generatedAt: index.generated_at,
  nodeCount: index.node_count,
  warnings: [],
  nodes,
}

mkdirSync(dirname(outFile), { recursive: true })
writeFileSync(outFile, JSON.stringify(payload, null, 2) + '\n')
console.log(`wrote ${outFile} (${nodes.length} nodes)`)
