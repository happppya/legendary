<script setup lang="ts">
// Detail sheet, composed from focused section components (see ./):
//   NodeBreadcrumbs · NodeHeader · NodeStatusChips · NodeMetaSheet ·
//   NodeNotes.
// This file keeps only the selection/filter wiring, the ancestor + blocked
// resolution shared by those sections, and the "nothing selected" empty
// state.

import { computed } from 'vue'
import type { NodeView } from '../../../types'
import { ancestorChain } from '../../../lib/node'
import NodeBreadcrumbs from './NodeBreadcrumbs.vue'
import NodeHeader from './NodeHeader.vue'
import NodeStatusChips from './NodeStatusChips.vue'
import NodeMetaSheet from './NodeMetaSheet.vue'
import NodeNotes from './NodeNotes.vue'

const props = defineProps<{
  node: NodeView | null
  nodesById: Map<string, NodeView>
  realmName: string
}>()

const emit = defineEmits<{
  select: [id: string]
  filter: [{ field: 'discipline' | 'epic' | 'tag' | 'landmark'; value: string }]
}>()

/* ---- position in the realm ------------------------------------------- */

/** Root → parent chain of the shown node (the node itself is excluded). */
const ancestors = computed<NodeView[]>(() =>
  props.node ? ancestorChain(props.nodesById, props.node.parent) : [],
)

/** blocked_by ids with titles resolved, for the blocked callout links. */
const blockedRefs = computed<{ id: string; label: string }[]>(() => {
  const n = props.node
  if (!n) return []
  return n.blockedBy.map((bid) => {
    const other = props.nodesById.get(bid)
    return { id: bid, label: other ? other.title : bid }
  })
})
</script>

<template>
  <article v-if="node" class="detail" :class="node.kind">
    <NodeBreadcrumbs :realm-name="realmName" :ancestors="ancestors" @select="emit('select', $event)" />
    <NodeHeader :node="node" />
    <NodeStatusChips :node="node" :blocked-refs="blockedRefs" @select="emit('select', $event)" />
    <NodeMetaSheet :node="node" @filter="emit('filter', $event)" />
    <NodeNotes :node="node" />
  </article>

  <article v-else class="detail empty-detail">
    <div class="empty-glyph" aria-hidden="true"></div>
    <p class="empty-title">Select a node to read it.</p>
    <p class="empty-hint">
      <span><kbd>/</kbd> filter</span>
      <span><kbd>↑</kbd><kbd>↓</kbd> move</span>
      <span><kbd>←</kbd><kbd>→</kbd> collapse</span>
      <span><kbd>esc</kbd> clear</span>
    </p>
  </article>
</template>

<style scoped>
.detail {
  min-width: 0;
  overflow-y: auto;
  padding: 18px 26px 60px;
  max-width: 1080px;
}

/* ---- empty state ------------------------------------------------------ */

.empty-detail {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  padding: 0 48px;
}

.empty-glyph {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: var(--bg-2);
  position: relative;
  margin-bottom: 18px;
}

.empty-glyph::before,
.empty-glyph::after {
  content: '';
  position: absolute;
  background: var(--gold);
  border-radius: 50%;
}

.empty-glyph::before {
  width: 9px;
  height: 9px;
  left: 10px;
  top: 10px;
}

.empty-glyph::after {
  width: 6px;
  height: 6px;
  right: 12px;
  bottom: 12px;
  background: var(--line-2);
}

.empty-title {
  margin: 0 0 10px;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.empty-hint {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 16px;
  margin: 0;
  color: var(--faint);
  font-size: 12px;
}

.empty-hint kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 17px;
  height: 18px;
  margin-right: 4px;
  padding: 0 4px;
  border-radius: var(--r-s);
  border: 1px solid var(--line-2);
  background: var(--bg-1);
  color: var(--text-2);
  font-size: 10px;
}
</style>
