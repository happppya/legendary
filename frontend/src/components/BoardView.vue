<script setup lang="ts">
// Kanban ("Landmark Matrix" stage 1, doc 05 §5.6): status columns built from
// the current filter view. Clicking a card inspects it (the right inspector
// follows shared selection in the workspace page).

import { computed } from 'vue'
import { PhPlus } from '@phosphor-icons/vue'
import type { NodeView } from '../types'
import { KIND_ICON, KIND_LABEL } from '../lib/kind'

const props = defineProps<{ nodes: NodeView[]; selectedId: string | null }>()
const emit = defineEmits<{ select: [id: string] }>()

const STATUSES = [
  { key: 'unstarted', label: 'Unstarted' },
  { key: 'active', label: 'Active' },
  { key: 'blocked', label: 'Blocked' },
  { key: 'vanquished', label: 'Vanquished' },
] as const

const columns = computed(() =>
  STATUSES.map((s) => ({
    ...s,
    cards: props.nodes.filter((n) => n.effectiveStatus === s.key),
  })),
)

function cardTitle(n: NodeView): string {
  return n.title
}

function qpOf(n: NodeView): number | null {
  return n.kind === 'card' ? (n.totalQp > 0 ? n.totalQp : null) : n.questPoints
}
</script>

<template>
  <section class="board" aria-label="Board view">
    <div class="board-cols">
      <div v-for="col in columns" :key="col.key" class="bcol" :class="col.key">
        <header class="bcol-head">
          <span class="bcol-title">
            <i class="sdot" :class="`dot-${col.key}`" aria-hidden="true"></i>
            {{ col.label }}
          </span>
          <span class="bcol-count mono">{{ col.cards.length }}</span>
          <button type="button" class="bcol-add" title="New node (editor milestone)" aria-label="Add node">
            <PhPlus :size="12" aria-hidden="true" />
          </button>
        </header>
        <div class="bcol-body">
          <button
            v-for="n in col.cards"
            :key="n.id"
            type="button"
            class="card"
            :class="[n.kind, { sel: n.id === selectedId, done: n.effectiveStatus === 'vanquished' }]"
            :title="`${KIND_LABEL[n.kind]} — ${n.id}`"
            @click="emit('select', n.id)"
          >
            <span class="card-top">
              <span class="kind-tile" :class="n.kind">
                <component :is="KIND_ICON[n.kind]" :size="10" aria-hidden="true" />
              </span>
              <span class="card-id mono">{{ n.id }}</span>
            </span>
            <span class="card-title">{{ cardTitle(n) }}</span>
            <span v-if="n.epics.length" class="card-epics">
              <span v-for="e in n.epics.slice(0, 2)" :key="e" class="chip tiny" :title="e">
                {{ e.split('/').pop() }}
              </span>
            </span>
            <span class="card-foot">
              <span v-if="qpOf(n) !== null" class="qp mono" :title="n.kind === 'card' ? 'aggregate QP' : 'quest points'">
                {{ qpOf(n) }} QP
              </span>
              <span v-if="n.blocked" class="bmark" title="Blocked by an unfinished prerequisite">blocked</span>
            </span>
          </button>
          <p v-if="!col.cards.length" class="bcol-empty">No cards</p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.board {
  min-width: 0;
  min-height: 0;
  flex: 1;
  overflow: hidden;
  background: var(--bg-0);
}

.board-cols {
  display: flex;
  gap: 10px;
  height: 100%;
  padding: 10px;
  overflow-x: auto;
}

.bcol {
  display: flex;
  flex-direction: column;
  flex: 1 1 0;
  min-width: 240px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-l);
  overflow: hidden;
}

.bcol-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-bottom: 1px solid var(--line-1);
}

.bcol-title {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-2);
}

.sdot {
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

.dot-vanquished {
  background: var(--dot-done);
}

.bcol-count {
  font-size: 10px;
  color: var(--faint);
}

.bcol-add {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.bcol-add:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.bcol-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.card {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 5px;
  padding: 8px 10px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--bg-2);
  text-align: left;
  color: var(--text-1);
  transition: border-color 0.12s ease, transform 0.12s ease, box-shadow 0.12s ease;
}

.card:hover {
  border-color: var(--line-2);
}

.card.sel {
  border-color: var(--gold);
  box-shadow: 0 0 0 1px var(--gold);
}

.card.done .card-title {
  text-decoration: line-through;
  color: var(--text-3);
}

.card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.kind-tile {
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

.card-id {
  font-size: 9px;
  color: var(--faint);
}

.card-title {
  font-size: 12.5px;
  line-height: 1.35;
  overflow-wrap: anywhere;
}

.card-epics {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.chip {
  padding: 1px 7px;
  border-radius: 999px;
  font-size: 9.5px;
  color: var(--text-2);
  border: 1px solid var(--line-1);
}

.card-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.qp {
  font-size: 10px;
  color: var(--gold);
}

.bmark {
  font-size: 9px;
  color: var(--chip-blocked-fg);
  border: 1px solid var(--chip-blocked-line);
  background: var(--chip-blocked-bg);
  border-radius: 999px;
  padding: 0 6px;
}

.bcol-empty {
  margin: 8px auto;
  font-size: 11px;
  color: var(--faint);
}
</style>
