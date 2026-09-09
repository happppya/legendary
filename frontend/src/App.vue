<script setup lang="ts">
// Legendary workspace shell.
//
// The top bar is an app-style menu bar (File owns realms, View owns theme),
// and the body is the reference four-panel workspace: explorer + filters ·
// graph/table split · detail inspector. All view switching lives in the
// explorer's Views list (the leftmost icon dock was merged into it). The
// default realm is the mock scene in lib/mockRealm.ts; opening a real realm
// (File → Open…) swaps every pane onto the real payload while the graph
// canvas shows its "later milestone" placeholder.
//
// Components live in folders by role (shell chrome, workspace panes, full
// pages, overlays) - see components/. This file owns cross-pane state and
// routing only: realm data, selection, filters, navigation, menus, palette
// and dialogs.

import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { PhFunnel } from '@phosphor-icons/vue'
import type { MenuItem, MenuSpec } from './components/shell/MenuBar.vue'
import TopBar from './components/shell/TopBar.vue'
import StatusBar from './components/shell/StatusBar.vue'
import ExplorerPanel from './components/workspace/ExplorerPanel.vue'
import FiltersPanel from './components/workspace/FiltersPanel.vue'
import WorkspaceCenter from './components/workspace/WorkspaceCenter.vue'
import NodeDetail from './components/workspace/detail/NodeDetail.vue'
import BoardView from './components/pages/BoardView.vue'
import CalendarView from './components/pages/CalendarView.vue'
import EntitiesView from './components/pages/EntitiesView.vue'
import LandmarkMatrix from './components/pages/LandmarkMatrix.vue'
import SettingsView from './components/pages/SettingsView.vue'
import CommandPalette, { type PaletteCommand } from './components/overlays/CommandPalette.vue'
import AboutDialog from './components/overlays/AboutDialog.vue'
import OpenRealmDialog from './components/overlays/OpenRealmDialog.vue'
import NewRealmDialog from './components/overlays/NewRealmDialog.vue'
import MutationDialog, { type MutationRequest } from './components/overlays/MutationDialog.vue'
import MoveToDialog from './components/overlays/MoveToDialog.vue'
import NodeCreateDialog from './components/overlays/NodeCreateDialog.vue'
import {
  createNode,
  deleteNode,
  inDesktop,
  onRealmChanged,
  openRealm,
  renameNode,
  reparentNode,
  setNodeBody,
  updateNodeComponents,
  updateNodeStatus,
  wrapNode,
} from './api'
import type { MetaEdits } from './components/workspace/detail/NodeMetaEditor.vue'
import { isContainerKind } from './lib/kind'
import type { MutationPayload as MutationPayloadLike, NodeView, RealmPayload } from './types'
import type { CheckRow, FilterModel, TaxRow } from './lib/filters'
import {
  disciplineRows,
  emptyFilters,
  epicRows,
  kindRows,
  landmarkRows,
  matchesFilters,
  priorityRows,
  statusRows,
  taxTree,
} from './lib/filters'
import { linkCount, type CompletionLevel } from './lib/node'
import type { OverlayMode } from './lib/overlay'
import type { GraphScope } from './components/workspace/graph/scene'
import { MOCK_BY_KEY, MOCK_NODES, MOCK_REALM } from './lib/mockRealm'

type Page = 'workspace' | 'board' | 'entities' | 'calendar' | 'matrix' | 'settings'

/* ---- theme ------------------------------------------------------------- */

const theme = ref<'dark' | 'light'>(
  document.documentElement.dataset.theme === 'light' ? 'light' : 'dark',
)

function setTheme(t: 'dark' | 'light') {
  theme.value = t
  document.documentElement.dataset.theme = t
  try {
    localStorage.setItem('legendary-theme', t)
  } catch {
    /* storage unavailable - no persistence */
  }
}

/* ---- realm data --------------------------------------------------------- */

type Source = 'mock' | 'realm'
const source = ref<Source>('mock')
const realmPath = ref(MOCK_REALM.path)
const nodes = ref<NodeView[]>(MOCK_NODES)
const error = ref('')
const isDesktop = ref(inDesktop())

const nodesById = computed(() => {
  const m = new Map<string, NodeView>()
  for (const n of nodes.value) m.set(n.id, n)
  return m
})

const realmName = computed(() => {
  if (source.value === 'mock') return MOCK_REALM.name
  const parts = realmPath.value.split(/[\\/]/).filter(Boolean)
  return parts.length ? (parts[parts.length - 1] as string) : realmPath.value
})

const totals = computed(() => ({ nodes: nodes.value.length, links: linkCount(nodes.value) }))

const isMockScene = computed(() => source.value === 'mock')
const canMutateAny = computed(() => isDesktop.value && source.value === 'realm')

/* ---- selection ------------------------------------------------------------ */

const selectedId = ref<string | null>(MOCK_BY_KEY.get('cmc')?.id ?? MOCK_NODES[0]?.id ?? null)

watch(nodes, (list) => {
  if (!selectedId.value || !list.some((n) => n.id === selectedId.value)) {
    selectedId.value = list[0]?.id ?? null
  }
})

const selected = computed<NodeView | null>(() =>
  selectedId.value ? (nodesById.value.get(selectedId.value) ?? null) : null,
)

function select(id: string) {
  if (nodesById.value.has(id)) selectedId.value = id
}

/* ---- filters ---------------------------------------------------------------- */

const filters = reactive(emptyFilters()) as FilterModel
// On narrow windows the filters pane collapses so the workspace stays wide.
const filtersOpen = ref(typeof window !== 'undefined' && window.innerWidth >= 1200)
const focusSignal = ref(0)

function toggleIn(set: Set<string>, key: string) {
  if (set.has(key)) set.delete(key)
  else set.add(key)
}

function toggleFilter(cat: string, key: string) {
  if (cat === 'search') return
  const record = filters as unknown as Record<string, Set<string>>
  const set = record[cat]
  if (set) toggleIn(set, key)
}

function clearFilters() {
  filters.search = ''
  for (const cat of ['kinds', 'statuses', 'priorities', 'epics', 'disciplines', 'landmarks'] as const) {
    ;(filters[cat] as Set<string>).clear()
  }
}

function focusFilters() {
  filtersOpen.value = true
  focusSignal.value += 1
}

/** Nodes matching the filter model (drives the table + board). */
const filteredNodes = computed(() => nodes.value.filter((n) => matchesFilters(n, filters)))

/* ---- row vocabularies ----------------------------------------------------------- */

const kindRowsC = computed<CheckRow[]>(() => kindRows(nodes.value))
const statusRowsC = computed<CheckRow[]>(() => statusRows(nodes.value))
const priorityRowsC = computed<CheckRow[]>(() => priorityRows(nodes.value))
const epicRowsC = computed<CheckRow[]>(() => epicRows(nodes.value))
const disciplineRowsC = computed<CheckRow[]>(() => disciplineRows(nodes.value))
const landmarkRowsC = computed<CheckRow[]>(() => landmarkRows(nodes.value))

/* ---- editor vocabulary (metadata editor pickers) ------------------------- */

const disciplineOptionsC = computed(() =>
  (source.value === 'realm' ? taxTree('disciplines', nodes.value) : disciplineRowsC.value).map(
    (r) => ({ key: r.key, label: r.label }),
  ),
)
const epicOptionsC = computed(() =>
  (source.value === 'realm' ? taxTree('epics', nodes.value) : epicRowsC.value).map((r) => ({
    key: r.key,
    label: r.label,
  })),
)
const landmarkOptionsC = computed(() =>
  landmarkRowsC.value.map((r) => ({ key: r.key, label: r.label })),
)
// Taxonomy *trees* (.legend/taxonomy.yaml vocabulary) exist only for real
// realms opened from disk; the mock realm keeps its flat branch rows.
const epicTreeC = computed<TaxRow[] | null>(() =>
  source.value === 'realm' ? taxTree('epics', nodes.value) : null,
)
const disciplineTreeC = computed<TaxRow[] | null>(() =>
  source.value === 'realm' ? taxTree('disciplines', nodes.value) : null,
)

/* ---- pages -------------------------------------------------------------- */

const page = ref<Page>('workspace')
/** Workspace split state (test-feedback A-4): which panes are open is
 * remembered across page switches, so leaving to Board and coming back
 * restores exactly what the user had open. */
const showGraph = ref(true)
const showTable = ref(true)

/* ---- graph overlays + scope (doc 05 §5.3/§5.5, test-feedback A-1) -------- */

const overlayMode = ref<OverlayMode>('none')
/** Graph scope: local neighbourhood, whole realm, or genre overview. */
const graphScope = ref<GraphScope>('local')
const completionLevel = ref<CompletionLevel>(2)
/** Last view chosen in the explorer list (workspace highlights as one row). */
const navChoice = ref<'workspace' | 'graph' | 'table'>('graph')

const explorerActive = computed(() =>
  page.value === 'workspace' ? navChoice.value : page.value,
)

function openView(id: string) {
  switch (id) {
    // graph/table are tab-level views over the same workspace page: open
    // the missing pane but never close its sibling (A-4 — closing is only
    // ever explicit via a tab's ×), then stay on the workspace page.
    case 'workspace':
    case 'graph':
    case 'table': {
      if (id === 'workspace') {
        showGraph.value = true
        showTable.value = true
      } else if (id === 'graph' && !showGraph.value) {
        showGraph.value = true
      } else if (id === 'table' && !showTable.value) {
        showTable.value = true
      }
      navChoice.value = id as 'workspace' | 'graph' | 'table'
      page.value = 'workspace'
      break
    }
    case 'board':
      page.value = 'board'
      break
    case 'matrix':
      page.value = 'matrix'
      break
    case 'calendar':
      page.value = 'calendar'
      break
    case 'entities':
      page.value = 'entities'
      break
    case 'settings':
      page.value = 'settings'
      break
  }
}

/** Kind / taxonomy / tag filters should always land somewhere visible. */
function toWorkspace() {
  page.value = 'workspace'
  if (!showGraph.value && !showTable.value) {
    showGraph.value = true
    showTable.value = true
  }
}

function onToggleKind(kind: string) {
  toggleFilter('kinds', kind)
  if (page.value !== 'workspace') toWorkspace()
}

function onToggleEpic(label: string) {
  toggleFilter('epics', label)
}

function onToggleDiscipline(label: string) {
  toggleFilter('disciplines', label)
}

function onToggleLandmark(label: string) {
  toggleFilter('landmarks', label)
}

function browseKind(kind: string) {
  filters.kinds = new Set([kind])
  filters.statuses.clear()
  toWorkspace()
}

function onDetailFilter(f: { field: 'discipline' | 'epic' | 'tag' | 'landmark'; value: string }) {
  switch (f.field) {
    case 'discipline':
      toggleFilter('disciplines', f.value)
      break
    case 'epic':
      toggleFilter('epics', f.value)
      break
    case 'landmark':
      toggleFilter('landmarks', f.value)
      break
    case 'tag':
      filters.search = f.value
      break
  }
  if (page.value !== 'workspace') toWorkspace()
}

/* ---- realms ------------------------------------------------------------ */

interface RecentRealm {
  label: string
  path: string
}

const RECENTS_KEY = 'legendary-recent-realms'
const recentRealms = ref<RecentRealm[]>(loadRecents())

function loadRecents(): RecentRealm[] {
  try {
    const raw = localStorage.getItem(RECENTS_KEY)
    if (raw) {
      const parsed = JSON.parse(raw) as RecentRealm[]
      if (Array.isArray(parsed)) return parsed.slice(0, 8)
    }
  } catch {
    /* ignore corrupted storage */
  }
  return []
}

function rememberRecent(label: string, path: string) {
  const list = recentRealms.value.filter((r) => r.path !== path)
  list.unshift({ label, path })
  recentRealms.value = list.slice(0, 8)
  try {
    localStorage.setItem(RECENTS_KEY, JSON.stringify(recentRealms.value))
  } catch {
    /* ignore */
  }
}

function useMockRealm() {
  source.value = 'mock'
  realmPath.value = MOCK_REALM.path
  nodes.value = MOCK_NODES
  clearFilters()
  selectedId.value = MOCK_BY_KEY.get('cmc')?.id ?? null
  error.value = ''
  mutationError.value = ''
}

/** Swap in a fresh snapshot from any mutation response, preserving the
 * selection when the node survived the write. */
function applySnapshot(realm: RealmPayload) {
  source.value = 'realm'
  realmPath.value = realm.root
  nodes.value = realm.nodes
  if (!selectedId.value || !realm.nodes.some((n) => n.id === selectedId.value)) {
    selectedId.value = realm.nodes[0]?.id ?? null
  }
}

async function openRealRealm(path: string): Promise<boolean> {
  try {
    const payload = await openRealm(path)
    source.value = 'realm'
    realmPath.value = payload.root
    nodes.value = payload.nodes
    clearFilters()
    selectedId.value = payload.nodes[0]?.id ?? null
    error.value = ''
    rememberRecent(realmName.value, payload.root)
    toWorkspace()
    return true
  } catch (err) {
    error.value = String(err)
    return false
  }
}

async function reloadRealm() {
  if (source.value === 'mock') return
  await openRealRealm(realmPath.value)
}

/* ---- external edits (desktop file watcher, spec §4.2) -------------------- */

let unlistenRealmChanged: (() => void) | null = null

onMounted(async () => {
  unlistenRealmChanged = await onRealmChanged(() => {
    if (source.value === 'realm') {
      // Debounced upstream; keep selection when the node still exists.
      void reloadRealm()
    }
  })
})

onBeforeUnmount(() => {
  unlistenRealmChanged?.()
  unlistenRealmChanged = null
})

/* ---- dialogs ------------------------------------------------------------- */

type Dialog = 'open' | 'new' | 'about' | null
const dialog = ref<Dialog>(null)

/* ---- mutations (doc 04 §4.3/§4.5 UI actions) ------------------------------ */

/** Pending confirmation dialog (vanquish cascade / recursive delete). */
const mutationRequest = ref<MutationRequest | null>(null)
const mutationBusy = ref(false)
/** Last engine rejection, surfaced in the status bar like CLI stderr. */
const mutationError = ref('')

function descendantsOf(id: string): NodeView[] {
  const out: NodeView[] = []
  const stack = [id]
  while (stack.length) {
    const cur = stack.pop()
    for (const n of nodes.value) {
      if (n.parent === cur) {
        out.push(n)
        stack.push(n.id)
      }
    }
  }
  return out
}

/** Unvanquished leaf tasks under a Card (engine's cascade count). */
function unvanquishedLeaves(card: NodeView): number {
  return descendantsOf(card.id).filter(
    (n) => !isContainerKind(n.kind) && n.effectiveStatus !== 'vanquished',
  ).length
}

/** Vanquish entry point: a Card with unvanquished leaves opens the prompt
 * (doc 04 §4.3); anything else vanquishes right away. */
function requestVanquish(node: NodeView) {
  mutationError.value = ''
  if (isContainerKind(node.kind) && unvanquishedLeaves(node) > 0) {
    mutationRequest.value = { flavour: 'vanquish', node, count: unvanquishedLeaves(node) }
    return
  }
  void runMutation(() => updateNodeStatus(node.id, 'vanquished'))
}

/** Delete entry point: a container with children opens the doc 04 §4.5
 * warning modal; leaves delete immediately (still reversible via Git). */
function requestDelete(node: NodeView) {
  mutationError.value = ''
  const kids = descendantsOf(node.id)
  if (kids.length > 0) {
    mutationRequest.value = {
      flavour: 'delete',
      node,
      count: kids.length + 1,
      notes: [`+ ${kids.length} descendant ${kids.length === 1 ? 'node' : 'nodes'} in the subgraph`],
    }
    return
  }
  void runMutation(() => deleteNode(node.id, false))
}

async function runMutation(fn: () => Promise<MutationPayloadLike>): Promise<void> {
  mutationBusy.value = true
  try {
    const out = await fn()
    applySnapshot(out.realm)
    mutationError.value = ''
  } catch (err) {
    mutationError.value = String(err)
  } finally {
    mutationBusy.value = false
  }
}

function confirmMutation() {
  const req = mutationRequest.value
  if (!req) return
  mutationRequest.value = null
  if (req.flavour === 'vanquish') {
    void runMutation(() => updateNodeStatus(req.node.id, 'vanquished', true))
  } else {
    void runMutation(() => deleteNode(req.node.id, true))
  }
}

function onWrap() {
  const n = selected.value
  if (n) void runMutation(() => wrapNode(n.id))
}

function onReparentRoot() {
  const n = selected.value
  if (n) void runMutation(() => reparentNode(n.id, null))
}

/* ---- create node (test-feedback A-2) ------------------------------------- */

/** Open when non-null; presets flow in from whichever view raised the +. */
const createRequest = ref<{ kind: string | null; parent: string | null; status: string | null } | null>(null)

function requestCreate(preset: { kind?: string | null; parent?: string | null; status?: string | null } = {}) {
  mutationError.value = ''
  createRequest.value = {
    kind: preset.kind ?? null,
    parent: preset.parent ?? null,
    status: preset.status ?? null,
  }
}

async function confirmCreate(args: {
  kind: string
  title: string
  parent: string | null
  priority: string
  questPoints: number | null
}) {
  const status = createRequest.value?.status
  createRequest.value = null
  const out = await runMutationReturning(() =>
    createNode({
      kind: args.kind,
      title: args.title,
      parent: args.parent,
      priority: args.priority,
      questPoints: args.questPoints,
    }),
  )
  // Board columns create into a status: move the fresh node there when the
  // engine default (unstarted) differs.
  if (out && status && status !== 'unstarted' && status !== 'blocked') {
    const newId = newestCreatedId(out)
    if (newId) {
      await runMutation(() => updateNodeStatus(newId, status as 'unstarted' | 'active' | 'vanquished'))
    }
  }
}

/** Best-effort: pick the node id most recently created by the last op. */
function newestCreatedId(out: MutationPayloadLike): string | null {
  const known = new Set(nodesBeforeLastMutation.value)
  const fresh = out.realm.nodes.filter((n) => !known.has(n.id))
  if (fresh.length === 1) return fresh[0]!.id
  if (fresh.length > 1) {
    return fresh.sort((a, b) => (b.createdAt ?? '').localeCompare(a.createdAt ?? ''))[0]!.id
  }
  return null
}

const nodesBeforeLastMutation = ref<string[]>([])

/** runMutation that also returns the payload (null on failure). */
async function runMutationReturning(fn: () => Promise<MutationPayloadLike>): Promise<MutationPayloadLike | null> {
  mutationBusy.value = true
  nodesBeforeLastMutation.value = nodes.value.map((n) => n.id)
  try {
    const out = await fn()
    applySnapshot(out.realm)
    mutationError.value = ''
    return out
  } catch (err) {
    mutationError.value = String(err)
    return null
  } finally {
    mutationBusy.value = false
  }
}

/* ---- focused-node workspace editors (doc 05 §5.1) ----------------------- */

/** Status set from the editor segment. A Card → vanquished with unvanquished
 * leaves is routed through the cascade confirmation (doc 04 §4.3). */
function onSetStatus(status: 'unstarted' | 'active' | 'vanquished') {
  const n = selected.value
  if (!n || n.status === status) return
  if (isContainerKind(n.kind) && status === 'vanquished' && unvanquishedLeaves(n) > 0) {
    requestVanquish(n)
    return
  }
  void runMutation(() => updateNodeStatus(n.id, status))
}

function onSaveTitle(title: string) {
  const n = selected.value
  if (!n || title === n.title) return
  void runMutation(() => renameNode(n.id, title))
}

function onSaveBody(body: string) {
  const n = selected.value
  if (!n || body === n.body) return
  void runMutation(() => setNodeBody(n.id, body))
}

/** Metadata editor save: component edits in one engine write. The editor's
 * exposed dirty flag lets the template disable Save. */
function onSaveMeta(edits: MetaEdits) {
  const n = selected.value
  if (!n) return
  void runMutation(() =>
    updateNodeComponents({
      id: n.id,
      priority: edits.priority,
      questPoints: edits.questPoints,
      clearQuestPoints: edits.clearQuestPoints,
      disciplines: edits.disciplines,
      epics: edits.epics,
      landmark: edits.landmark,
      clearLandmark: edits.clearLandmark,
      blockedBy: edits.blockedBy,
    }),
  )
}

/* ---- move subgraph (doc 04 §4.5 Batch Re-parenting Action) ---------------- */

/** Open when non-null; a 'graph:<id>' payload means a graph drop reparent. */
const moveToRequest = ref<{ node: NodeView } | null>(null)

function requestMoveSubgraph(node: NodeView) {
  mutationError.value = ''
  moveToRequest.value = { node }
}

function confirmMoveTo(parent: string | null) {
  const req = moveToRequest.value
  moveToRequest.value = null
  if (!req) return
  void runMutation(() => reparentNode(req.node.id, parent))
}

/** Delete from any view (board/table/matrix hover trash). Routes through
 * the doc 04 §4.5 confirmation modal when the node has descendants. */
function requestDeleteById(id: string) {
  const n = nodesById.value.get(id)
  if (n) requestDelete(n)
}

/** Edit entry point from card-hover affordances: select the node and open
 * the focused-node workspace, where the title/body editors live. */
function requestEditById(id: string) {
  select(id)
  page.value = 'workspace'
  showGraph.value = true
  showTable.value = true
}

/** Drop a dragged graph node onto another node: reparent through the same
 * dialog-free path (the engine still cycle-checks). */
function onGraphDropReparent(childId: string, parentId: string) {
  const child = nodesById.value.get(childId)
  if (!child || child.parent === parentId) return
  mutationError.value = ''
  void runMutation(() => reparentNode(childId, parentId))
}

/** Double-click navigation (test-feedback A-1 follow-up): focus a node's
 * local graph from the global/overview canvases. */
function openLocalGraph(id: string) {
  select(id)
  graphScope.value = 'local'
  navChoice.value = 'graph'
  page.value = 'workspace'
}
const openPath = ref('examples/realm-demo')
const dialogError = ref('')
const openBusy = ref(false)

async function submitOpen() {
  openBusy.value = true
  dialogError.value = ''
  const ok = await openRealRealm(openPath.value.trim() || 'examples/realm-demo')
  openBusy.value = false
  if (ok) dialog.value = null
  else dialogError.value = error.value
}

function openRecent(p: RecentRealm) {
  openPath.value = p.path
  void submitOpen()
}

/* ---- palette ---------------------------------------------------------------- */

const paletteOpen = ref(false)

const paletteCommands = computed<PaletteCommand[]>(() => {
  const out: PaletteCommand[] = [
    { id: 'file.open', title: 'Open realm…' },
    { id: 'file.new', title: 'New realm…' },
  ]
  if (source.value === 'realm') {
    out.push({ id: 'file.reload', title: 'Reload realm', hint: 're-read from disk' })
  } else {
    out.push({ id: 'mock.restore', title: 'Demo realm', hint: 'mock scene' })
  }
  out.push({ id: 'edit.copyid', title: 'Copy node ID', hint: selected.value?.id })
  if (canMutateAny.value) {
    out.push({
      id: 'node.move',
      title: 'Move Subgraph To…',
      hint: selected.value ? `move ${selected.value.id} under another Card` : undefined,
    })
    out.push({ id: 'node.create', title: 'New Node…', hint: 'create from any view' })
  }
  out.push({ id: 'graph.overlay.none', title: 'Overlay: Color by Kind' })
  out.push({ id: 'graph.overlay.epic', title: 'Overlay: Color by Epic', hint: 'C E' })
  out.push({ id: 'graph.overlay.discipline', title: 'Overlay: Color by Discipline', hint: 'C D' })
  out.push({ id: 'graph.overlay.priority', title: 'Overlay: Color by Priority', hint: 'C P' })
  out.push({ id: 'graph.overlay.status', title: 'Overlay: Color by Status', hint: 'C S' })
  out.push({ id: 'graph.overlay.effort', title: 'Overlay: Effort (QP heatmap)' })
  out.push({ id: 'graph.scope.local', title: 'Graph: Local neighbourhood' })
  out.push({ id: 'graph.scope.global', title: 'Graph: Global (whole realm)', hint: 'Feature A-a' })
  out.push({ id: 'graph.scope.overview', title: 'Graph: Genre overview', hint: 'Feature A-b' })
  if (canMutateAny.value) {
    out.push({ id: 'node.vanquish', title: 'Vanquish node', hint: 'Ctrl+Enter' })
    out.push({ id: 'node.wrapshortcut', title: 'Wrap in Card Group', hint: 'Ctrl+Shift+W' })
  }
  out.push({ id: 'view.workspace', title: 'Open Explorer workspace' })
  out.push({ id: 'view.graph', title: 'Open Graph View', hint: showGraph.value ? undefined : 'reopen tab' })
  out.push({ id: 'view.table', title: 'Switch to Table View', hint: showTable.value ? undefined : 'reopen tab' })
  out.push({ id: 'view.board', title: 'Switch to Board View' })
  out.push({ id: 'view.matrix', title: 'Open Landmark Matrix' })
  out.push({ id: 'view.calendar', title: 'Open Calendar' })
  out.push({ id: 'view.settings', title: 'Open Settings' })
  out.push({ id: 'tools.clearfilters', title: 'Clear filters' })
  out.push({ id: 'tools.search', title: 'Show filters & search', hint: '/' })
  out.push({ id: 'help.about', title: 'About Legendary' })
  return out
})

function runCommand(id: string) {
  runMenu(id)
}

function togglePalette() {
  paletteOpen.value = !paletteOpen.value
}

/* ---- menu model -------------------------------------------------------------- */

const menus = computed<MenuSpec[]>(() => {
  const hasSelection = Boolean(selected.value)
  const activeFilterCount =
    filters.kinds.size + filters.statuses.size + filters.priorities.size + filters.epics.size

  const recentItems: MenuItem[] =
    recentRealms.value.length > 0
      ? recentRealms.value.map((r) => ({ id: `recent:${r.path}`, label: r.label, accel: r.path }))
      : [{ id: 'recent:empty', label: 'No recent realms', disabled: true }]

  const check = (t: 'dark' | 'light'): MenuItem => ({
    id: `view.theme.${t}`,
    label: t === 'dark' ? 'Night' : 'Map (light)',
    checked: theme.value === t,
  })

  const file: MenuSpec = {
    id: 'file',
    label: 'File',
    items: [
      { id: 'file.new', label: 'New Realm…', accel: '⌘N' },
      { id: 'file.open', label: 'Open Realm…', accel: '⌘O' },
      {
        id: 'file.recent',
        label: 'Open Recent',
        items: recentItems,
      },
      { separator: true, id: 's1' },
      {
        id: 'file.reload',
        label: 'Reload Realm',
        accel: '⌘R',
        disabled: source.value !== 'realm',
      },
      { id: 'mock.restore', label: 'Back to Demo Realm', disabled: source.value !== 'realm' },
      { separator: true, id: 's2' },
      {
        id: 'file.quit',
        label: 'Quit',
        accel: '⌘Q',
        disabled: !isDesktop.value,
      },
    ],
  }

  const edit: MenuSpec = {
    id: 'edit',
    label: 'Edit',
    items: [
      { id: 'edit.undo', label: 'Undo', accel: '⌘Z', disabled: true },
      { id: 'edit.redo', label: 'Redo', accel: '⇧⌘Z', disabled: true },
      { separator: true, id: 's1' },
      { id: 'edit.copyid', label: 'Copy Node ID', accel: '⇧⌘C', disabled: !hasSelection },
      { id: 'edit.find', label: 'Find Node…', accel: '⌘K' },
      { separator: true, id: 's2' },
      {
        id: 'tools.clearfilters',
        label: 'Clear Filters',
        disabled: activeFilterCount === 0 && !filters.search,
      },
    ],
  }

  const activeView: 'workspace' | 'graph' | 'table' | Page = (() => {
    if (page.value === 'workspace') {
      // Both tabs open → highlight the Explorer row; a single tab keeps its
      // own view row highlighted (menu checks mirror the explorer list).
      if (showGraph.value && showTable.value) return 'workspace'
      return showGraph.value ? 'graph' : 'table'
    }
    return page.value
  })()

  const VIEW_ITEMS: { key: 'workspace' | 'graph' | 'table' | Page; label: string }[] = [
    { key: 'workspace', label: 'Explorer Workspace' },
    { key: 'graph', label: 'Graph View' },
    { key: 'table', label: 'Table View' },
    { key: 'board', label: 'Board View' },
    { key: 'matrix', label: 'Landmark Matrix' },
    { key: 'calendar', label: 'Calendar' },
    { key: 'entities', label: 'Entities' },
  ]

  const view: MenuSpec = {
    id: 'view',
    label: 'View',
    items: [
      ...VIEW_ITEMS.map<MenuItem>((v) => ({
        id: `view.${v.key}`,
        label: v.label,
        checked: activeView === v.key,
      })),
      { separator: true, id: 's1' },
      {
        id: 'view.appearance',
        label: 'Appearance',
        items: [check('dark'), check('light')],
      },
    ],
  }

  const go: MenuSpec = {
    id: 'go',
    label: 'Go',
    items: [
      {
        id: 'go.parent',
        label: 'Parent Node',
        accel: '⌘↑',
        disabled: !selected.value?.parent,
      },
      {
        id: 'go.root',
        label: 'Root Card',
        disabled: !selected.value,
      },
      { separator: true, id: 's1' },
      { id: 'go.next', label: 'Next Row', disabled: true },
      { id: 'go.prev', label: 'Previous Row', disabled: true },
    ],
  }

  const graphMenu: MenuSpec = {
    id: 'graph',
    label: 'Graph',
    items: [
      { id: 'graph.overlay.none', label: 'Color by Kind', checked: overlayMode.value === 'none' },
      { id: 'graph.overlay.epic', label: 'Color by Epic', checked: overlayMode.value === 'epic', accel: 'C E' },
      { id: 'graph.overlay.discipline', label: 'Color by Discipline', checked: overlayMode.value === 'discipline', accel: 'C D' },
      { id: 'graph.overlay.priority', label: 'Color by Priority', checked: overlayMode.value === 'priority', accel: 'C P' },
      { id: 'graph.overlay.status', label: 'Color by Status', checked: overlayMode.value === 'status', accel: 'C S' },
      { id: 'graph.overlay.effort', label: 'Effort (QP heatmap)', checked: overlayMode.value === 'effort' },
      { separator: true, id: 's1' },
      { id: 'graph.scope.local', label: 'Local Graph', checked: graphScope.value === 'local' },
      { id: 'graph.scope.global', label: 'Global Graph (whole realm)', checked: graphScope.value === 'global' },
      { id: 'graph.scope.overview', label: 'Genre Overview', checked: graphScope.value === 'overview' },
      { separator: true, id: 's2' },
      { id: 'graph.autolayout', label: 'Auto-layout', checked: true, disabled: true },
      { id: 'graph.fit', label: 'Zoom to Fit', disabled: true },
    ],
  }

  const tools: MenuSpec = {
    id: 'tools',
    label: 'Tools',
    items: [
      { id: 'tools.palette', label: 'Command Palette', accel: '⌘K' },
      { id: 'tools.search', label: 'Search Filters', accel: '/' },
      {
        id: 'tools.clearfilters',
        label: 'Clear Filters',
        disabled: activeFilterCount === 0 && !filters.search,
      },
      { separator: true, id: 's1' },
      { id: 'tools.realm', label: 'Realm Status…', disabled: true },
    ],
  }

  const windowMenu: MenuSpec = {
    id: 'window',
    label: 'Window',
    items: [
      { id: 'window.fullscreen', label: 'Toggle Full Screen', accel: 'F11' },
      { id: 'window.zoomin', label: 'Zoom In', disabled: true },
      { id: 'window.zoomout', label: 'Zoom Out', disabled: true },
      { separator: true, id: 's1' },
      { id: 'window.reopen', label: 'Reopen Closed Tab', disabled: true },
    ],
  }

  const help: MenuSpec = {
    id: 'help',
    label: 'Help',
    items: [
      { id: 'help.shortcuts', label: 'Keyboard Shortcuts' },
      { id: 'help.about', label: 'About Legendary' },
    ],
  }

  return [file, edit, view, go, graphMenu, tools, windowMenu, help]
})

/* ---- menu routing ------------------------------------------------------------ */

async function runMenu(id: string) {
  if (id.startsWith('recent:')) {
    const path = id.slice('recent:'.length)
    const rec = recentRealms.value.find((r) => r.path === path)
    if (rec) openRecent(rec)
    return
  }
  switch (id) {
    case 'file.new':
      dialog.value = 'new'
      break
    case 'file.open':
      dialog.value = 'open'
      dialogError.value = ''
      break
    case 'file.reload':
      void reloadRealm()
      break
    case 'mock.restore':
      useMockRealm()
      toWorkspace()
      break
    case 'file.quit':
      if (inDesktop()) {
        const win = await (await import('@tauri-apps/api/window')).getCurrentWindow()
        await win.close()
      }
      break
    case 'node.move': {
      const n = selected.value
      if (n) requestMoveSubgraph(n)
      break
    }
    case 'node.create':
      requestCreate({ parent: selected.value?.id ?? null })
      break
    case 'edit.copyid': {
      const n = selected.value
      if (n) {
        try {
          await navigator.clipboard.writeText(n.id)
        } catch {
          /* clipboard unavailable */
        }
      }
      break
    }
    case 'edit.find':
    case 'tools.palette':
      togglePalette()
      break
    case 'tools.search':
      if (page.value !== 'workspace') toWorkspace()
      focusFilters()
      break
    case 'tools.clearfilters':
      clearFilters()
      break
    case 'view.workspace':
    case 'view.graph':
    case 'view.table':
    case 'view.board':
    case 'view.matrix':
    case 'view.calendar':
    case 'view.entities':
    case 'view.settings': {
      const target = id.slice('view.'.length)
      openView(target)
      break
    }
    case 'view.theme.dark':
      setTheme('dark')
      break
    case 'view.theme.light':
      setTheme('light')
      break
    case 'go.parent': {
      const p = selected.value?.parent
      if (p) select(p)
      break
    }
    case 'go.root': {
      let cur = selected.value
      let root = cur
      while (cur?.parent) {
        root = nodesById.value.get(cur.parent) ?? root
        cur = root
      }
      if (root) select(root.id)
      break
    }
    case 'graph.overlay.none':
      overlayMode.value = 'none'
      break
    case 'graph.overlay.epic':
      overlayMode.value = 'epic'
      break
    case 'graph.overlay.discipline':
      overlayMode.value = 'discipline'
      break
    case 'graph.overlay.priority':
      overlayMode.value = 'priority'
      break
    case 'graph.overlay.status':
      overlayMode.value = 'status'
      break
    case 'graph.overlay.effort':
      overlayMode.value = 'effort'
      break
    case 'graph.scope.local':
      graphScope.value = 'local'
      break
    case 'graph.scope.global':
      graphScope.value = 'global'
      break
    case 'graph.scope.overview':
      graphScope.value = 'overview'
      break
    case 'node.vanquish': {
      const n = selected.value
      if (n) requestVanquish(n)
      break
    }
    case 'node.wrapshortcut': {
      onWrap()
      break
    }
    case 'node.moveshortcut': {
      const n = selected.value
      if (n) requestMoveSubgraph(n)
      break
    }
    case 'window.fullscreen': {
      try {
        if (document.fullscreenElement) await document.exitFullscreen()
        else await document.documentElement.requestFullscreen()
      } catch {
        /* fullscreen denied */
      }
      break
    }
    case 'help.shortcuts':
      page.value = 'settings'
      break
    case 'help.about':
      dialog.value = 'about'
      break
    default:
      break
  }
}

/* ---- keyboard ----------------------------------------------------------------- */

function onGlobalKey(e: KeyboardEvent) {
  const el = e.target instanceof HTMLElement ? e.target : null
  const typing = el?.closest('input, textarea, [contenteditable="true"]') != null
  const mod = e.ctrlKey || e.metaKey

  if (typing && !(e.key === 'Escape')) return

  if (mod && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    togglePalette()
    return
  }
  if (!typing && e.key === '/') {
    e.preventDefault()
    togglePalette()
    return
  }
  if (mod && e.key.toLowerCase() === 'o') {
    e.preventDefault()
    dialog.value = 'open'
    return
  }
  if (mod && e.key.toLowerCase() === 'r') {
    e.preventDefault()
    void reloadRealm()
    return
  }

  // doc 05 §5.7 contextual graph actions (desktop mutation contexts only)
  if (canMutateAny.value) {
    if (mod && e.shiftKey && e.key.toLowerCase() === 'w') {
      e.preventDefault()
      onWrap()
      return
    }
    if (mod && e.key.toLowerCase() === 'm') {
      e.preventDefault()
      const n = selected.value
      if (n) requestMoveSubgraph(n)
      return
    }
    if (mod && e.key === 'Enter') {
      e.preventDefault()
      const n = selected.value
      if (n) requestVanquish(n)
      return
    }
  }

  // doc 05 §5.7 overlay toggles: the two-key chord C then E/D/P/S
  if (overlayChordArmed && !typing) {
    const k = e.key.toLowerCase()
    if (k === 'e') {
      overlayMode.value = overlayMode.value === 'epic' ? 'none' : 'epic'
      e.preventDefault()
      overlayChordArmed = false
      return
    }
    if (k === 'd') {
      overlayMode.value = overlayMode.value === 'discipline' ? 'none' : 'discipline'
      e.preventDefault()
      overlayChordArmed = false
      return
    }
    if (k === 'p') {
      overlayMode.value = overlayMode.value === 'priority' ? 'none' : 'priority'
      e.preventDefault()
      overlayChordArmed = false
      return
    }
    if (k === 's') {
      overlayMode.value = overlayMode.value === 'status' ? 'none' : 'status'
      e.preventDefault()
      overlayChordArmed = false
      return
    }
    overlayChordArmed = false
  }
  if (!typing && !mod && e.key.toLowerCase() === 'c') {
    overlayChordArmed = true
    // disarm if no follow-up key arrives within 1.2s
    window.setTimeout(() => {
      overlayChordArmed = false
    }, 1200)
  }
}

/** True while a `C` chord is waiting for its mode letter. */
let overlayChordArmed = false

onMounted(() => window.addEventListener('keydown', onGlobalKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onGlobalKey))
</script>

<template>
  <div class="shell">
    <TopBar :menus="menus" @run="runMenu" @palette="togglePalette()" />

    <div class="app-body">
      <template v-if="page === 'workspace'">
        <ExplorerPanel
          :nodes="nodes"
          :realm-name="realmName"
          :realm-path="realmPath"
          :nav-id="explorerActive"
          :filters="filters"
          :epic-rows="epicRowsC"
          :discipline-rows="disciplineRowsC"
          :landmark-rows="landmarkRowsC"
          :epic-tree="epicTreeC"
          :discipline-tree="disciplineTreeC"
          @nav="openView"
          @toggle-kind="onToggleKind"
          @toggle-epic="onToggleEpic"
          @toggle-discipline="onToggleDiscipline"
          @toggle-landmark="onToggleLandmark"
          @open-filters="filtersOpen = true; focusFilters()"
        />
        <FiltersPanel
          v-if="filtersOpen"
          :filters="filters"
          :kind-rows="kindRowsC"
          :status-rows="statusRowsC"
          :priority-rows="priorityRowsC"
          :epic-rows="epicRowsC"
          :focus-signal="focusSignal"
          @toggle="toggleFilter"
          @clear="clearFilters"
          @close="filtersOpen = false"
          @update:search="filters.search = $event"
        />
        <!-- A-3: the reopen affordance is always present while filters are
             hidden — a wide, labelled rail button (works by click AND by
             the ⌘/ keyboard shortcut, Tools → Search Filters, or the
             Command Palette "Search Filters" entry). -->
        <button
          v-else
          type="button"
          class="filters-reopen"
          title="Show filters (and focus search)"
          aria-label="Show filters"
          @click="filtersOpen = true; focusFilters()"
        >
          <PhFunnel :size="13" aria-hidden="true" />
          <span class="filters-reopen-label">Filters</span>
        </button>

        <WorkspaceCenter
          :nodes="nodes"
          :nodes-by-id="nodesById"
          :filters="filters"
          :selected-id="selectedId"
          :is-mock-scene="isMockScene"
          :realm-name="realmName"
          :status-counts="statusRowsC"
          :show-graph="showGraph"
          :show-table="showTable"
          :overlay-mode="overlayMode"
          :scope="graphScope"
          :completion-level="completionLevel"
          :can-mutate="canMutateAny"
          @select="select"
          @update:show-graph="showGraph = $event"
          @update:show-table="showTable = $event"
          @toggle-status="toggleFilter('statuses', $event)"
          @open-filters="filtersOpen = true; focusFilters()"
          @clear-filters="clearFilters"
          @focus-search="filtersOpen = true; focusFilters()"
          @reparent="onGraphDropReparent"
          @open-local="openLocalGraph"
          @create="requestCreate({ parent: $event })"
          @create-any="requestCreate()"
          @delete-node="requestDeleteById"
          @update:overlay-mode="overlayMode = $event"
          @update:scope="graphScope = $event"
          @update:completion-level="completionLevel = $event"
        />

        <aside class="inspector" aria-label="Detail inspector">
          <NodeDetail
            :node="selected"
            :nodes-by-id="nodesById"
            :realm-name="realmName"
            :can-mutate="isDesktop && source === 'realm'"
            :busy="mutationBusy"
            :discipline-options="disciplineOptionsC"
            :epic-options="epicOptionsC"
            :landmark-options="landmarkOptionsC"
            @select="select"
            @filter="onDetailFilter"
            @vanquish="selected && requestVanquish(selected)"
            @delete-subgraph="selected && requestDelete(selected)"
            @wrap="onWrap"
            @reparent-root="onReparentRoot"
            @save-title="onSaveTitle"
            @save-body="onSaveBody"
            @set-status="onSetStatus"
            @save-meta="onSaveMeta"
            @move-subgraph="selected && requestMoveSubgraph(selected)"
          />
        </aside>
      </template>

      <template v-else>
        <ExplorerPanel
          :nodes="nodes"
          :realm-name="realmName"
          :realm-path="realmPath"
          :nav-id="explorerActive"
          :filters="filters"
          :epic-rows="epicRowsC"
          :discipline-rows="disciplineRowsC"
          :landmark-rows="landmarkRowsC"
          :epic-tree="epicTreeC"
          :discipline-tree="disciplineTreeC"
          @nav="openView"
          @toggle-kind="onToggleKind"
          @toggle-epic="onToggleEpic"
          @toggle-discipline="onToggleDiscipline"
          @toggle-landmark="onToggleLandmark"
          @open-filters="filtersOpen = true; toWorkspace(); focusFilters()"
        />

        <main class="page-main">
          <template v-if="page === 'board'">
            <BoardView
              :nodes="filteredNodes"
              :selected-id="selectedId"
              :can-mutate="canMutateAny"
              @select="select"
              @create="requestCreate({ status: $event })"
              @edit="requestEditById"
              @delete-node="requestDeleteById"
            />
          </template>
          <template v-else-if="page === 'matrix'">
            <LandmarkMatrix
              :nodes="filteredNodes"
              :selected-id="selectedId"
              :can-mutate="canMutateAny"
              @select="select"
              @delete-node="requestDeleteById"
            />
          </template>
          <template v-else-if="page === 'entities'">
            <EntitiesView
              :nodes="nodes"
              :kind-counts="kindRowsC"
              :can-mutate="canMutateAny"
              @browse="browseKind"
              @create="requestCreate({ kind: $event })"
            />
          </template>
          <template v-else-if="page === 'calendar'">
            <CalendarView />
          </template>
          <template v-else-if="page === 'settings'">
            <SettingsView
              :theme="theme"
              :realm-name="realmName"
              :node-count="nodes.length"
              @update:theme="setTheme"
            />
          </template>
        </main>

        <aside v-if="page === 'board' || page === 'matrix' || page === 'entities'" class="inspector" aria-label="Detail inspector">
          <NodeDetail
            :node="selected"
            :nodes-by-id="nodesById"
            :realm-name="realmName"
            :can-mutate="isDesktop && source === 'realm'"
            :busy="mutationBusy"
            :discipline-options="disciplineOptionsC"
            :epic-options="epicOptionsC"
            :landmark-options="landmarkOptionsC"
            @select="select"
            @filter="onDetailFilter"
            @vanquish="selected && requestVanquish(selected)"
            @delete-subgraph="selected && requestDelete(selected)"
            @wrap="onWrap"
            @reparent-root="onReparentRoot"
            @save-title="onSaveTitle"
            @save-body="onSaveBody"
            @set-status="onSetStatus"
            @save-meta="onSaveMeta"
            @move-subgraph="selected && requestMoveSubgraph(selected)"
          />
        </aside>
      </template>
    </div>

    <StatusBar
      :nodes="totals.nodes"
      :links="totals.links"
      :realm-name="realmName"
      :is-mock="source === 'mock'"
      :live="isDesktop && source === 'realm'"
      :error="error || mutationError"
    />

    <!-- command palette -->
    <CommandPalette
      v-if="paletteOpen"
      :nodes="nodes"
      :commands="paletteCommands"
      @close="paletteOpen = false"
      @command="(id) => { paletteOpen = false; runCommand(id) }"
      @select-node="(id) => { paletteOpen = false; select(id); toWorkspace() }"
    />

    <!-- dialogs -->
    <MutationDialog
      v-if="mutationRequest"
      :request="mutationRequest"
      :busy="mutationBusy"
      @close="mutationRequest = null"
      @confirm="confirmMutation"
    />
    <MoveToDialog
      v-if="moveToRequest"
      :node="moveToRequest.node"
      :nodes="nodes"
      :busy="mutationBusy"
      @close="moveToRequest = null"
      @move="confirmMoveTo"
    />
    <NodeCreateDialog
      v-if="createRequest"
      :nodes="nodes"
      :initial-kind="createRequest.kind"
      :initial-parent="createRequest.parent"
      :busy="mutationBusy"
      :error="mutationError"
      @close="createRequest = null"
      @create="confirmCreate"
    />
    <AboutDialog v-if="dialog === 'about'" :is-desktop="isDesktop" @close="dialog = null" />
    <OpenRealmDialog
      v-if="dialog === 'open'"
      :realm-path="realmPath"
      :is-mock="source === 'mock'"
      :mock-name="MOCK_REALM.name"
      :mock-path="MOCK_REALM.path"
      :recents="recentRealms"
      :busy="openBusy"
      :error="dialogError"
      :is-desktop="isDesktop"
      v-model:path="openPath"
      @close="dialog = null"
      @use-mock="useMockRealm(); dialog = null"
      @open-recent="openRecent"
      @submit="submitOpen"
    />
    <NewRealmDialog v-if="dialog === 'new'" @close="dialog = null" />
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

.app-body {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: stretch;
}

.filters-reopen {
  flex: none;
  width: 30px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  gap: 6px;
  padding-top: 10px;
  background: var(--bg-1);
  border-right: 1px solid var(--line-1);
  color: var(--text-3);
}

.filters-reopen:hover {
  color: var(--gold);
  background: var(--bg-2);
}

.filters-reopen-label {
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  writing-mode: vertical-rl;
  user-select: none;
}

.page-main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
}

.inspector {
  width: 352px;
  flex: none;
  min-height: 0;
  background: var(--bg-1);
  border-left: 1px solid var(--line-1);
  overflow: hidden;
  display: flex;
}

.inspector > * {
  width: 100%;
}

.inspector :deep(.detail) {
  padding: 16px 20px 56px;
  max-width: none;
}

/* ---- narrow windows: keep the workspace usable ----------------------- */

@media (max-width: 1200px) {
  .inspector {
    width: 300px;
  }
}

@media (max-width: 980px) {
  .inspector {
    width: 268px;
  }
}
</style>
