<script setup lang="ts">
// Mutation bar of the detail sheet (doc 04 §4.3/§4.5 UI Actions): the only
// place a node can be vanquished, deleted, wrapped or re-parented. Everything
// funnels to App.vue, which owns dialogs + IPC; in the read-only browser
// preview the bar shows a hint instead of destructive controls.

import { computed } from 'vue'
import { PhCrown, PhCornersOut, PhPlusSquare, PhShuffle, PhTrash } from '@phosphor-icons/vue'
import type { NodeView } from '../../../types'

const props = defineProps<{
  node: NodeView
  /** False in the browser fixture preview (mutations require the desktop). */
  canMutate: boolean
}>()

const emit = defineEmits<{
  vanquish: []
  'delete-subgraph': []
  wrap: []
  'reparent-root': []
  'move-subgraph': []
}>()

const isCard = computed(() => props.node.kind === 'card')
const isRoot = computed(() => props.node.parent === null)

/** For a Card: hint line explaining why direct vanquish is gated. */
const vanquishHint = computed(() =>
  isCard.value
    ? props.node.effectiveStatus === 'vanquished'
      ? 'Already vanquished'
      : props.node.blocked
        ? 'Resolve blocked prerequisites first'
        : null
    : null,
)
</script>

<template>
  <section class="actions" aria-label="Node actions">
    <h2 class="section-title">Actions</h2>

    <template v-if="canMutate">
      <div class="action-grid">
        <button
          v-if="isCard"
          type="button"
          class="act act-vanquish"
          :title="vanquishHint ?? 'Mark vanquished (cascade with confirmation)'"
          :disabled="node.effectiveStatus === 'vanquished'"
          @click="emit('vanquish')"
        >
          <PhCrown :size="13" aria-hidden="true" />
          Vanquish
        </button>

        <button
          type="button"
          class="act"
          title="Wrap this node in a new Card group (Promote to Card)"
          :disabled="isCard"
          @click="emit('wrap')"
        >
          <PhPlusSquare :size="13" aria-hidden="true" />
          Wrap in Card
        </button>

        <button
          type="button"
          class="act"
          title="Move this node (and its subtree) under another Card"
          @click="emit('move-subgraph')"
        >
          <PhShuffle :size="13" aria-hidden="true" />
          Move to…
        </button>

        <button
          type="button"
          class="act"
          title="Promote to a root node (parent: null)"
          :disabled="isRoot"
          @click="emit('reparent-root')"
        >
          <PhCornersOut :size="13" aria-hidden="true" />
          Move to root
        </button>

        <button
          type="button"
          class="act act-danger"
          title="Delete this node (recursive delete asks for confirmation)"
          @click="emit('delete-subgraph')"
        >
          <PhTrash :size="13" aria-hidden="true" />
          Delete…
        </button>
      </div>

      <p v-if="isCard && vanquishHint" class="act-note">{{ vanquishHint }}</p>
    </template>

    <p v-else class="act-note">
      Mutations need the desktop shell — the browser preview is read-only.
    </p>
  </section>
</template>

<style scoped>
.actions {
  margin-top: 30px;
  padding-top: 18px;
  border-top: 1px solid var(--line-1);
}

.section-title {
  margin: 0 0 12px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--faint);
  font-weight: 600;
}

.action-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.act {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 11px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--bg-1);
  color: var(--text-2);
  font-size: 11.5px;
}

.act:hover:not(:disabled) {
  border-color: var(--gold);
  color: var(--gold);
}

.act:disabled {
  opacity: 0.45;
  cursor: default;
}

.act-vanquish {
  color: var(--gold);
  border-color: var(--line-2);
}

.act-vanquish:hover:not(:disabled) {
  background: var(--inset);
}

.act-danger:hover:not(:disabled) {
  border-color: var(--err-line);
  color: var(--err-fg);
}

.act-note {
  margin: 10px 0 0;
  font-size: 11.5px;
  color: var(--faint);
}
</style>
