<script setup lang="ts">
// Table view (workspace bottom half): the realm as a sortable, filterable
// data grid. The header pills are quick status filters; the funnel opens the
// Filters pane; Columns/⋯ control grid chrome. Clicking a row drives the
// shared selection + inspector.

import { computed, ref } from 'vue'
import {
  PhCaretDown,
  PhColumns,
  PhFunnel,
  PhDotsThree,
  PhMagnifyingGlass,
  PhPlus,
  PhX,
} from '@phosphor-icons/vue'
import type { NodeView } from '../types'
import type { CheckRow, FilterModel } from '../lib/mockRealm'
import { matchesFilters } from '../lib/mockRealm'
import { KIND_ICON, KIND_LABEL } from '../lib/kind'

const props = defineProps<{
  nodes: NodeView[]
  filters: FilterModel
  selectedId: string | null
  statusCounts: CheckRow[]
}>()

const emit = defineEmits<{
  select: [id: string]
  close: []
  toggleStatus: [key: string]
  openFilters: []
  clearFilters: []
  focusSearch: []
}>()

/* ---- columns ----------------------------------------------------------- */

const COLS = [
  { key: 'title', label: 'Name', grow: true },
  { key: 'kind', label: 'Kind' },
  { key: 'status', label: 'Status' },
  { key: 'priority', label: 'Priority' },
  { key: 'qp', label: 'Quest Points', mono: true },
  { key: 'epics', label: 'Epics' },
  { key: 'disciplines', label: 'Disciplines' },
  { key: 'landmark', label: 'Landmark' },
] as const

type ColKey = (typeof COLS)[number]['key']

const hiddenCols = ref<Set<ColKey>>(new Set())
function toggleCol(key: ColKey) {
  const next = new Set(hiddenCols.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  hiddenCols.value = next
}

/* ---- sorting ------------------------------------------------------------ */

const STATUS_ORDER: Record<string, number> = { unstarted: 0, active: 1, blocked: 2, vanquished: 3 }
const KIND_ORDER: Record<string, number> = { card: 0, action: 1, guard: 2, idea: 3 }
const PRIORITY_ORDER: Record<string, number> = { critical: 0, high: 1, medium: 2, low: 3 }

const sort = ref<{ key: ColKey; dir: 1 | -1 } | null>(null)

function sortBy(key: ColKey) {
  if (sort.value?.key === key) {
    if (sort.value.dir === 1) sort.value = { key, dir: -1 }
    else sort.value = null
    return
  }
  sort.value = { key, dir: 1 }
}

function qpOf(n: NodeView): number {
  return n.kind === 'card' ? n.totalQp : (n.questPoints ?? 0)
}

const visible = computed(() =>
  props.nodes.filter((n) => matchesFilters(n, props.filters)),
)

const rows = computed(() => {
  const s = sort.value
  if (!s) return visible.value
  const arr = [...visible.value]
  const cmp = (a: NodeView, b: NodeView): number => {
    const c = (() => {
      switch (s.key) {
        case 'title':
          return a.title.localeCompare(b.title)
        case 'kind':
          return KIND_ORDER[a.kind] - KIND_ORDER[b.kind]
        case 'status':
          return STATUS_ORDER[a.effectiveStatus] - STATUS_ORDER[b.effectiveStatus]
        case 'priority':
          return PRIORITY_ORDER[a.priority] - PRIORITY_ORDER[b.priority]
        case 'qp':
          return qpOf(a) - qpOf(b)
        case 'epics':
          return a.epics.join(' / ').localeCompare(b.epics.join(' / '))
        case 'disciplines':
          return a.disciplines.join(' / ').localeCompare(b.disciplines.join(' / '))
        case 'landmark':
          return (a.landmark ?? '').localeCompare(b.landmark ?? '')
        default:
          return 0
      }
    })()
    return s.dir === 1 ? c : -c
  }
  return arr.sort(cmp)
})

/* ---- status pills --------------------------------------------------------- */

const PILLS = ['unstarted', 'active', 'blocked', 'vanquished'] as const

function pillOn(key: string): boolean {
  return props.filters.statuses.has(key)
}

/* ---- dropdowns -------------------------------------------------------------- */

const openMenu = ref<'cols' | 'more' | null>(null)

function toggleMenu(m: 'cols' | 'more') {
  openMenu.value = openMenu.value === m ? null : m
}

function rowClass(n: NodeView): string {
  return n.effectiveStatus === 'vanquished' ? 'done' : ''
}
</script>

<template>
  <section class="table-pane" aria-label="Table view">
    <header class="table-head">
      <div class="tab-left">
        <span class="view-tab" title="Table view">
          Table View
          <button
            type="button"
            class="tab-close"
            aria-label="Close table view"
            @click="emit('close')"
          >
            <PhX :size="10" aria-hidden="true" />
          </button>
        </span>

        <span class="pill-row" role="group" aria-label="Filter by status">
          <button
            v-for="p in PILLS"
            :key="p"
            type="button"
            class="pill"
            :class="[p, { on: pillOn(p) }]"
            :aria-pressed="pillOn(p)"
            @click="emit('toggleStatus', p)"
          >
            <span class="sdot" :class="`dot-${p}`" aria-hidden="true"></span>
            {{ p[0]!.toUpperCase() + p.slice(1) }}
            <span class="pcount mono">{{ props.statusCounts.find((r) => r.key === p)?.count ?? 0 }}</span>
          </button>
          <button
            type="button"
            class="pill add-pill"
            title="New node — ships with the editor milestone"
            aria-label="New node"
          >
            <PhPlus :size="11" aria-hidden="true" />
          </button>
        </span>
      </div>

      <div class="tab-right">
        <span v-if="visible.length !== nodes.length" class="rowcount mono">
          {{ visible.length }} of {{ nodes.length }}
        </span>
        <span class="rel">
          <button
            type="button"
            class="head-btn"
            :class="{ on: openMenu === 'cols' }"
            title="Columns"
            aria-label="Columns"
            @click="toggleMenu('cols')"
          >
            <PhColumns :size="13" aria-hidden="true" />
          </button>
          <div v-if="openMenu === 'cols'" class="menu-pop" role="menu">
            <div
              v-for="c in COLS"
              :key="c.key"
              class="menu-item"
              role="menuitemcheckbox"
              :aria-checked="!hiddenCols.has(c.key)"
              @click="toggleCol(c.key)"
            >
              <span class="check" aria-hidden="true">{{ hiddenCols.has(c.key) ? '' : '✓' }}</span>
              {{ c.label }}
            </div>
          </div>
        </span>
        <button
          type="button"
          class="head-btn"
          title="Open filters pane"
          aria-label="Open filters"
          @click="emit('openFilters')"
        >
          <PhFunnel :size="13" aria-hidden="true" />
        </button>
        <span class="rel">
          <button
            type="button"
            class="head-btn"
            :class="{ on: openMenu === 'more' }"
            title="More"
            aria-label="More"
            @click="toggleMenu('more')"
          >
            <PhDotsThree :size="15" aria-hidden="true" />
          </button>
          <div v-if="openMenu === 'more'" class="menu-pop right" role="menu">
            <button type="button" class="menu-item" role="menuitem" @click="emit('focusSearch')">
              <PhMagnifyingGlass :size="12" aria-hidden="true" />
              Search filters
            </button>
            <button
              type="button"
              class="menu-item"
              role="menuitem"
              :disabled="visible.length === nodes.length"
              @click="emit('clearFilters')"
            >
              Clear filters
            </button>
            <div class="menu-sep"></div>
            <div class="menu-item disabled" title="Editor milestone">New node from row…</div>
          </div>
        </span>
      </div>
    </header>

    <div class="grid-outer">
      <table class="grid" :class="{ 'has-more': openMenu !== null }">
        <thead>
          <tr>
            <th
              v-for="c in COLS.filter((c) => !hiddenCols.has(c.key))"
              :key="c.key"
              :class="{ sortable: true, sorted: sort?.key === c.key }"
              @click="sortBy(c.key)"
            >
              <span class="th-label">{{ c.label }}</span>
              <PhCaretDown
                v-if="sort?.key === c.key"
                :size="9"
                class="th-caret"
                :class="{ asc: sort.dir === 1 }"
                aria-hidden="true"
              />
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="n in rows"
            :key="n.id"
            :class="[rowClass(n), { selected: n.id === selectedId }]"
            @click="emit('select', n.id)"
          >
            <td class="col-title">
              <span class="kind-tile" :class="n.kind" :title="`${KIND_LABEL[n.kind]} node`">
                <component :is="KIND_ICON[n.kind]" :size="10" aria-hidden="true" />
              </span>
              <span class="t-title" :title="n.title">{{ n.title }}</span>
              <span class="t-id mono">{{ n.id }}</span>
            </td>
            <td class="col-kind mono">{{ KIND_LABEL[n.kind] }}</td>
            <td class="col-status">
              <span class="st" :class="`st-${n.effectiveStatus}`">
                <i class="sdot" :class="`dot-${n.effectiveStatus}`" aria-hidden="true"></i>
                {{ n.effectiveStatus[0]!.toUpperCase() + n.effectiveStatus.slice(1) }}
              </span>
            </td>
            <td class="col-priority">
              <span class="st">
                <i class="pdot" :class="`p-${n.priority}`" aria-hidden="true"></i>
                {{ n.priority[0]!.toUpperCase() + n.priority.slice(1) }}
              </span>
            </td>
            <td class="col-qp mono">
              <span v-if="n.kind === 'card' && n.totalQp > 0" class="qp-val" :title="`${n.totalQp} QP aggregate`">
                {{ n.totalQp }}
              </span>
              <span v-else-if="n.questPoints !== null" class="qp-val">{{ n.questPoints }}</span>
              <span v-else class="qp-empty">—</span>
            </td>
            <td class="col-epics">
              <span class="taglist">
                <span v-for="e in n.epics.slice(0, 2)" :key="e" class="tag tiny" :title="e">
                  {{ e.split('/').pop() }}
                </span>
              </span>
            </td>
            <td class="col-disciplines">
              <span class="taglist">
                <span v-for="d in n.disciplines.slice(0, 2)" :key="d" class="tag disc tiny" :title="d">
                  {{ d.split('/').pop() }}
                </span>
              </span>
            </td>
            <td class="col-landmark" :title="n.landmark ?? ''">
              <span v-if="n.landmark" class="lmark">{{ n.landmark }}</span>
              <span v-else class="lmark none">—</span>
            </td>
          </tr>
          <tr v-if="!rows.length">
            <td :colspan="COLS.length - hiddenCols.size" class="grid-empty">
              <p>No nodes match the current filters.</p>
              <button type="button" class="clear-link" @click="emit('clearFilters')">
                Clear filters
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<style scoped>
.table-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  background: var(--bg-1);
  flex: 1;
}

.table-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 4px 8px 4px 10px;
  border-bottom: 1px solid var(--line-1);
  min-height: 34px;
}

.tab-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  flex-wrap: wrap;
}

.view-tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 4px 3px 8px;
  border-radius: var(--r-m) 0 0 var(--r-m);
  border: 1px solid var(--line-2);
  border-right: 0;
  background: var(--bg-2);
  color: var(--text-1);
  font-size: 11.5px;
  font-weight: 600;
}

.tab-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.tab-close:hover {
  color: var(--text-1);
  background: var(--bg-3);
}

.pill-row {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 9px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  color: var(--text-2);
  font-size: 11px;
  transition: all 0.12s ease;
}

.pill:hover {
  border-color: var(--line-2);
  color: var(--text-1);
}

.pcount {
  font-size: 9.5px;
  color: var(--faint);
}

.pill.on {
  background: var(--chip-active-bg);
  border-color: var(--chip-active-line);
  color: var(--chip-active-fg);
}

.pill.on .pcount {
  color: inherit;
  opacity: 0.8;
}

.pill.unstarted.on {
  background: var(--chip-unstarted-bg);
  border-color: var(--chip-unstarted-line);
  color: var(--chip-unstarted-fg);
}

.pill.blocked.on {
  background: var(--chip-blocked-bg);
  border-color: var(--chip-blocked-line);
  color: var(--chip-blocked-fg);
}

.pill.vanquished.on {
  background: var(--chip-done-bg);
  border-color: var(--chip-done-line);
  color: var(--chip-done-fg);
}

.pill.active.on {
  background: var(--chip-active-bg);
  border-color: var(--chip-active-line);
  color: var(--chip-active-fg);
}

.sdot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex: none;
}

.dot-unstarted {
  background: var(--dot-unstarted);
}

.dot-active {
  background: var(--dot-active);
}

.dot-blocked {
  background: var(--dot-blocked);
}

.dot-vanquished {
  background: var(--dot-done);
}

.add-pill {
  color: var(--faint);
  padding: 2px 7px;
}

.tab-right {
  display: flex;
  align-items: center;
  gap: 3px;
  flex: none;
  margin-left: auto;
}

.rowcount {
  font-size: 10px;
  color: var(--faint);
  margin-right: 6px;
}

.head-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 24px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.head-btn:hover,
.head-btn.on {
  background: var(--bg-2);
  color: var(--text-1);
}

.rel {
  position: relative;
}

.menu-pop {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 30;
  min-width: 168px;
  padding: 4px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  box-shadow: var(--shadow-1);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 100%;
  padding: 4px 8px;
  border-radius: var(--r-s);
  font-size: 12px;
  color: var(--text-2);
  text-align: left;
}

.menu-item:hover:not(.disabled) {
  background: var(--bg-2);
  color: var(--text-1);
}

.menu-item.disabled {
  color: var(--faint);
  cursor: default;
}

.check {
  width: 12px;
  flex: none;
  font-size: 10px;
  color: var(--gold);
}

.menu-sep {
  height: 1px;
  margin: 4px 6px;
  background: var(--line-1);
}

/* ---- grid --------------------------------------------------------------- */

.grid-outer {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.grid {
  width: 100%;
  min-width: 920px;
  border-collapse: separate;
  border-spacing: 0;
  table-layout: fixed;
  font-size: 12px;
}

.grid thead th {
  position: sticky;
  top: 0;
  z-index: 2;
  background: var(--bg-1);
  text-align: left;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--faint);
  padding: 6px 10px;
  border-bottom: 1px solid var(--line-1);
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
}

.grid thead th:hover {
  color: var(--text-2);
}

.th-label {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}

.th-caret {
  color: var(--gold);
}

.th-caret.asc {
  transform: rotate(180deg);
}

.grid td {
  padding: 0 10px;
  height: 30px;
  border-bottom: 1px solid var(--line-0);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

tbody tr {
  cursor: pointer;
}

tbody tr:hover {
  background: var(--bg-2);
}

tbody tr.selected {
  background: var(--sel-bg);
}

tbody tr.selected td:first-child {
  box-shadow: inset 2px 0 0 var(--gold);
}

tr.done .t-title {
  text-decoration: line-through;
  text-decoration-color: var(--line-2);
  color: var(--text-3);
}

/* column widths */
.col-title {
  width: 34%;
}

.col-kind {
  width: 9%;
}

.col-status {
  width: 10%;
}

.col-priority {
  width: 10%;
}

.col-qp {
  width: 9%;
}

.col-epics {
  width: 11%;
}

.col-disciplines {
  width: 11%;
}

.col-landmark {
  width: 13%;
}

.col-title {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 0;
}

.t-title {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.t-id {
  margin-left: auto;
  flex: none;
  font-size: 9px;
  color: var(--faint);
}

.kind-tile {
  flex: none;
  width: 16px;
  height: 16px;
  border-radius: 4px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--badge-ink);
}

.kind-tile.card {
  background: var(--kind-card);
}

.kind-tile.action {
  background: var(--kind-action);
}

.kind-tile.guard {
  background: var(--kind-guard);
}

.kind-tile.idea {
  background: var(--kind-idea);
}

.st {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-2);
}

.st-blocked {
  color: var(--chip-blocked-fg);
}

.st-active {
  color: var(--chip-active-fg);
}

.st-vanquished {
  color: var(--chip-done-fg);
}

.st-unstarted {
  color: var(--chip-unstarted-fg);
}

.pdot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex: none;
}

.p-critical {
  background: #e34c3e;
}

.p-high {
  background: #e0703c;
}

.p-medium {
  background: #d0a03c;
}

.p-low {
  background: var(--dot-unstarted);
}

.qp-val {
  color: var(--gold);
  font-size: 11px;
}

.qp-empty {
  color: var(--faint);
}

.taglist {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  max-width: 100%;
}

.tag {
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  background: var(--inset);
  color: var(--text-2);
  font-size: 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.lmark {
  color: var(--text-2);
}

.lmark.none {
  color: var(--faint);
}

.grid-empty {
  height: 120px;
  text-align: center;
  color: var(--faint);
}

.clear-link {
  color: var(--gold);
}

.clear-link:hover {
  text-decoration: underline;
}
</style>
