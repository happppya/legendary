<script setup lang="ts">
// The central split workspace: Graph View over Table View, each with its own
// closable tab and a draggable divider between them.

import { onBeforeUnmount, ref } from 'vue'
import type { NodeView } from '../../types'
import type { CheckRow, FilterModel } from '../../lib/filters'
import GraphPane from './graph/GraphPane.vue'
import TablePane from './TablePane.vue'

defineProps<{
  nodes: NodeView[]
  nodesById: Map<string, NodeView>
  filters: FilterModel
  selectedId: string | null
  isMockScene: boolean
  realmName: string
  statusCounts: CheckRow[]
  showGraph: boolean
  showTable: boolean
}>()

const emit = defineEmits<{
  select: [id: string]
  'update:showGraph': [v: boolean]
  'update:showTable': [v: boolean]
  toggleStatus: [key: string]
  openFilters: []
  clearFilters: []
  focusSearch: []
}>()

const ratio = ref(0.56)

/* ---- divider drag ------------------------------------------------------- */

const dragging = ref(false)
const dragStart = ref({ y: 0, ratio: 0.56 })
const splitRef = ref<HTMLElement | null>(null)

function onDividerDown(e: PointerEvent) {
  dragging.value = true
  dragStart.value = { y: e.clientY, ratio: ratio.value }
  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
}

function onMove(e: PointerEvent) {
  if (!dragging.value) return
  const box = splitRef.value?.getBoundingClientRect()
  if (!box || box.height < 40) return
  const dy = e.clientY - dragStart.value.y
  const next = dragStart.value.ratio + dy / box.height
  ratio.value = Math.min(0.85, Math.max(0.18, next))
}

function onUp() {
  dragging.value = false
  window.removeEventListener('pointermove', onMove)
  window.removeEventListener('pointerup', onUp)
}

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', onMove)
  window.removeEventListener('pointerup', onUp)
})
</script>

<template>
  <section ref="splitRef" class="workspace-center" :class="{ dragging }" aria-label="Workspace">
    <template v-if="showGraph">
      <GraphPane
        class="pane graph-pane-region"
        :class="{ grow: !showTable }"
        :style="showTable ? { flex: `${ratio} 1 0` } : undefined"
        :nodes-by-id="nodesById"
        :selected-id="selectedId"
        :is-mock-scene="isMockScene"
        :realm-name="realmName"
        @select="emit('select', $event)"
      />
      <button
        v-if="showTable"
        type="button"
        class="divider"
        :aria-label="'Resize panes'"
        @pointerdown="onDividerDown"
      >
        <span class="grip" aria-hidden="true"></span>
      </button>
    </template>

    <template v-if="showTable">
      <TablePane
        class="pane"
        :class="{ grow: !showGraph }"
        :style="showGraph ? { flex: `${1 - ratio} 1 0` } : undefined"
        :nodes="nodes"
        :filters="filters"
        :selected-id="selectedId"
        :status-counts="statusCounts"
        @select="emit('select', $event)"
        @close="emit('update:showTable', false)"
        @toggle-status="emit('toggleStatus', $event)"
        @open-filters="emit('openFilters')"
        @clear-filters="emit('clearFilters')"
        @focus-search="emit('focusSearch')"
      />
    </template>

    <div v-if="!showGraph && !showTable" class="center-empty">
      <p class="empty-title">Nothing open in this pane</p>
      <p class="empty-body">Reopen a view to keep working with the realm.</p>
      <div class="empty-actions">
        <button type="button" class="open-view" @click="emit('update:showGraph', true)">
          Graph View
        </button>
        <button type="button" class="open-view" @click="emit('update:showTable', true)">
          Table View
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.workspace-center {
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  flex: 1;
  background: var(--bg-0);
}

.pane {
  min-height: 0;
  min-width: 0;
}

.pane.grow {
  flex: 1 1 0;
}

.divider {
  flex: none;
  height: 5px;
  margin: -2px 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: row-resize;
  background: transparent;
  border: 0;
  z-index: 5;
}

.divider:hover .grip,
.workspace-center.dragging .grip {
  opacity: 1;
  width: 56px;
}

.grip {
  width: 40px;
  height: 3px;
  border-radius: 2px;
  background: var(--line-2);
  opacity: 0.6;
  transition: opacity 0.12s ease, width 0.12s ease;
}

.workspace-center.dragging {
  cursor: row-resize;
  user-select: none;
}

.workspace-center.dragging .divider .grip {
  background: var(--gold);
}

.center-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 24px;
  text-align: center;
}

.empty-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-2);
}

.empty-body {
  margin: 0 0 10px;
  font-size: 12px;
  color: var(--faint);
}

.empty-actions {
  display: flex;
  gap: 8px;
}

.open-view {
  padding: 5px 14px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-2);
  color: var(--text-2);
  font-size: 12px;
  background: var(--bg-2);
}

.open-view:hover {
  border-color: var(--gold);
  color: var(--gold);
}
</style>
