<script setup lang="ts">
// Legendary read-only realm preview.
//
// Two panes over one source of truth: the left pane is the realm index
// (status quick-filters, full-text search, collapsible node tree), the right
// pane is the detail sheet for the selected node (breadcrumb, status/effort
// chips, grouped metadata, rendered Markdown body).
//
// Information access is keyboard-first: / or Ctrl/Cmd+K focuses the filter,
// j/k or the arrow keys move the selection, Left/Right collapse/expand,
// Escape clears filters. The same data model serves the desktop shell (live
// IPC) and the browser preview (checked-in fixture), so there is exactly one
// renderer path.
//
// Read-only prototype: everything below filters or selects; nothing writes.

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { PhArrowClockwise, PhMoon, PhSun } from '@phosphor-icons/vue'
import { inDesktop, openRealm } from './api'
import type { NodeView, RealmPayload, TreeRow } from './types'
import { KIND_RANK } from './types'
import NodeTree from './components/NodeTree.vue'
import NodeDetail from './components/NodeDetail.vue'

const DEFAULT_PATH = 'examples/realm-demo'

type StatusFilter = 'all' | 'active' | 'blocked' | 'unstarted' | 'vanquished'

const isDesktop = ref(inDesktop())
const pathInput = ref(DEFAULT_PATH)
const payload = ref<RealmPayload | null>(null)
const loading = ref(false)
const error = ref('')
const selectedId = ref<string | null>(null)
const filter = ref('')
const statusSel = ref<StatusFilter>('all')
const collapsed = ref<Set<string>>(new Set())

/* ---- theme ------------------------------------------------------------ */

const theme = ref<'dark' | 'light'>(
  document.documentElement.dataset.theme === 'light' ? 'light' : 'dark',
)

function toggleTheme() {
  theme.value = theme.value === 'light' ? 'dark' : 'light'
  document.documentElement.dataset.theme = theme.value
  try {
    localStorage.setItem('legendary-theme', theme.value)
  } catch {
    /* storage unavailable (hardened webview) - theme just won't persist */
  }
}

/* ---- realm index ------------------------------------------------------ */

const nodesById = computed(() => {
  const m = new Map<string, NodeView>()
  for (const n of payload.value?.nodes ?? []) m.set(n.id, n)
  return m
})

const childrenOf = computed(() => {
  const m = new Map<string, NodeView[]>()
  for (const n of payload.value?.nodes ?? []) {
    const p = n.parent
    if (p && nodesById.value.has(p)) {
      const arr = m.get(p) ?? []
      arr.push(n)
      m.set(p, arr)
    }
  }
  for (const arr of m.values()) arr.sort(byKindThenId)
  return m
})

function byKindThenId(a: NodeView, b: NodeView): number {
  return KIND_RANK[a.kind] - KIND_RANK[b.kind] || a.id.localeCompare(b.id)
}

const roots = computed(() =>
  (payload.value?.nodes ?? [])
    .filter((n) => !n.parent || !nodesById.value.has(n.parent))
    .sort(byKindThenId),
)

const counts = computed(() => {
  const list = payload.value?.nodes ?? []
  const c = { total: list.length, active: 0, blocked: 0, unstarted: 0, vanquished: 0 }
  for (const n of list) {
    const key = n.effectiveStatus
    if (key === 'active') c.active++
    else if (key === 'blocked') c.blocked++
    else if (key === 'unstarted') c.unstarted++
    else if (key === 'vanquished') c.vanquished++
  }
  return c
})

const realmName = computed(() => {
  const root = payload.value?.root ?? ''
  const parts = root.split(/[\\/]/).filter(Boolean)
  return parts.length ? (parts[parts.length - 1] as string) : root
})

/** Tree rows honoring collapse state and the active filter (or both). */
const rows = computed<TreeRow[]>(() => {
  const query = filter.value.trim().toLowerCase()
  const searching = query.length > 0
  const filtering = statusSel.value !== 'all'

  if (!searching && !filtering) {
    const out: TreeRow[] = []
    const walk = (node: NodeView, depth: number) => {
      const kids = childrenOf.value.get(node.id) ?? []
      out.push({ node, depth, hasChildren: kids.length > 0 })
      if (!collapsed.value.has(node.id)) {
        for (const child of kids) walk(child, depth + 1)
      }
    }
    for (const r of roots.value) walk(r, 0)
    return out
  }

  // While filtering, walk the whole tree but only expose nodes that match,
  // keeping their ancestors so a result always shows where it lives.
  const keep = new Set<string>()
  const mark = (node: NodeView): boolean => {
    const self = searching ? matches(node, query) : node.effectiveStatus === statusSel.value
    let kidMatches = false
    for (const child of childrenOf.value.get(node.id) ?? []) {
      if (mark(child)) kidMatches = true
    }
    if (self || kidMatches) keep.add(node.id)
    return self || kidMatches
  }
  for (const r of roots.value) mark(r)

  const out: TreeRow[] = []
  const walk = (node: NodeView, depth: number) => {
    const kids = childrenOf.value.get(node.id) ?? []
    out.push({ node, depth, hasChildren: kids.length > 0 })
    for (const child of kids) {
      if (keep.has(child.id)) walk(child, depth + 1)
    }
  }
  for (const r of roots.value) {
    if (keep.has(r.id)) walk(r, 0)
  }
  return out
})

function matches(n: NodeView, query: string): boolean {
  return (
    n.title.toLowerCase().includes(query) ||
    n.id.toLowerCase().includes(query) ||
    n.kind.toLowerCase().includes(query) ||
    n.effectiveStatus.toLowerCase().includes(query) ||
    n.tags.some((t) => t.toLowerCase().includes(query)) ||
    n.epics.some((e) => e.toLowerCase().includes(query)) ||
    n.disciplines.some((d) => d.toLowerCase().includes(query)) ||
    (n.landmark ?? '').toLowerCase().includes(query) ||
    (n.parent ?? '').toLowerCase().includes(query) ||
    n.blockedBy.some((b) => b.toLowerCase().includes(query))
  )
}

const selected = computed<NodeView | null>(() => {
  if (!payload.value) return null
  return selectedId.value ? (nodesById.value.get(selectedId.value) ?? null) : null
})

/* ---- selection integrity ---------------------------------------------- */

function selectionVisible(): boolean {
  const cur = selectedId.value
  if (!cur) return false
  const n = nodesById.value.get(cur)
  if (!n) return false
  if (filter.value.trim() && !matches(n, filter.value.trim().toLowerCase())) return false
  if (statusSel.value !== 'all' && n.effectiveStatus !== statusSel.value) return false
  return true
}

// When the visible set changes (filter, collapse, realm switch) and the
// current node drops out of it, follow the first visible row instead.
watch(rows, () => {
  if (!rows.value.length) {
    selectedId.value = null
    return
  }
  if (!selectionVisible()) {
    selectedId.value = rows.value[0].node.id
  }
})

function toggle(id: string) {
  const next = new Set(collapsed.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  collapsed.value = next
}

/** Jump to a node; drop the filters only if that node is hidden by them. */
function goTo(id: string) {
  const n = nodesById.value.get(id)
  if (!n) return
  selectedId.value = id
  const visible = rows.value.some((r) => r.node.id === id)
  if (!visible) {
    filter.value = ''
    statusSel.value = 'all'
    collapsed.value = new Set()
  }
}

function applyTerm(term: string) {
  filter.value = term
  statusSel.value = 'all'
  treeRef.value?.focusSearch()
}

/* ---- keyboard navigation ---------------------------------------------- */

const treeRef = ref<InstanceType<typeof NodeTree> | null>(null)

function moveSelection(delta: number) {
  const list = rows.value
  if (!list.length) return
  const cur = selectedId.value
  const idx = cur ? list.findIndex((r) => r.node.id === cur) : -1
  if (idx === -1) {
    selectedId.value = list[0].node.id
    return
  }
  const next = (idx + delta + list.length) % list.length
  selectedId.value = list[next].node.id
}

function selectAncestorOf(current: TreeRow): boolean {
  const list = rows.value
  const idx = list.findIndex((r) => r.node.id === current.node.id)
  for (let i = idx - 1; i >= 0; i--) {
    if (list[i].depth < current.depth) {
      selectedId.value = list[i].node.id
      return true
    }
  }
  return false
}

function clearFilters() {
  filter.value = ''
  statusSel.value = 'all'
}

function onKey(e: KeyboardEvent) {
  if (!payload.value) return
  const el = e.target instanceof HTMLElement ? e.target : null
  const inSearch = el?.classList.contains('realm-filter') ?? false
  const typing = el?.closest('input, textarea, [contenteditable="true"]') != null
  const mod = e.ctrlKey || e.metaKey

  // Filter focus: "/" anywhere outside text fields, Cmd/Ctrl+K everywhere.
  if ((e.key === '/' && !typing) || (mod && e.key.toLowerCase() === 'k')) {
    e.preventDefault()
    treeRef.value?.focusSearch()
    return
  }
  if (typing) {
    if (inSearch && e.key === 'Escape' && filter.value) {
      filter.value = ''
      e.preventDefault()
    } else if (inSearch && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
      e.preventDefault()
      moveSelection(e.key === 'ArrowDown' ? 1 : -1)
    }
    return
  }
  if (e.altKey) return

  switch (e.key) {
    case 'ArrowDown':
    case 'j':
      e.preventDefault()
      moveSelection(1)
      break
    case 'ArrowUp':
    case 'k':
      e.preventDefault()
      moveSelection(-1)
      break
    case 'ArrowRight':
    case 'l': {
      e.preventDefault()
      const row = rows.value.find((r) => r.node.id === selectedId.value)
      if (!row) break
      if (row.hasChildren && collapsed.value.has(row.node.id)) {
        const next = new Set(collapsed.value)
        next.delete(row.node.id)
        collapsed.value = next
      } else if (row.hasChildren) {
        moveSelection(1) // first child
      }
      break
    }
    case 'ArrowLeft':
    case 'h': {
      e.preventDefault()
      const row = rows.value.find((r) => r.node.id === selectedId.value)
      if (!row) break
      if (row.hasChildren && !collapsed.value.has(row.node.id)) {
        const next = new Set(collapsed.value)
        next.add(row.node.id)
        collapsed.value = next
      } else if (!selectAncestorOf(row) && row.depth === 0) {
        // at the root with no ancestor - collapse nothing, stay put
      }
      break
    }
    case 'Escape':
      if (filter.value || statusSel.value !== 'all') {
        clearFilters()
        e.preventDefault()
      }
      break
  }
}

/* ---- opening the realm ------------------------------------------------ */

async function open() {
  loading.value = true
  error.value = ''
  try {
    const data = await openRealm(pathInput.value.trim() || DEFAULT_PATH)
    payload.value = data
    selectedId.value = data.nodes[0]?.id ?? null
    collapsed.value = new Set()
    filter.value = ''
    statusSel.value = 'all'
  } catch (err) {
    payload.value = null
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKey)
  void open()
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
})
</script>

<template>
  <div class="shell">
    <header class="topbar">
      <div class="brand">
        <span class="mark" aria-hidden="true">
          <i class="n1"></i><i class="n2"></i><i class="n3"></i>
        </span>
        <span class="wordmark">Legendary</span>
        <span class="mode" :class="{ desktop: isDesktop }">
          <span class="mode-dot" aria-hidden="true"></span>
          {{ isDesktop ? 'desktop / live IPC' : 'web / demo fixture' }}
        </span>
      </div>

      <div class="tools">
        <form class="opener" role="search" @submit.prevent="open">
          <input
            v-model="pathInput"
            type="text"
            spellcheck="false"
            aria-label="Realm path"
            placeholder="realm path, e.g. examples/realm-demo"
            :disabled="loading || !isDesktop"
            :title="isDesktop ? 'Path to the realm directory' : 'The browser preview always reads the checked-in snapshot'"
          />
          <button type="submit" class="primary" :disabled="loading || !isDesktop">
            {{ loading ? 'Opening' : 'Open realm' }}
          </button>
        </form>

        <button
          type="button"
          class="icon-btn"
          :disabled="loading || !payload"
          title="Re-read the realm from disk"
          aria-label="Re-read the realm"
          @click="open"
        >
          <PhArrowClockwise :size="15" aria-hidden="true" />
        </button>

        <button
          type="button"
          class="icon-btn"
          :title="theme === 'light' ? 'Switch to night theme' : 'Switch to map theme'"
          :aria-label="theme === 'light' ? 'Switch to night theme' : 'Switch to map theme'"
          @click="toggleTheme"
        >
          <PhMoon v-if="theme === 'light'" :size="15" aria-hidden="true" />
          <PhSun v-else :size="15" aria-hidden="true" />
        </button>
      </div>
    </header>

    <div v-if="error && !payload" class="banner-row">
      <div class="banner error" role="alert">
        <span class="banner-title">Could not open the realm</span>
        <span class="banner-text">{{ error }}</span>
      </div>
    </div>

    <div v-if="payload && !isDesktop" class="note-strip">
      Web preview reads the checked-in <code>realm-demo.json</code> snapshot; the desktop shell
      reads the realm live over Tauri IPC.
    </div>

    <main v-if="payload" class="panes">
      <NodeTree
        ref="treeRef"
        :rows="rows"
        :selected-id="selectedId"
        :filter="filter"
        :status="statusSel"
        :collapsed="collapsed"
        :counts="counts"
        :realm-name="realmName"
        :realm-path="payload.root"
        :warnings="payload.warnings"
        @select="goTo"
        @toggle="toggle"
        @update:filter="filter = $event"
        @update:status="statusSel = $event"
      />
      <NodeDetail
        :node="selected"
        :nodes-by-id="nodesById"
        :realm-name="realmName"
        @select="goTo"
        @filter-by="applyTerm"
      />
    </main>

    <main v-else class="idle">
      <div v-if="loading" class="idle-box">
        <span class="loader" aria-hidden="true"><i></i><i></i><i></i></span>
        <p>Reading realm&hellip;</p>
      </div>
      <div v-else-if="error" class="idle-box">
        <p class="idle-title">Nothing open</p>
        <p class="idle-error">{{ error }}</p>
        <button type="button" class="primary" @click="open">Try again</button>
      </div>
      <div v-else class="idle-box">
        <p>No realm open.</p>
      </div>
    </main>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;
  min-height: 0;
}

/* ---- top bar ---------------------------------------------------------- */

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 0 12px;
  height: 50px;
  flex: none;
  background: var(--header-bg);
  border-bottom: 1px solid var(--header-line);
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.mark {
  position: relative;
  width: 26px;
  height: 26px;
  flex: none;
  border-radius: var(--r-m);
  background: var(--bg-2);
  border: 1px solid var(--line-1);
}

.mark i {
  position: absolute;
  border-radius: 50%;
}

.mark .n1 {
  width: 7px;
  height: 7px;
  left: 4px;
  top: 4px;
  background: var(--gold);
}

.mark .n2 {
  width: 6px;
  height: 6px;
  right: 5px;
  bottom: 6px;
  background: var(--kind-action);
}

.mark .n3 {
  width: 4px;
  height: 4px;
  right: 6px;
  top: 9px;
  background: var(--kind-guard);
}

.wordmark {
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--text-1);
}

.mode {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  letter-spacing: 0.09em;
  text-transform: uppercase;
  color: var(--text-3);
  border: 1px solid var(--line-1);
  border-radius: 999px;
  padding: 2px 9px;
  white-space: nowrap;
}

.mode-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--dot-unstarted);
}

.mode.desktop .mode-dot {
  background: var(--dot-active);
}

.tools {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.opener {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.opener input {
  width: clamp(200px, 30vw, 380px);
  height: 30px;
  background: var(--inset);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  color: var(--text-1);
  padding: 0 10px;
  font-size: 12px;
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
  outline: none;
  transition: border-color 0.15s ease;
}

.opener input:focus {
  border-color: var(--gold);
}

.opener input::placeholder {
  color: var(--faint);
}

.opener input:disabled {
  opacity: 0.75;
}

button.primary {
  height: 30px;
  flex: none;
  background: var(--btn-primary-bg);
  border: 1px solid var(--btn-primary-bg);
  color: var(--btn-primary-fg);
  border-radius: var(--r-m);
  padding: 0 14px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.01em;
  transition: background-color 0.15s ease, border-color 0.15s ease;
}

button.primary:hover:not(:disabled) {
  background: var(--btn-primary-hover);
  border-color: var(--btn-primary-hover);
}

button.primary:disabled {
  opacity: 0.55;
}

.icon-btn {
  width: 30px;
  height: 30px;
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--r-m);
  color: var(--text-3);
  border: 1px solid var(--line-1);
  background: transparent;
  transition: color 0.15s ease, background-color 0.15s ease, border-color 0.15s ease;
}

.icon-btn:hover:not(:disabled) {
  color: var(--text-1);
  background: var(--bg-2);
  border-color: var(--line-2);
}

.icon-btn:disabled {
  opacity: 0.5;
}

/* ---- banners ---------------------------------------------------------- */

.banner-row {
  padding: 10px 14px 0;
  flex: none;
}

.banner {
  display: flex;
  flex-direction: column;
  gap: 2px;
  border-radius: var(--r-l);
  border: 1px solid var(--err-line);
  background: var(--err-bg);
  color: var(--err-fg);
  padding: 10px 14px;
  font-size: 12.5px;
}

.banner-title {
  font-weight: 600;
}

.banner-text {
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.note-strip {
  flex: none;
  display: flex;
  gap: 6px;
  align-items: baseline;
  padding: 6px 14px;
  font-size: 11.5px;
  color: var(--note-fg);
  background: var(--note-bg);
  border-bottom: 1px solid var(--note-line);
}

.note-strip code {
  background: var(--inset);
  border: 1px solid var(--line-1);
  border-radius: var(--r-s);
  padding: 0 4px;
}

/* ---- panes ------------------------------------------------------------ */

.panes {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 356px minmax(0, 1fr);
}

/* ---- idle state ------------------------------------------------------- */

.idle {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.idle-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-3);
}

.idle-box p {
  margin: 0;
  font-size: 13px;
}

.idle-title {
  font-weight: 600;
  color: var(--text-2);
  font-size: 15px !important;
}

.idle-error {
  color: var(--err-fg);
  max-width: 560px;
  text-align: center;
  overflow-wrap: anywhere;
}

.loader {
  display: inline-flex;
  gap: 5px;
}

.loader i {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--gold);
  animation: pulse 1.1s ease-in-out infinite;
}

.loader i:nth-child(2) {
  animation-delay: 0.18s;
}

.loader i:nth-child(3) {
  animation-delay: 0.36s;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.25;
    transform: translateY(0);
  }
  50% {
    opacity: 1;
    transform: translateY(-2px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .loader i {
    animation: none;
  }
}

/* ---- narrow layouts --------------------------------------------------- */

@media (max-width: 900px) {
  .topbar {
    flex-wrap: wrap;
    height: auto;
    padding: 8px 12px;
    row-gap: 8px;
  }

  .tools {
    order: 3;
    flex: 1 1 100%;
  }

  .opener {
    flex: 1;
  }

  .opener input {
    flex: 1;
    width: auto;
  }

  .panes {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(200px, 42vh) minmax(0, 1fr);
  }
}
</style>
