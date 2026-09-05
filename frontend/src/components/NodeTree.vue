<script setup lang="ts">
// Node index pane: realm identity, status quick-filters with live counts,
// full-text search, then the collapsible node tree. Every control here is a
// read filter - no writes. The tree stays a flat, cheap list of rows; the
// filter + search state lives in the parent so keyboard navigation can stay
// global.

import { computed, nextTick, ref, watch } from 'vue'
import {
  PhCaretDown,
  PhCaretRight,
  PhMagnifyingGlass,
  PhX,
} from '@phosphor-icons/vue'
import type { TreeRow } from '../types'
import { KIND_ICON, KIND_LABEL, kindOf } from '../lib/kind'

const props = defineProps<{
  rows: TreeRow[]
  selectedId: string | null
  filter: string
  status: 'all' | 'active' | 'blocked' | 'unstarted' | 'vanquished'
  collapsed: Set<string>
  counts: { total: number; active: number; blocked: number; unstarted: number; vanquished: number }
  realmName: string
  realmPath: string
  warnings: string[]
}>()

const emit = defineEmits<{
  select: [id: string]
  toggle: [id: string]
  'update:filter': [value: string]
  'update:status': [value: 'all' | 'active' | 'blocked' | 'unstarted' | 'vanquished']
}>()

const STATUS_PILLS = [
  { key: 'active', label: 'Active' },
  { key: 'blocked', label: 'Blocked' },
  { key: 'unstarted', label: 'Unstarted' },
  { key: 'vanquished', label: 'Vanquished' },
] as const

type StatusKey = (typeof STATUS_PILLS)[number]['key']

function pillCount(key: StatusKey | 'all'): number {
  if (key === 'all') return props.counts.total
  return props.counts[key]
}

function pillActive(key: StatusKey | 'all'): boolean {
  return props.status === key
}

function setStatus(key: StatusKey | 'all') {
  emit('update:status', pillActive(key) ? 'all' : key)
}

function hasQp(row: TreeRow): boolean {
  const n = row.node
  return n.questPoints !== null || (n.kind === 'card' && n.totalQp > 0)
}

function qpValue(row: TreeRow): number {
  return row.node.kind === 'card' ? row.node.totalQp : (row.node.questPoints ?? 0)
}

/* ---- search input + scroll-into-view --------------------------------- */

const filterModel = computed({
  get: () => props.filter,
  set: (v: string) => emit('update:filter', v),
})

const searchRef = ref<HTMLInputElement | null>(null)

function focusSearch() {
  nextTick(() => {
    searchRef.value?.focus()
    searchRef.value?.select()
  })
}

defineExpose({ focusSearch })

const rootRef = ref<HTMLElement | null>(null)

watch(
  () => props.selectedId,
  () => {
    nextTick(() => {
      if (!props.selectedId || !rootRef.value) return
      const el = rootRef.value.querySelector<HTMLElement>(`[data-row-id="${props.selectedId}"]`)
      el?.scrollIntoView({ block: 'nearest' })
    })
  },
)
</script>

<template>
  <aside ref="rootRef" class="tree-pane" aria-label="Node index">
    <div class="pane-top">
      <span class="realm-name" :title="realmPath">{{ realmName }}</span>
      <span class="node-count mono">{{ counts.total }} nodes</span>
    </div>

    <div class="filters" role="group" aria-label="Filter nodes by status">
      <button
        type="button"
        class="pill all"
        :class="{ on: status === 'all' }"
        :aria-pressed="status === 'all'"
        @click="setStatus('all')"
      >
        All <span class="pill-count mono">{{ pillCount('all') }}</span>
      </button>
      <button
        v-for="p in STATUS_PILLS"
        :key="p.key"
        type="button"
        class="pill"
        :class="[p.key, { on: pillActive(p.key) }]"
        :aria-pressed="pillActive(p.key)"
        @click="setStatus(p.key)"
      >
        {{ p.label }}
        <span class="pill-count mono">{{ pillCount(p.key) }}</span>
      </button>
    </div>

    <div class="search">
      <PhMagnifyingGlass class="search-icon" :size="13" aria-hidden="true" />
      <input
        ref="searchRef"
        v-model="filterModel"
        class="realm-filter"
        type="search"
        placeholder="Filter title, id, tag, epic, landmark"
        aria-label="Filter nodes"
        autocomplete="off"
        spellcheck="false"
      />
      <button
        v-if="filter"
        type="button"
        class="clear"
        aria-label="Clear filter"
        @click="emit('update:filter', '')"
      >
        <PhX :size="12" aria-hidden="true" />
      </button>
      <kbd v-else class="hint" aria-hidden="true">/</kbd>
    </div>

    <ul class="tree" role="list">
      <li v-for="row in rows" :key="row.node.id">
        <div
          class="row"
          :class="[row.node.kind, { selected: row.node.id === selectedId }]"
          :data-row-id="row.node.id"
          :style="{ '--depth': row.depth }"
        >
          <button
            v-if="row.hasChildren"
            type="button"
            class="twisty"
            :aria-expanded="!collapsed.has(row.node.id)"
            :aria-label="`${collapsed.has(row.node.id) ? 'Expand' : 'Collapse'} ${row.node.title}`"
            @click="emit('toggle', row.node.id)"
          >
            <PhCaretRight v-if="collapsed.has(row.node.id)" :size="12" aria-hidden="true" />
            <PhCaretDown v-else :size="12" aria-hidden="true" />
          </button>
          <span v-else class="twisty leaf"></span>

          <button
            type="button"
            class="main"
            :class="{ done: row.node.effectiveStatus === 'vanquished' }"
            :aria-current="row.node.id === selectedId ? 'true' : undefined"
            @click="emit('select', row.node.id)"
          >
            <span class="kind-tile" :class="row.node.kind" :title="`${KIND_LABEL[kindOf(row.node)]} node`">
              <component :is="KIND_ICON[row.node.kind]" :size="11" aria-hidden="true" />
            </span>
            <span class="title">{{ row.node.title }}</span>
            <span
              class="dot"
              :class="row.node.effectiveStatus"
              :title="`Status: ${row.node.effectiveStatus}`"
              aria-hidden="true"
            ></span>
            <span class="sr-only">{{ row.node.effectiveStatus }}</span>
          </button>
          <span v-if="hasQp(row)" class="qp mono" :title="`${qpValue(row)} QP`">{{ qpValue(row) }}</span>
        </div>
      </li>
      <li v-if="!rows.length" class="empty">
        <p v-if="filter || status !== 'all'">Nothing matches the current filter.</p>
        <p v-else>This realm has no nodes yet.</p>
      </li>
    </ul>

    <div v-if="warnings.length" class="warnings">
      <p v-for="w in warnings" :key="w" class="warning">{{ w }}</p>
    </div>
  </aside>
</template>

<style scoped>
.tree-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--bg-1);
  border-right: 1px solid var(--line-1);
  min-width: 0;
}

/* ---- pane top: realm identity ---------------------------------------- */

.pane-top {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 14px 0;
}

.realm-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.01em;
  color: var(--text-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-count {
  flex: none;
  font-size: 10.5px;
  color: var(--text-3);
}

/* ---- status quick filters -------------------------------------------- */

.filters {
  display: flex;
  gap: 6px;
  padding: 10px 14px 0;
  overflow-x: auto;
  scrollbar-width: none;
}

.filters::-webkit-scrollbar {
  display: none;
}

.pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex: none;
  padding: 3px 9px 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  color: var(--text-2);
  font-size: 11px;
  background: transparent;
  transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease;
}

.pill:hover {
  border-color: var(--line-2);
  color: var(--text-1);
}

.pill-count {
  font-size: 10px;
  color: var(--faint);
}

.pill.on .pill-count {
  color: inherit;
  opacity: 0.85;
}

/* Active filter state takes the status color so the legend == the filter */
.pill.all.on {
  background: var(--btn-primary-bg);
  border-color: var(--btn-primary-bg);
  color: var(--btn-primary-fg);
}

.pill.active.on {
  background: var(--chip-active-bg);
  border-color: var(--chip-active-line);
  color: var(--chip-active-fg);
}

.pill.blocked.on {
  background: var(--chip-blocked-bg);
  border-color: var(--chip-blocked-line);
  color: var(--chip-blocked-fg);
}

.pill.unstarted.on {
  background: var(--chip-unstarted-bg);
  border-color: var(--chip-unstarted-line);
  color: var(--chip-unstarted-fg);
}

.pill.vanquished.on {
  background: var(--chip-done-bg);
  border-color: var(--chip-done-line);
  color: var(--chip-done-fg);
}

/* ---- search ----------------------------------------------------------- */

.search {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 10px 14px 0;
  padding: 0 8px;
  height: 30px;
  background: var(--inset);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  color: var(--text-3);
  transition: border-color 0.15s ease;
}

.search:focus-within {
  border-color: var(--gold);
}

.search-icon {
  flex: none;
  color: var(--text-3);
}

.search input {
  flex: 1;
  min-width: 0;
  border: 0;
  background: transparent;
  outline: none;
  color: var(--text-1);
  font-size: 12px;
}

.search input::placeholder {
  color: var(--faint);
}

.search input::-webkit-search-cancel-button {
  display: none;
}

.search .clear {
  flex: none;
  display: inline-flex;
  padding: 2px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.search .clear:hover {
  color: var(--text-1);
  background: var(--bg-2);
}

.search .hint {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: var(--r-s);
  border: 1px solid var(--line-1);
  background: var(--bg-2);
  color: var(--faint);
  font-size: 10px;
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}

/* ---- tree ------------------------------------------------------------- */

.tree {
  list-style: none;
  margin: 8px 0 0;
  padding: 0 0 10px;
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1;
  min-height: 0;
}

/* Cheap windowing for large realms: off-screen rows skip layout/paint */
.tree li {
  content-visibility: auto;
  contain-intrinsic-size: 25px;
}

.row {
  display: flex;
  align-items: stretch;
  padding-right: 8px;
  border-left: 2px solid transparent;
}

.row.selected {
  background: var(--sel-bg);
  border-left-color: var(--gold);
}

.twisty {
  flex: none;
  width: 22px;
  align-self: center;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--text-3);
  border-radius: var(--r-s);
  height: 22px;
  margin-left: calc(4px + var(--depth) * 16px);
  transition: color 0.12s ease, transform 0.12s ease;
}

.twisty.leaf {
  visibility: hidden;
}

button.twisty:hover {
  color: var(--text-1);
  background: var(--bg-2);
}

.row .main {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 7px;
  text-align: left;
  padding: 3px 4px 3px 0;
  border-radius: var(--r-s);
  color: var(--text-2);
}

.main:hover {
  color: var(--text-1);
}

.main.done .title {
  text-decoration: line-through;
  text-decoration-color: var(--text-3);
  color: var(--text-3);
}

.main.done .dot {
  background: var(--dot-done);
}

.kind-tile {
  flex: none;
  width: 18px;
  height: 18px;
  border-radius: var(--r-s);
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

.title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12.5px;
}

.dot {
  flex: none;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--dot-unstarted);
}

.dot.active {
  background: var(--dot-active);
}

.dot.blocked {
  background: var(--dot-blocked);
}

.dot.unstarted {
  background: var(--dot-unstarted);
}

.dot.vanquished {
  background: var(--dot-done);
}

.qp {
  flex: none;
  align-self: center;
  color: var(--gold);
  font-size: 10.5px;
  padding-left: 6px;
}

.empty {
  padding: 14px;
  color: var(--faint);
  font-size: 12px;
}

.empty p {
  margin: 0;
}

/* ---- index warnings --------------------------------------------------- */

.warnings {
  border-top: 1px solid var(--line-1);
  padding: 8px 14px 10px;
}

.warning {
  margin: 2px 0;
  color: var(--warn-fg);
  font-size: 11px;
  overflow-wrap: anywhere;
}
</style>
