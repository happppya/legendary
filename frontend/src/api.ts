// Data layer: desktop shell talks to the engine over Tauri IPC commands
// (live realm); a plain-browser dev session (no `__TAURI_INTERNALS__`)
// falls back to the checked-in realm-demo fixture so the exact same UI can
// be previewed in a normal browser tab.

import { invoke } from '@tauri-apps/api/core'
import type { MutationPayload, RealmPayload, SyncPayload } from './types'

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown
  }
}

/** True when running inside the Tauri desktop shell (dev or bundled). */
export function inDesktop(): boolean {
  return '__TAURI_INTERNALS__' in window
}

/** Open (or reload) a realm by path. In the browser, loads the demo fixture. */
export async function openRealm(path: string): Promise<RealmPayload> {
  if (inDesktop()) {
    return invoke<RealmPayload>('realm_open', { path })
  }
  const res = await fetch('realm-demo.json')
  if (!res.ok) {
    throw new Error(`demo fixture realm-demo.json unavailable (HTTP ${res.status})`)
  }
  return (await res.json()) as RealmPayload
}

/**
 * Fetch one node's raw Markdown body. Not used by the current preview UI
 * (list payloads already embed bodies); kept for the upcoming workspace view
 * where bodies stream on selection.
 */
export async function readNodeBody(id: string): Promise<string> {
  if (!inDesktop()) {
    throw new Error('node_body is only available in the desktop shell')
  }
  return invoke<string>('node_body', { id })
}

/* ---- mutations (desktop only; shared legend_core::ops semantics) -------- */

function requireDesktop(op: string) {
  if (!inDesktop()) {
    throw new Error(`${op} requires the desktop shell (realm writes are disabled in the browser preview)`)
  }
}

/** Create a node (`legend create`). */
export async function createNode(args: {
  kind: string
  title: string
  parent?: string | null
  priority?: string
  questPoints?: number | null
}): Promise<MutationPayload> {
  requireDesktop('createNode')
  return invoke<MutationPayload>('node_create', {
    kind: args.kind,
    title: args.title,
    parent: args.parent ?? null,
    priority: args.priority ?? null,
    questPoints: args.questPoints ?? null,
  })
}

/** Update a node's stored lifecycle status (`legend update --status`). */
export async function updateNodeStatus(
  id: string,
  status: 'unstarted' | 'active' | 'vanquished',
  cascade = false,
): Promise<MutationPayload> {
  requireDesktop('updateNodeStatus')
  return invoke<MutationPayload>('node_update_status', { id, status, cascade })
}

/** Rename a node: new title + re-derived file slug (spec §3.3). */
export async function renameNode(id: string, title: string): Promise<MutationPayload> {
  requireDesktop('renameNode')
  return invoke<MutationPayload>('node_rename', { id, title })
}

/** Overwrite a node's unmanaged Markdown body (frontmatter preserved). */
export async function setNodeBody(id: string, body: string): Promise<MutationPayload> {
  requireDesktop('setNodeBody')
  return invoke<MutationPayload>('node_set_body', { id, body })
}

/** Edit ECS components in one write (doc 05 §5.1 metadata panel).
 * Nullable components (questPoints, landmark, parent) use explicit clear
 * flags because JSON cannot express "clear" vs "untouched". */
export async function updateNodeComponents(args: {
  id: string
  priority?: string
  questPoints?: number
  clearQuestPoints?: boolean
  disciplines?: string[]
  epics?: string[]
  landmark?: string
  clearLandmark?: boolean
  tags?: string[]
  blockedBy?: string[]
  parent?: string
  clearParent?: boolean
}): Promise<MutationPayload> {
  requireDesktop('updateNodeComponents')
  return invoke<MutationPayload>('node_update_components', {
    id: args.id,
    priority: args.priority ?? null,
    questPoints: args.questPoints ?? null,
    clearQuestPoints: args.clearQuestPoints ?? false,
    disciplines: args.disciplines ?? null,
    epics: args.epics ?? null,
    landmark: args.landmark ?? null,
    clearLandmark: args.clearLandmark ?? false,
    tags: args.tags ?? null,
    blockedBy: args.blockedBy ?? null,
    parent: args.parent ?? null,
    clearParent: args.clearParent ?? false,
  })
}

/** Wrap a node in a new Card group ("Promote to Card", spec §4.1). */
export async function wrapNode(id: string, title?: string): Promise<MutationPayload> {
  requireDesktop('wrapNode')
  return invoke<MutationPayload>('node_wrap', { id, title: title ?? null })
}

/** Re-parent a node (cycle-checked) or promote it to root. */
export async function reparentNode(id: string, parent: string | null): Promise<MutationPayload> {
  requireDesktop('reparentNode')
  return invoke<MutationPayload>('node_reparent', { id, parent })
}

/** Delete a node or its whole subgraph (spec §4.5). */
export async function deleteNode(id: string, recursive = false): Promise<MutationPayload> {
  requireDesktop('deleteNode')
  return invoke<MutationPayload>('node_delete', { id, recursive })
}

/** Rebuild indexes after external edits (`legend index sync`). */
export async function syncIndex(): Promise<SyncPayload> {
  if (inDesktop()) {
    return invoke<SyncPayload>('index_sync')
  }
  // Browser preview: the fixture is static, so "sync" is a plain re-read.
  const realm = await openRealm('realm-demo.json')
  return { realm, issues: [] }
}

/** Subscribe to debounced external-change events from the desktop watcher. */
export async function onRealmChanged(handler: () => void): Promise<() => void> {
  if (!inDesktop()) return () => {}
  const { listen } = await import('@tauri-apps/api/event')
  const unlisten = await listen<null>('realm://changed', handler)
  return unlisten
}
