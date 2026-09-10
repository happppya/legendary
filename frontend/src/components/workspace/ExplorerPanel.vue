<script setup lang="ts">
// Explorer column: realm selector + navigation groups (Graph, Entities,
// Taxonomy, Tools). Entity and taxonomy rows double as live filters over
// the workspace table/board; taxonomy rows expand into their branch lists.

import { computed, ref } from 'vue'
import {
  PhCalendarBlank,
  PhCaretDown,
  PhCaretRight,
  PhCompass,
  PhKanban,
  PhLinkSimple,
  PhMagnifyingGlass,
  PhShapes,
  PhSquaresFour,
  PhTrash,
  PhTree,
} from '@phosphor-icons/vue'
import type { Component } from 'vue'
import type { NodeView } from '../../types'
import type { CheckRow, FilterModel, TaxRow } from '../../lib/filters'

const props = defineProps<{
  nodes: NodeView[]
  realmName: string
  realmPath: string
  navId: string
  filters: FilterModel
  /** Flat branch rows - used for realms without taxonomy hierarchy. */
  epicRows: CheckRow[]
  disciplineRows: CheckRow[]
  landmarkRows: CheckRow[]
  /** Collapsible taxonomy trees (.legend/taxonomy.yaml); null = flat rows above. */
  epicTree: TaxRow[] | null
  disciplineTree: TaxRow[] | null
}>()

const emit = defineEmits<{
  nav: [id: string]
  toggleKind: [kind: string]
  toggleEpic: [label: string]
  toggleDiscipline: [label: string]
  toggleLandmark: [label: string]
  openFilters: []
}>()

const kindCounts = computed(() => {
  const m: Record<string, number> = { card: 0, genre: 0, action: 0, guard: 0, idea: 0 }
  for (const n of props.nodes) m[n.kind] = (m[n.kind] ?? 0) + 1
  return m
})

/* ---- taxonomy expanders --------------------------------------------- */

const expanded = ref<Set<string>>(new Set(['tax-epics']))
function toggleSection(key: string) {
  const next = new Set(expanded.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  expanded.value = next
}

/* ---- collapsible taxonomy trees ------------------------------------- */

interface FlatTaxRow {
  key: string
  label: string
  count: number
  depth: number
  caret: boolean
}

/** Flatten a TaxRow[] honouring the per-branch expansion state. */
function flattenTax(tree: TaxRow[] | null, ns: string): FlatTaxRow[] {
  if (!tree) return []
  const out: FlatTaxRow[] = []
  const walk = (rows: TaxRow[], depth: number) => {
    for (const r of rows) {
      const caret = Boolean(r.children?.length)
      out.push({ key: r.key, label: r.label, count: r.count, depth, caret })
      if (caret && expanded.value.has(`${ns}:${r.key}`)) walk(r.children ?? [], depth + 1)
    }
  }
  walk(tree, 0)
  return out
}

const flatEpicTree = computed<FlatTaxRow[]>(() => flattenTax(props.epicTree, 'e'))
const flatDiscTree = computed<FlatTaxRow[]>(() => flattenTax(props.disciplineTree, 'd'))

/** Vocabulary size shown next to the group title: leaves when a tree is
 * supplied (roots are containers), distinct rows otherwise. */
function leafCount(tree: TaxRow[] | null): number {
  if (!tree) return 0
  let n = 0
  const walk = (rows: TaxRow[]) => {
    for (const r of rows) {
      if (r.children?.length) walk(r.children)
      else n += 1
    }
  }
  walk(tree)
  return n
}

const epicBadge = computed(() => (props.epicTree ? leafCount(props.epicTree) : props.epicRows.length))
const discBadge = computed(() => (props.disciplineTree ? leafCount(props.disciplineTree) : props.disciplineRows.length))

function taxOpen(ns: string, key: string): boolean {
  return expanded.value.has(`${ns}:${key}`)
}

function toggleTaxCat(ns: string, key: string) {
  const next = new Set(expanded.value)
  const k = `${ns}:${key}`
  if (next.has(k)) next.delete(k)
  else next.add(k)
  expanded.value = next
}

/* ---- tools counts ---------------------------------------------------- */

const tools = computed(() => {
  const ids = new Set(props.nodes.map((n) => n.id))
  const referenced = new Set<string>()
  for (const n of props.nodes) {
    if (n.parent && ids.has(n.parent)) referenced.add(n.parent)
    for (const b of n.blockedBy) if (ids.has(b)) referenced.add(b)
  }
  const childrenOf = new Set<string>()
  for (const n of props.nodes) if (n.parent) childrenOf.add(n.parent)
  const orphans = props.nodes.filter((n) => !n.parent && !childrenOf.has(n.id)).length
  return { backlinks: referenced.size, orphans, trash: 0 }
})

function isNavActive(id: string): boolean {
  return props.navId === id
}

const KIND_GROUPS: { id: string; label: string; hint?: string }[] = [
  { id: 'card', label: 'Cards', hint: 'Groups' },
  { id: 'genre', label: 'Genres', hint: 'Categories' },
  { id: 'action', label: 'Actions' },
  { id: 'guard', label: 'Guards' },
  { id: 'idea', label: 'Ideas' },
]

interface ViewRow {
  id: string
  label: string
  icon: Component
  sep?: boolean
}

/** View switcher rows - every top-level page lives here now. The Explorer
 * workspace already contains the graph and table panes (they are tabs, not
 * separate pages), so there are no Graph/Table rows (test-feedback Bug7);
 * Settings moved under the File menu (Bug8). */
const VIEWS: ViewRow[] = [
  { id: 'workspace', label: 'Explorer', icon: PhCompass },
  { id: 'board', label: 'Board View', icon: PhKanban },
  { id: 'matrix', label: 'Landmark Matrix', icon: PhSquaresFour },
  { id: 'calendar', label: 'Calendar', icon: PhCalendarBlank },
  { id: 'entities', label: 'Entities', icon: PhShapes },
]
</script>

<template>
  <aside class="explorer" aria-label="Explorer">
    <header class="ex-header">
      <span class="ex-eyebrow">Explorer</span>
    </header>

    <button type="button" class="realm-select" :title="`Realm at ${realmPath}`">
      <span class="realm-label">{{ realmName }}</span>
      <PhCaretDown :size="11" class="realm-caret" aria-hidden="true" />
    </button>

    <div class="ex-body">
      <!-- Views group -->
      <section class="group">
        <h2 class="group-title">Views</h2>
        <template v-for="row in VIEWS" :key="row.id">
          <div v-if="row.sep" class="group-sep" aria-hidden="true"></div>
          <button
            type="button"
            class="row view-row"
            :class="{ active: isNavActive(row.id) }"
            :title="row.label"
            :aria-current="isNavActive(row.id) ? 'page' : undefined"
            @click="emit('nav', row.id)"
          >
            <span class="vicon" aria-hidden="true">
              <component :is="row.icon" :size="14" />
            </span>
            <span class="row-label">{{ row.label }}</span>
          </button>
        </template>
      </section>

      <!-- Entities group -->
      <section class="group">
        <h2 class="group-title">Entities</h2>
        <button
          v-for="k in KIND_GROUPS"
          :key="k.id"
          type="button"
          class="row"
          :class="{ active: filters.kinds.has(k.id) }"
          :title="`Show or hide ${k.label.toLowerCase()}`"
          @click="emit('toggleKind', k.id)"
        >
          <span class="check" aria-hidden="true">{{ filters.kinds.has(k.id) ? '✓' : '' }}</span>
          <span class="row-label">
            {{ k.label }}<span v-if="k.hint" class="row-hint">{{ k.hint }}</span>
          </span>
          <span class="badge mono">{{ kindCounts[k.id] ?? 0 }}</span>
        </button>
      </section>

      <!-- Taxonomy group -->
      <section class="group">
        <h2 class="group-title">Taxonomy</h2>
        <button
          type="button"
          class="row expander"
          :aria-expanded="expanded.has('tax-epics')"
          @click="toggleSection('tax-epics')"
        >
          <PhCaretRight v-if="!expanded.has('tax-epics')" :size="11" class="twisty" aria-hidden="true" />
          <PhCaretDown v-else :size="11" class="twisty" aria-hidden="true" />
          <span class="row-label">Epics</span>
          <span class="badge mono">{{ epicBadge }}</span>
        </button>
        <div v-if="expanded.has('tax-epics')" class="sub-list" role="group">
          <template v-if="epicTree">
            <div
              v-for="row in flatEpicTree"
              :key="row.key"
              class="taxrow"
              :style="{ paddingLeft: 4 + row.depth * 14 + 'px' }"
            >
              <button
                v-if="row.caret"
                type="button"
                class="tax-caret"
                :aria-expanded="taxOpen('e', row.key)"
                :aria-label="`${taxOpen('e', row.key) ? 'Collapse' : 'Expand'} epic ${row.label}`"
                @click.stop="toggleTaxCat('e', row.key)"
              >
                <PhCaretRight v-if="!taxOpen('e', row.key)" :size="11" aria-hidden="true" />
                <PhCaretDown v-else :size="11" aria-hidden="true" />
              </button>
              <span v-else class="tax-caret-spacer" aria-hidden="true"></span>
              <button
                type="button"
                class="row tax-leaf"
                :class="{ active: filters.epics.has(row.key) }"
                :title="`Filter nodes in epic ${row.label}`"
                @click="emit('toggleEpic', row.key)"
              >
                <span class="check" aria-hidden="true">{{ filters.epics.has(row.key) ? '✓' : '' }}</span>
                <span class="row-label">{{ row.label }}</span>
                <span class="badge mono">{{ row.count }}</span>
              </button>
            </div>
          </template>
          <button
            v-else
            v-for="row in epicRows"
            :key="row.key"
            type="button"
            class="row sub"
            :class="{ active: filters.epics.has(row.key) }"
            :title="`Filter nodes in epic ${row.label}`"
            @click="emit('toggleEpic', row.key)"
          >
            <span class="check" aria-hidden="true">{{ filters.epics.has(row.key) ? '✓' : '' }}</span>
            <span class="row-label">{{ row.label }}</span>
            <span class="badge mono">{{ row.count }}</span>
          </button>
        </div>

        <button
          type="button"
          class="row expander"
          :aria-expanded="expanded.has('tax-disciplines')"
          @click="toggleSection('tax-disciplines')"
        >
          <PhCaretRight v-if="!expanded.has('tax-disciplines')" :size="11" class="twisty" aria-hidden="true" />
          <PhCaretDown v-else :size="11" class="twisty" aria-hidden="true" />
          <span class="row-label">Disciplines</span>
          <span class="badge mono">{{ discBadge }}</span>
        </button>
        <div v-if="expanded.has('tax-disciplines')" class="sub-list" role="group">
          <template v-if="disciplineTree">
            <div
              v-for="row in flatDiscTree"
              :key="row.key"
              class="taxrow"
              :style="{ paddingLeft: 4 + row.depth * 14 + 'px' }"
            >
              <button
                v-if="row.caret"
                type="button"
                class="tax-caret"
                :aria-expanded="taxOpen('d', row.key)"
                :aria-label="`${taxOpen('d', row.key) ? 'Collapse' : 'Expand'} discipline ${row.label}`"
                @click.stop="toggleTaxCat('d', row.key)"
              >
                <PhCaretRight v-if="!taxOpen('d', row.key)" :size="11" aria-hidden="true" />
                <PhCaretDown v-else :size="11" aria-hidden="true" />
              </button>
              <span v-else class="tax-caret-spacer" aria-hidden="true"></span>
              <button
                type="button"
                class="row tax-leaf"
                :class="{ active: filters.disciplines.has(row.key) }"
                :title="`Filter nodes in discipline ${row.label}`"
                @click="emit('toggleDiscipline', row.key)"
              >
                <span class="check" aria-hidden="true">{{ filters.disciplines.has(row.key) ? '✓' : '' }}</span>
                <span class="row-label">{{ row.label }}</span>
                <span class="badge mono">{{ row.count }}</span>
              </button>
            </div>
          </template>
          <button
            v-else
            v-for="row in disciplineRows"
            :key="row.key"
            type="button"
            class="row sub"
            :class="{ active: filters.disciplines.has(row.key) }"
            :title="`Filter nodes in discipline ${row.label}`"
            @click="emit('toggleDiscipline', row.key)"
          >
            <span class="check" aria-hidden="true">{{ filters.disciplines.has(row.key) ? '✓' : '' }}</span>
            <span class="row-label">{{ row.label }}</span>
            <span class="badge mono">{{ row.count }}</span>
          </button>
        </div>

        <button
          type="button"
          class="row expander"
          :aria-expanded="expanded.has('tax-landmarks')"
          @click="toggleSection('tax-landmarks')"
        >
          <PhCaretRight v-if="!expanded.has('tax-landmarks')" :size="11" class="twisty" aria-hidden="true" />
          <PhCaretDown v-else :size="11" class="twisty" aria-hidden="true" />
          <span class="row-label">Landmarks</span>
          <span class="badge mono">{{ landmarkRows.length }}</span>
        </button>
        <div v-if="expanded.has('tax-landmarks')" class="sub-list" role="group">
          <button
            v-for="row in landmarkRows"
            :key="row.key"
            type="button"
            class="row sub"
            :class="{ active: filters.landmarks.has(row.label) }"
            :title="`Filter nodes in landmark ${row.label}`"
            @click="emit('toggleLandmark', row.label)"
          >
            <span class="check" aria-hidden="true">{{ filters.landmarks.has(row.label) ? '✓' : '' }}</span>
            <span class="row-label">{{ row.label }}</span>
            <span class="badge mono">{{ row.count }}</span>
          </button>
        </div>
      </section>

      <!-- Tools group -->
      <section class="group">
        <h2 class="group-title">Tools</h2>
        <button type="button" class="row" @click="emit('openFilters')">
          <PhMagnifyingGlass :size="12" class="tool-icon" aria-hidden="true" />
          <span class="row-label">Search</span>
        </button>
        <button type="button" class="row faint" title="Inbound references to the selected node - coming with the editor milestone">
          <PhLinkSimple :size="12" class="tool-icon" aria-hidden="true" />
          <span class="row-label">Backlinks</span>
          <span class="badge mono">{{ tools.backlinks }}</span>
        </button>
        <button type="button" class="row faint" title="Nodes with no parent and no children">
          <PhTree :size="12" class="tool-icon" aria-hidden="true" />
          <span class="row-label">Orphans</span>
          <span class="badge mono">{{ tools.orphans }}</span>
        </button>
        <button type="button" class="row faint" title="Delete workflow lands with the editor milestone">
          <PhTrash :size="12" class="tool-icon" aria-hidden="true" />
          <span class="row-label">Trash</span>
          <span class="badge mono">{{ tools.trash }}</span>
        </button>
      </section>
    </div>
  </aside>
</template>

<style scoped>
.explorer {
  display: flex;
  flex-direction: column;
  min-width: 0;
  width: 220px;
  flex: none;
  background: var(--bg-1);
  border-right: 1px solid var(--line-1);
}

.ex-header {
  padding: 12px 12px 0;
}

.ex-eyebrow {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.11em;
  text-transform: uppercase;
  color: var(--faint);
}

.realm-select {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: calc(100% - 20px);
  margin: 9px 10px 2px;
  padding: 6px 9px;
  border-radius: var(--r-m);
  background: var(--bg-2);
  border: 1px solid var(--line-1);
  color: var(--text-1);
  font-size: 12.5px;
  font-weight: 500;
  text-align: left;
  transition: border-color 0.12s ease;
}

.realm-select:hover {
  border-color: var(--line-2);
}

.realm-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.realm-caret {
  flex: none;
  color: var(--faint);
}

.ex-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px 0 12px;
}

.group {
  margin-bottom: 4px;
}

.group-title {
  margin: 8px 12px 2px;
  font-size: 9.5px;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--faint);
}

.row {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  padding: 3px 12px 3px 12px;
  border-radius: var(--r-s);
  color: var(--text-2);
  font-size: 12.5px;
  text-align: left;
}

.row:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.row.active {
  background: var(--sel-bg);
  color: var(--text-1);
}

.row.active .row-label {
  color: var(--gold);
}

.row-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-hint {
  margin-left: 5px;
  color: var(--faint);
  font-size: 10.5px;
}

.check {
  width: 11px;
  flex: none;
  font-size: 9.5px;
  color: var(--gold);
  text-align: center;
}

.badge {
  flex: none;
  font-size: 10px;
  color: var(--faint);
}

.row.active .badge {
  color: var(--gold);
  opacity: 0.85;
}

.twisty {
  flex: none;
  color: var(--text-3);
}

.row.expander:hover .twisty {
  color: var(--text-1);
}

.sub-list {
  padding: 1px 0;
}

.row.sub {
  padding-left: 25px;
}

.row.faint {
  color: var(--text-3);
}

.row.faint:hover {
  color: var(--text-2);
}

.tool-icon {
  flex: none;
  color: var(--text-3);
}

.vicon {
  flex: none;
  width: 15px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-3);
}

.row:hover .vicon {
  color: var(--text-1);
}

.row.active .vicon {
  color: var(--gold);
}

.group-sep {
  height: 1px;
  margin: 6px 12px;
  background: var(--line-1);
}

/* ---- taxonomy tree rows ------------------------------------------- */

.taxrow {
  display: flex;
  align-items: center;
  padding-right: 12px;
}

.tax-caret {
  flex: none;
  width: 16px;
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  color: var(--text-3);
}

.tax-caret:hover {
  color: var(--text-1);
}

.tax-caret-spacer {
  flex: none;
  width: 16px;
}

.taxrow .tax-leaf {
  flex: 1;
  min-width: 0;
  width: auto;
  padding-left: 4px;
  padding-right: 0;
}

.taxrow .tax-leaf .check {
  width: 12px;
  margin-right: 2px;
}
</style>
