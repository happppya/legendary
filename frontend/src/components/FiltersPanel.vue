<script setup lang="ts">
// Secondary left pane: a checklist of live filters (kind, status, priority,
// epic) plus text search, mirroring the "Filters" pane of the reference.
// All checkboxes are read filters - nothing writes.

import { computed, nextTick, ref, watch } from 'vue'
import { PhMagnifyingGlass, PhX } from '@phosphor-icons/vue'
import type { CheckRow, FilterModel } from '../lib/mockRealm'

const props = defineProps<{
  filters: FilterModel
  kindRows: CheckRow[]
  statusRows: CheckRow[]
  priorityRows: CheckRow[]
  epicRows: CheckRow[]
  focusSignal?: number
}>()

const searchEl = ref<HTMLInputElement | null>(null)
watch(
  () => props.focusSignal ?? 0,
  () => {
    nextTick(() => {
      searchEl.value?.focus()
      searchEl.value?.select()
    })
  },
)

const emit = defineEmits<{
  toggle: [cat: string, key: string]
  clear: []
  close: []
  'update:search': [value: string]
}>()

const activeCount = computed(
  () =>
    props.filters.kinds.size +
    props.filters.statuses.size +
    props.filters.priorities.size +
    props.filters.epics.size +
    props.filters.disciplines.size +
    props.filters.landmarks.size,
)

const STATUS_DOT: Record<string, string> = {
  unstarted: 'dot-unstarted',
  active: 'dot-active',
  blocked: 'dot-blocked',
  vanquished: 'dot-done',
}

const PRIORITY_DOT: Record<string, string> = {
  critical: 'prio-critical',
  high: 'prio-high',
  medium: 'prio-medium',
  low: 'prio-low',
}

function checked(cat: string, key: string): boolean {
  const f = props.filters as unknown as Record<string, Set<string>>
  return f[cat]?.has(key) ?? false
}

const searchModel = computed({
  get: () => props.filters.search,
  set: (v: string) => emit('update:search', v),
})
</script>

<template>
  <aside class="filters" aria-label="Filters">
    <header class="filters-header">
      <span class="eyebrow">Filters</span>
      <button
        type="button"
        class="icon-btn"
        title="Hide filters"
        aria-label="Hide filters"
        @click="emit('close')"
      >
        <PhX :size="12" aria-hidden="true" />
      </button>
    </header>

    <div class="filter-search">
      <PhMagnifyingGlass class="search-icon" :size="12" aria-hidden="true" />
      <input
        ref="searchEl"
        v-model="searchModel"
        class="search-input"
        type="search"
        placeholder="Search filters…"
        aria-label="Search nodes"
        spellcheck="false"
      />
    </div>

    <div class="filter-body">
      <section class="fgroup">
        <h2 class="fgroup-title">Kind</h2>
        <button
          v-for="row in kindRows"
          :key="row.key"
          type="button"
          class="frow"
          :class="{ on: checked('kinds', row.key) }"
          @click="emit('toggle', 'kinds', row.key)"
        >
          <span class="fbox" aria-hidden="true"></span>
          <span class="flabel">{{ row.label }}</span>
          <span class="fcount mono">{{ row.count }}</span>
        </button>
      </section>

      <section class="fgroup">
        <h2 class="fgroup-title">Status</h2>
        <button
          v-for="row in statusRows"
          :key="row.key"
          type="button"
          class="frow"
          :class="{ on: checked('statuses', row.key) }"
          @click="emit('toggle', 'statuses', row.key)"
        >
          <span class="fbox" aria-hidden="true"></span>
          <span class="sdot" :class="STATUS_DOT[row.key]" aria-hidden="true"></span>
          <span class="flabel">{{ row.label }}</span>
          <span class="fcount mono">{{ row.count }}</span>
        </button>
      </section>

      <section class="fgroup">
        <h2 class="fgroup-title">Priority</h2>
        <button
          v-for="row in priorityRows"
          :key="row.key"
          type="button"
          class="frow"
          :class="{ on: checked('priorities', row.key) }"
          @click="emit('toggle', 'priorities', row.key)"
        >
          <span class="fbox" aria-hidden="true"></span>
          <span class="pdot" :class="PRIORITY_DOT[row.key]" aria-hidden="true"></span>
          <span class="flabel">{{ row.label }}</span>
          <span class="fcount mono">{{ row.count }}</span>
        </button>
      </section>

      <section class="fgroup">
        <h2 class="fgroup-title">Epics</h2>
        <button
          v-for="row in epicRows"
          :key="row.key"
          type="button"
          class="frow"
          :class="{ on: checked('epics', row.key) }"
          @click="emit('toggle', 'epics', row.key)"
        >
          <span class="fbox" aria-hidden="true"></span>
          <span class="flabel" :title="row.label">{{ row.label }}</span>
          <span class="fcount mono">{{ row.count }}</span>
        </button>
      </section>
    </div>

    <footer class="filters-foot">
      <button
        type="button"
        class="clear"
        :disabled="activeCount === 0 && !filters.search"
        @click="emit('clear')"
      >
        Clear filters
      </button>
      <span v-if="activeCount > 0" class="active mono">{{ activeCount }} active</span>
      <span v-else class="active none mono">all nodes</span>
    </footer>
  </aside>
</template>

<style scoped>
.filters {
  display: flex;
  flex-direction: column;
  min-width: 0;
  width: 228px;
  flex: none;
  background: var(--bg-1);
  border-right: 1px solid var(--line-1);
}

.filters-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 10px 0 12px;
}

.eyebrow {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.11em;
  text-transform: uppercase;
  color: var(--faint);
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.icon-btn:hover {
  color: var(--text-1);
  background: var(--bg-2);
}

.filter-search {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 9px 10px 0;
  padding: 0 8px;
  height: 28px;
  border-radius: var(--r-m);
  background: var(--inset);
  border: 1px solid var(--line-1);
  color: var(--text-3);
}

.filter-search:focus-within {
  border-color: var(--gold);
}

.search-icon {
  flex: none;
}

.search-input {
  flex: 1;
  min-width: 0;
  border: 0;
  background: transparent;
  outline: none;
  color: var(--text-1);
  font-size: 12px;
}

.search-input::placeholder {
  color: var(--faint);
}

.search-input::-webkit-search-cancel-button {
  display: none;
}

.filter-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 6px 0 10px;
}

.fgroup {
  margin-bottom: 2px;
}

.fgroup-title {
  margin: 10px 12px 2px;
  font-size: 9.5px;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--faint);
}

.frow {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 3px 12px;
  border-radius: var(--r-s);
  color: var(--text-2);
  font-size: 12.5px;
  text-align: left;
}

.frow:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.frow.on {
  color: var(--text-1);
}

.fbox {
  flex: none;
  width: 13px;
  height: 13px;
  border-radius: 3px;
  border: 1px solid var(--line-2);
  background: var(--bg-0);
  position: relative;
}

.frow.on .fbox {
  border-color: var(--gold);
  background: var(--gold);
}

.frow.on .fbox::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 1px;
  width: 3px;
  height: 7px;
  border: solid var(--on-gold);
  border-width: 0 1.5px 1.5px 0;
  transform: rotate(45deg);
}

.flabel {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fcount {
  flex: none;
  font-size: 10px;
  color: var(--faint);
}

.frow.on .fcount {
  color: var(--gold);
  opacity: 0.85;
}

.sdot,
.pdot {
  flex: none;
  width: 7px;
  height: 7px;
  border-radius: 50%;
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

.dot-done {
  background: var(--dot-done);
}

.prio-critical {
  background: #e34c3e;
}

.prio-high {
  background: #e0703c;
}

.prio-medium {
  background: #d0a03c;
}

.prio-low {
  background: var(--dot-unstarted);
}

.filters-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 12px 9px;
  border-top: 1px solid var(--line-1);
}

.clear {
  color: var(--gold);
  font-size: 11.5px;
  padding: 2px 4px;
  border-radius: var(--r-s);
}

.clear:hover:not(:disabled) {
  background: var(--sel-bg);
}

.clear:disabled {
  color: var(--faint);
  cursor: default;
}

.active {
  font-size: 10px;
}

.active.none {
  color: var(--faint);
}
</style>
