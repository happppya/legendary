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
import SettingsView from './components/pages/SettingsView.vue'
import CommandPalette, { type PaletteCommand } from './components/overlays/CommandPalette.vue'
import AboutDialog from './components/overlays/AboutDialog.vue'
import OpenRealmDialog from './components/overlays/OpenRealmDialog.vue'
import NewRealmDialog from './components/overlays/NewRealmDialog.vue'
import { inDesktop, openRealm } from './api'
import type { NodeView } from './types'
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
import { linkCount } from './lib/node'
import { MOCK_BY_KEY, MOCK_NODES, MOCK_REALM } from './lib/mockRealm'

type Page = 'workspace' | 'board' | 'entities' | 'calendar' | 'settings'

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
const showGraph = ref(true)
const showTable = ref(true)
/** Last view chosen in the explorer list (workspace highlights as one row). */
const navChoice = ref<'workspace' | 'graph' | 'table'>('graph')

const explorerActive = computed(() =>
  page.value === 'workspace' ? navChoice.value : page.value,
)

function openView(id: string) {
  switch (id) {
    case 'workspace':
      page.value = 'workspace'
      showGraph.value = true
      showTable.value = true
      navChoice.value = 'workspace'
      break
    case 'graph':
      page.value = 'workspace'
      showGraph.value = true
      showTable.value = false
      navChoice.value = 'graph'
      break
    case 'table':
      page.value = 'workspace'
      showGraph.value = false
      showTable.value = true
      navChoice.value = 'table'
      break
    case 'board':
      page.value = 'board'
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
  showGraph.value = true
  showTable.value = true
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

/* ---- dialogs ------------------------------------------------------------- */

type Dialog = 'open' | 'new' | 'about' | null
const dialog = ref<Dialog>(null)
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
  out.push({ id: 'view.workspace', title: 'Open Explorer workspace' })
  out.push({ id: 'view.table', title: 'Switch to Table View' })
  out.push({ id: 'view.board', title: 'Switch to Board View' })
  out.push({ id: 'view.calendar', title: 'Open Calendar' })
  out.push({ id: 'view.settings', title: 'Open Settings' })
  out.push({ id: 'tools.clearfilters', title: 'Clear filters' })
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
      { id: 'graph.overlay.kind', label: 'Color by Kind', checked: true },
      { id: 'graph.overlay.epic', label: 'Color by Epic', disabled: true, title: 'Epic overlay lands with the graph milestone' },
      { id: 'graph.overlay.discipline', label: 'Color by Discipline', disabled: true },
      { id: 'graph.overlay.priority', label: 'Color by Priority', disabled: true },
      { id: 'graph.overlay.status', label: 'Color by Status', disabled: true },
      { separator: true, id: 's1' },
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
}

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
        <button
          v-else
          type="button"
          class="filters-reopen"
          title="Show filters"
          aria-label="Show filters"
          @click="filtersOpen = true"
        >
          <PhFunnel :size="13" aria-hidden="true" />
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
          @select="select"
          @update:show-graph="showGraph = $event"
          @update:show-table="showTable = $event"
          @toggle-status="toggleFilter('statuses', $event)"
          @open-filters="filtersOpen = true; focusFilters()"
          @clear-filters="clearFilters"
          @focus-search="filtersOpen = true; focusFilters()"
        />

        <aside class="inspector" aria-label="Detail inspector">
          <NodeDetail
            :node="selected"
            :nodes-by-id="nodesById"
            :realm-name="realmName"
            @select="select"
            @filter="onDetailFilter"
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
            <BoardView :nodes="filteredNodes" :selected-id="selectedId" @select="select" />
          </template>
          <template v-else-if="page === 'entities'">
            <EntitiesView :nodes="nodes" :kind-counts="kindRowsC" @browse="browseKind" />
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

        <aside v-if="page === 'board' || page === 'entities'" class="inspector" aria-label="Detail inspector">
          <NodeDetail
            :node="selected"
            :nodes-by-id="nodesById"
            :realm-name="realmName"
            @select="select"
            @filter="onDetailFilter"
          />
        </aside>
      </template>
    </div>

    <StatusBar
      :nodes="totals.nodes"
      :links="totals.links"
      :realm-name="realmName"
      :is-mock="source === 'mock'"
      :error="error"
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
  width: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-1);
  border-right: 1px solid var(--line-1);
  color: var(--text-3);
}

.filters-reopen:hover {
  color: var(--gold);
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
