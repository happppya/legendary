// Mirror of the desktop IPC payloads (desktop/src/ipc.rs). The `.legend/
// index.json` snapshot and the generated demo fixture use the same shape, so
// the renderer is agnostic to whether data came over Tauri commands or a
// static snapshot.

export interface NodeView {
  id: string
  kind: 'card' | 'action' | 'guard' | 'idea'
  title: string
  status: 'unstarted' | 'active' | 'vanquished'
  /** Computed at runtime: unstarted | active | vanquished | blocked */
  effectiveStatus: string
  blocked: boolean
  priority: 'critical' | 'high' | 'medium' | 'low'
  disciplines: string[]
  epics: string[]
  questPoints: number | null
  landmark: string | null
  explicitLandmark: string | null
  parent: string | null
  blockedBy: string[]
  totalQp: number
  lockedQpPercent: number | null
  completedAt: string | null
  createdAt: string | null
  tags: string[]
  path: string
  validationError: string | null
  body: string
}

export interface RealmPayload {
  root: string
  generatedAt: string
  nodeCount: number
  warnings: string[]
  nodes: NodeView[]
}

export interface TreeRow {
  node: NodeView
  depth: number
  hasChildren: boolean
}
