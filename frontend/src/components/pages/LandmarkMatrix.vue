<script setup lang="ts">
// The Landmark Matrix (doc 05 §5.6): two modes over the same data.
//   • Kanban — status columns (Unstarted/Active/Blocked/Vanquished) of
//     Action/Guard/Idea nodes grouped by effective (explicit or inherited)
//     Landmark.
//   • Swimlanes — matrix crossing Disciplines or Epics with status columns.
// Landmark sections fold by default when there are many; all rows come from
// the current filter view, so the explorer filter set applies here too.
// Clicking a card inspects it via the shared selection.

import { computed, ref } from 'vue'
import { PhCaretDown, PhCaretRight, PhTrash } from '@phosphor-icons/vue'
import type { NodeView } from '../../types'
import { KIND_ICON, KIND_LABEL, isContainerKind } from '../../lib/kind'
import { STATUSES } from '../../lib/status'
import { qpOf } from '../../lib/node'

type MatrixMode = 'landmarks' | 'disciplines' | 'epics'

const props = defineProps<{
  nodes: NodeView[]
  selectedId: string | null
  /** Mutations are desktop-only; the browser fixture preview hides them. */
  canMutate: boolean
}>()
const emit = defineEmits<{
  select: [id: string]
  /** Delete a node (App routes through the confirmation modal). */
  'delete-node': [id: string]
}>()

const mode = ref<MatrixMode>('landmarks')

/** Leaf tasks only (Cards and Genres are containers, not columns' residents). */
const leafTasks = computed(() => props.nodes.filter((n) => !isContainerKind(n.kind)))

/* ---- kanban grouped by effective landmark ------------------------------ */

const landmarkGroups = computed(() => {
  const m = new Map<string, NodeView[]>()
  for (const n of leafTasks.value) {
    const key = n.landmark ?? '— no landmark —'
    const list = m.get(key) ?? []
    list.push(n)
    m.set(key, list)
  }
  return [...m.entries()]
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([landmark, items]) => ({
      landmark,
      columns: STATUSES.map((s) => ({
        ...s,
        cards: items.filter((n) => n.effectiveStatus === s.key),
      })),
      total: items.length,
    }))
})

/** Sections with many landmarks start folded. */
const folded = ref<Set<string>>(new Set())
function toggleFold(landmark: string) {
  const next = new Set(folded.value)
  if (next.has(landmark)) next.delete(landmark)
  else next.add(landmark)
  folded.value = next
}

/* ---- domain swimlanes --------------------------------------------------- */

interface SwimRow {
  label: string
  columns: { key: string; label: string; cards: NodeView[] }[]
  total: number
}

function branchSwimlanes(field: 'disciplines' | 'epics'): SwimRow[] {
  const m = new Map<string, NodeView[]>()
  for (const n of leafTasks.value) {
    const keys = n[field].length ? n[field] : ['— unassigned —']
    for (const k of keys) {
      const list = m.get(k) ?? []
      list.push(n)
      m.set(k, list)
    }
  }
  return [...m.entries()]
    .sort((a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0]))
    .map(([label, items]) => ({
      label,
      columns: STATUSES.map((s) => ({
        key: s.key,
        label: s.label,
        cards: items.filter((n) => n.effectiveStatus === s.key),
      })),
      total: items.length,
    }))
}

const swimRows = computed<SwimRow[]>(() =>
  mode.value === 'landmarks' ? [] : branchSwimlanes(mode.value),
)

const MODES: { key: MatrixMode; label: string }[] = [
  { key: 'landmarks', label: 'Landmarks' },
  { key: 'disciplines', label: 'Disciplines' },
  { key: 'epics', label: 'Epics' },
]

function shortBranch(p: string): string {
  const parts = p.split('/')
  return parts[parts.length - 1] as string
}
</script>

<template>
  <section class="matrix" aria-label="Landmark Matrix">
    <header class="mx-head">
      <h1 class="mx-title">Landmark Matrix</h1>
      <div class="mx-modes" role="tablist" aria-label="Matrix mode">
        <button
          v-for="m in MODES"
          :key="m.key"
          type="button"
          class="mx-mode"
          :class="{ on: mode === m.key }"
          role="tab"
          :aria-selected="mode === m.key"
          @click="mode = m.key"
        >
          {{ m.label }}
        </button>
      </div>
    </header>

    <!-- kanban grouped by effective landmark -->
    <div v-if="mode === 'landmarks'" class="mx-body">
      <section v-for="g in landmarkGroups" :key="g.landmark" class="lm-group">
        <button
          type="button"
          class="lm-head"
          :aria-expanded="!folded.has(g.landmark)"
          @click="toggleFold(g.landmark)"
        >
          <PhCaretRight v-if="folded.has(g.landmark)" :size="11" class="twisty" aria-hidden="true" />
          <PhCaretDown v-else :size="11" class="twisty" aria-hidden="true" />
          <span class="lm-name">{{ g.landmark }}</span>
          <span class="badge mono">{{ g.total }}</span>
        </button>
        <div v-if="!folded.has(g.landmark)" class="lm-cols">
          <div v-for="col in g.columns" :key="col.key" class="mx-col" :class="col.key">
            <header class="mx-col-head">
              <span class="mx-col-title">
                <i class="sdot" :class="`dot-${col.key}`" aria-hidden="true"></i>
                {{ col.label }}
              </span>
              <span class="badge mono">{{ col.cards.length }}</span>
            </header>
            <div class="mx-col-body">
              <div
                v-for="n in col.cards"
                :key="n.id"
                class="card"
                :class="[n.kind, { sel: n.id === selectedId, done: n.effectiveStatus === 'vanquished' }]"
                :title="`${KIND_LABEL[n.kind]} — ${n.id}`"
                role="button"
                tabindex="0"
                @click="emit('select', n.id)"
                @keydown.enter.prevent="emit('select', n.id)"
              >
                <span class="card-top">
                  <span class="kind-tile" :class="n.kind">
                    <component :is="KIND_ICON[n.kind]" :size="10" aria-hidden="true" />
                  </span>
                  <span class="card-id mono">{{ n.id }}</span>
                  <span v-if="canMutate" class="card-acts">
                    <button
                      type="button"
                      class="card-act danger"
                      title="Delete node"
                      aria-label="Delete node"
                      @click.stop="emit('delete-node', n.id)"
                    >
                      <PhTrash :size="10" aria-hidden="true" />
                    </button>
                  </span>
                  <span v-if="qpOf(n) !== null" class="qp mono">{{ qpOf(n) }}</span>
                </span>
                <span class="card-title">{{ n.title }}</span>
              </div>
              <p v-if="!col.cards.length" class="col-empty">—</p>
            </div>
          </div>
        </div>
      </section>
      <p v-if="!landmarkGroups.length" class="mx-empty">No leaf tasks match the current filters.</p>
    </div>

    <!-- domain swimlanes -->
    <div v-else class="mx-body">
      <section class="swim-matrix">
        <div class="swim-corner">
          <span class="swim-corner-label">{{ mode === 'disciplines' ? 'Discipline' : 'Epic' }} / status</span>
        </div>
        <header v-for="s in STATUSES" :key="s.key" class="swim-col-head">
          <i class="sdot" :class="`dot-${s.key}`" aria-hidden="true"></i>
          {{ s.label }}
        </header>

        <template v-for="row in swimRows" :key="row.label">
          <div class="swim-row-label">
            <span class="swim-name" :title="row.label">{{ shortBranch(row.label) }}</span>
            <span class="badge mono">{{ row.total }}</span>
          </div>
          <div v-for="col in row.columns" :key="row.label + col.key" class="swim-cell">
            <button
              v-for="n in col.cards"
              :key="n.id"
              type="button"
              class="card compact"
              :class="{ sel: n.id === selectedId, done: n.effectiveStatus === 'vanquished' }"
              :title="`${KIND_LABEL[n.kind]} — ${n.title}`"
              @click="emit('select', n.id)"
            >
              <span class="card-id mono">{{ n.id }}</span>
              <span class="swim-title">{{ n.title }}</span>
            </button>
          </div>
        </template>
      </section>
      <p v-if="!swimRows.length" class="mx-empty">No leaf tasks match the current filters.</p>
    </div>
  </section>
</template>

<style scoped>
.matrix {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-0);
}

.mx-head {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px 8px;
}

.mx-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.mx-modes {
  display: flex;
  gap: 4px;
}

.mx-mode {
  padding: 3px 12px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  background: transparent;
  color: var(--text-2);
  font-size: 11.5px;
}

.mx-mode:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.mx-mode.on {
  background: var(--inset);
  border-color: var(--gold);
  color: var(--gold);
}

.mx-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 4px 16px 16px;
}

/* ---- landmark groups ---------------------------------------------------- */

.lm-group {
  margin-bottom: 16px;
}

.lm-head {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 3px 8px;
  border-radius: var(--r-m);
  color: var(--text-1);
  font-size: 13px;
  font-weight: 600;
}

.lm-head:hover {
  background: var(--bg-2);
}

.lm-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.twisty {
  color: var(--text-3);
}

.badge {
  font-size: 10px;
  color: var(--faint);
}

.lm-cols {
  display: flex;
  gap: 8px;
  margin-top: 6px;
}

.mx-col {
  flex: 1 1 0;
  min-width: 150px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  overflow: hidden;
}

.mx-col-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border-bottom: 1px solid var(--line-1);
}

.mx-col-title {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--text-2);
}

.sdot {
  width: 6px;
  height: 6px;
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

.dot-vanquished {
  background: var(--dot-done);
}

.mx-col-body {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 6px;
  min-height: 40px;
}

.col-empty {
  margin: 4px auto;
  font-size: 10.5px;
  color: var(--faint);
}

/* ---- cards --------------------------------------------------------------- */

.card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 9px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--bg-2);
  text-align: left;
  color: var(--text-1);
}

.card:hover {
  border-color: var(--line-2);
}

.card.sel {
  border-color: var(--gold);
  box-shadow: 0 0 0 1px var(--gold);
}

.card-acts {
  display: none;
  align-items: center;
  margin-left: auto;
}

.card:hover .card-acts,
.card:focus-within .card-acts {
  display: inline-flex;
}

.card-act {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 17px;
  height: 17px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.card-act.danger:hover {
  background: var(--err-bg);
  color: var(--err-fg);
}

.card.done .card-title,
.card.done .swim-title {
  text-decoration: line-through;
  color: var(--text-3);
}

.card-top {
  display: flex;
  align-items: center;
  gap: 6px;
}

.kind-tile {
  width: 15px;
  height: 15px;
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

.card-id {
  font-size: 9px;
  color: var(--faint);
  flex: 1;
}

.qp {
  font-size: 9.5px;
  color: var(--gold);
}

.card-title {
  font-size: 12px;
  line-height: 1.35;
  overflow-wrap: anywhere;
}

/* ---- swimlanes ------------------------------------------------------------ */

.swim-matrix {
  display: grid;
  grid-template-columns: minmax(140px, 220px) repeat(4, 1fr);
  gap: 6px;
}

.swim-corner {
  display: flex;
  align-items: flex-end;
}

.swim-corner-label {
  font-size: 9.5px;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--faint);
}

.swim-col-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--text-2);
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-s);
}

.swim-row-label {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-s);
}

.swim-name {
  font-size: 11.5px;
  color: var(--text-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.swim-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 30px;
}

.card.compact {
  flex-direction: row;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
}

.swim-title {
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mx-empty {
  margin: 24px auto;
  font-size: 12.5px;
  color: var(--faint);
}
</style>
