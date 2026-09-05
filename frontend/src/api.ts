// Data layer: desktop shell talks to the engine over Tauri IPC commands
// (live realm); a plain-browser dev session (no `__TAURI_INTERNALS__`)
// falls back to the checked-in realm-demo fixture so the exact same UI can
// be previewed in a normal browser tab.

import { invoke } from '@tauri-apps/api/core'
import type { RealmPayload } from './types'

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
