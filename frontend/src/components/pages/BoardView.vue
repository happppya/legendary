<script setup lang="ts">
// Kanban ("Landmark Matrix" stage 1, doc 05 §5.6): status columns built from
// the current filter view. Clicking a card inspects it (the right inspector
// follows shared selection in the workspace page).

import { computed } from 'vue'
import { PhPencilSimple, PhPlus, PhTrash } from '@phosphor-icons/vue'
import type { NodeView } from '../../types'
import { KIND_ICON, KIND_LABEL, isContainerKind } from '../../lib/kind'
import { STATUSES } from '../../lib/status'
import { qpOf } from '../../lib/node'

const props = defineProps<{
  nodes: NodeView[]
  selectedId: string | null
  /** Mutations are desktop-only; the browser fixture preview hides them. */
  canMutate: boolean
}>()
const emit = defineEmits<{
  select: [id: string]
  /** Create a node seeded with the column's status as its initial one. */
  create: [status: string]
  /** Edit (rename / notes) the node — opens the shared edit flow. */
  edit: [id: string]
  /** Delete the node (App routes through the confirmation modal). */
  'delete-node': [id: string]
}>()

const columns = computed(() =>
  STATUSES.map((s) => ({
    ...s,
    cards: props.nodes.filter((n) => n.effectiveStatus === s.key),
  })),
)

function cardTitle(n: NodeView): string {
  return n.title
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
          <button
            v-if="canMutate"
            type="button"
            class="bcol-add"
            title="New node in this column"
            aria-label="Add node"
            @click="emit('create', col.key)"
          >
            <PhPlus :size="12" aria-hidden="true" />
          </button>
        </header>
        <div class="bcol-body">
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
                  class="card-act"
                  title="Edit title & notes"
                  aria-label="Edit node"
                  @click.stop="emit('edit', n.id)"
                >
                  <PhPencilSimple :size="10" aria-hidden="true" />
                </button>
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
            </span>
            <span class="card-title">{{ cardTitle(n) }}</span>
            <span v-if="n.epics.length" class="card-epics">
              <span v-for="e in n.epics.slice(0, 2)" :key="e" class="chip tiny" :title="e">
                {{ e.split('/').pop() }}
              </span>
            </span>
            <span class="card-foot">
              <span v-if="qpOf(n) !== null" class="qp mono" :title="isContainerKind(n.kind) ? 'aggregate QP' : 'quest points'">
                {{ qpOf(n) }} QP
              </span>
              <span v-if="n.blocked" class="bmark" title="Blocked by an unfinished prerequisite">blocked</span>
            </span>
          </div>
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
  cursor: pointer;
  transition: border-color 0.12s ease, transform 0.12s ease, box-shadow 0.12s ease;
}

.card-acts {
  display: none;
  align-items: center;
  gap: 2px;
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
  width: 18px;
  height: 18px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.card-act:hover {
  background: var(--bg-3);
  color: var(--text-1);
}

.card-act.danger:hover {
  background: var(--err-bg);
  color: var(--err-fg);
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

.kind-tile.genre {
  background: var(--kind-genre);
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
