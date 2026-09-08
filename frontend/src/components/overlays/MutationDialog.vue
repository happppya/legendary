<script setup lang="ts">
// Mutation confirmation overlay (doc 04 §4.3 UI Action / §4.5 warning modal).
// Two flavours:
//   • vanquish  — "Card contains N active tasks. Force complete all child
//     tasks?" before a cascade vanquish reuses the engine's leaf count.
//   • delete    — warns with the total count of affected descendant nodes
//     before a recursive subgraph delete.
// Both resolve through `confirm` (emit) or are dismissed via Escape/scrim/×.
// The busy flag shows "Working…" while the IPC round-trip is in flight.

import { onBeforeUnmount, onMounted } from 'vue'
import { PhCrown, PhTrash, PhWarning } from '@phosphor-icons/vue'
import type { NodeView } from '../../types'

export interface MutationRequest {
  flavour: 'vanquish' | 'delete'
  /** Node the action targets (cascade vanquish / subgraph delete root). */
  node: NodeView
  /** Engine-derived count: unvanquished leaf tasks or affected descendants. */
  count: number
  /** Extra context lines shown under the headline. */
  notes?: string[]
}

const props = defineProps<{ request: MutationRequest; busy: boolean }>()

const emit = defineEmits<{ close: []; confirm: [] }>()

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

const title = props.request.flavour === 'vanquish' ? 'Vanquish Card' : 'Delete Node'
</script>

<template>
  <div
    class="dlg-layer"
    role="dialog"
    aria-modal="true"
    :aria-label="request.flavour === 'vanquish' ? 'Force-complete card' : 'Delete node'"
  >
    <div class="dlg-scrim" @click="emit('close')"></div>
    <div class="dlg-box" :class="request.flavour">
      <span class="dlg-glyph" aria-hidden="true">
        <PhCrown v-if="request.flavour === 'vanquish'" :size="16" />
        <PhTrash v-else :size="16" />
      </span>

      <h2 class="dlg-title">{{ title }}</h2>

      <p class="dlg-target">
        <span class="mono id">{{ request.node.id }}</span>
        <span class="t">{{ request.node.title }}</span>
      </p>

      <!-- Vanquish cascade prompt (doc 04 §4.3, verbatim prompt). -->
      <p v-if="request.flavour === 'vanquish'" class="dlg-lede">
        Card contains <strong>{{ request.count }}</strong> active
        {{ request.count === 1 ? 'task' : 'tasks' }}.
        Force complete all child tasks?
      </p>

      <!-- Recursive delete warning (doc 04 §4.5). -->
      <p v-else class="dlg-lede">
        This will permanently delete <strong>{{ request.count }}</strong>
        {{ request.count === 1 ? 'node' : 'nodes' }} from disk (the node plus
        its whole subgraph).
      </p>

      <ul v-if="request.notes?.length" class="dlg-notes">
        <li v-for="(n, i) in request.notes" :key="i">{{ n }}</li>
      </ul>

      <p v-if="request.flavour === 'vanquish'" class="dlg-warn">
        <PhWarning :size="13" aria-hidden="true" />
        Every descendant leaf Action, Guard and Idea is marked
        <span class="mono">vanquished</span> with a completion timestamp.
      </p>
      <p v-else class="dlg-warn">
        <PhWarning :size="13" aria-hidden="true" />
        Dangling <span class="mono">blocked_by</span> references on survivors
        are stripped automatically.
      </p>

      <div class="dlg-actions">
        <button type="button" class="btn-ghost" :disabled="busy" @click="emit('close')">
          Cancel
        </button>
        <button
          type="button"
          class="btn-danger"
          :disabled="busy"
          @click="emit('confirm')"
        >
          {{ busy ? 'Working…' : request.flavour === 'vanquish' ? 'Force complete all' : 'Delete subgraph' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dlg-layer {
  position: fixed;
  inset: 0;
  z-index: 320;
}

.dlg-scrim {
  position: absolute;
  inset: 0;
  background: rgba(4, 7, 12, 0.46);
  backdrop-filter: blur(1px);
}

.dlg-box {
  position: relative;
  width: min(460px, 90vw);
  margin: 16vh auto 0;
  background: var(--bg-1);
  border: 1px solid var(--line-2);
  border-radius: var(--r-l);
  box-shadow: 0 14px 44px rgba(0, 0, 0, 0.4);
  padding: 18px 20px;
}

.dlg-box.delete {
  border-color: var(--err-line);
}

.dlg-glyph {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: var(--r-m);
  margin-bottom: 8px;
}

.dlg-box.vanquish .dlg-glyph {
  color: var(--gold);
  background: var(--inset);
  border: 1px solid var(--line-2);
}

.dlg-box.delete .dlg-glyph {
  color: var(--err-fg);
  background: var(--err-bg);
  border: 1px solid var(--err-line);
}

.dlg-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.dlg-target {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 8px 0 0;
  min-width: 0;
}

.dlg-target .id {
  flex: none;
  font-size: 11px;
  color: var(--gold);
  background: var(--inset);
  border: 1px solid var(--line-1);
  border-radius: var(--r-s);
  padding: 1px 7px;
}

.dlg-target .t {
  font-size: 12.5px;
  color: var(--text-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dlg-lede {
  margin: 12px 0 0;
  font-size: 13px;
  line-height: 1.55;
  color: var(--text-1);
}

.dlg-lede strong {
  color: var(--gold);
  font-variant-numeric: tabular-nums;
}

.dlg-box.delete .dlg-lede strong {
  color: var(--err-fg);
}

.dlg-notes {
  margin: 10px 0 0;
  padding: 0 0 0 16px;
  font-size: 11.5px;
  color: var(--text-3);
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.dlg-warn {
  display: flex;
  align-items: center;
  gap: 7px;
  margin: 12px 0 0;
  font-size: 11.5px;
  color: var(--text-3);
}

.dlg-warn :deep(svg) {
  flex: none;
  color: var(--text-3);
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}

.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 18px;
}

.btn-ghost {
  padding: 5px 14px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-2);
  color: var(--text-2);
  font-size: 12px;
  background: transparent;
}

.btn-ghost:hover {
  border-color: var(--text-2);
  color: var(--text-1);
}

.btn-danger {
  padding: 5px 14px;
  border-radius: var(--r-m);
  border: 1px solid var(--err-line);
  background: var(--err-bg);
  color: var(--err-fg);
  font-size: 12px;
  font-weight: 600;
}

.btn-danger:hover {
  filter: brightness(1.08);
}

.btn-danger:disabled {
  opacity: 0.6;
  cursor: default;
}
</style>
