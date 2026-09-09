<script setup lang="ts">
// The central split workspace: Graph View over Table View as closable,
// reopenable tabs (test-feedback A-4) with a draggable divider between
// them. The ⋯ tab-strip menu lists every closed tab so a pane can always
// be brought back after its × is clicked.

import { computed, onBeforeUnmount, ref } from 'vue'
import { PhDotsThree } from '@phosphor-icons/vue'
import type { NodeView } from '../../types'
import type { CheckRow, FilterModel } from '../../lib/filters'
import type { OverlayMode } from '../../lib/overlay'
import type { CompletionLevel } from '../../lib/node'
import type { GraphScope } from './graph/scene'
import GraphPane from './graph/GraphPane.vue'
import TablePane from './TablePane.vue'

const props = defineProps<{
  nodes: NodeView[]
  nodesById: Map<string, NodeView>
  filters: FilterModel
  selectedId: string | null
  isMockScene: boolean
  realmName: string
  statusCounts: CheckRow[]
  showGraph: boolean
  showTable: boolean
  overlayMode: OverlayMode
  scope: GraphScope
  completionLevel: CompletionLevel
  /** Mutations are desktop-only; toggles create/delete affordances. */
  canMutate: boolean
}>()

const emit = defineEmits<{
  select: [id: string]
  'update:showGraph': [v: boolean]
  'update:showTable': [v: boolean]
  toggleStatus: [key: string]
  openFilters: []
  clearFilters: []
  focusSearch: []
  reparent: [childId: string, parentId: string]
  'update:overlayMode': [mode: OverlayMode]
  'update:scope': [scope: GraphScope]
  'update:completionLevel': [level: CompletionLevel]
  /** Double-click a node in global/overview: open its local graph. */
  openLocal: [id: string]
  /** Create a node (graph toolbar preset: parent id from the selection). */
  create: [parentId: string | null]
  /** Create a node from the table's + (no preset). */
  'create-any': []
  /** Delete a node from the table's more-menu. */
  'delete-node': [id: string]
}>()

/* ---- reopenable tab strip (test-feedback A-4) --------------------------- */

/** Closed panes, in closure order (most recent first) — the ⋯ menu lists
 * them so any closed tab can be reopened with one click. */
const closedTabs = computed(() => {
  const out: { id: 'graph' | 'table'; label: string }[] = []
  if (!props.showGraph) out.push({ id: 'graph', label: 'Graph View' })
  if (!props.showTable) out.push({ id: 'table', label: 'Table View' })
  return out
})

const menuOpen = ref(false)

function reopen(id: 'graph' | 'table') {
  menuOpen.value = false
  if (id === 'graph') emit('update:showGraph', true)
  else emit('update:showTable', true)
}

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
    <!-- reopenable tab strip (test-feedback A-4): shows only while something
         is closed, so open panes keep their own in-pane tabs -->
    <div v-if="closedTabs.length && (showGraph || showTable)" class="tab-strip">
      <span
        v-for="t in closedTabs"
        :key="t.id"
        class="tab-closed"
        role="button"
        tabindex="0"
        :title="`Reopen ${t.label}`"
        @click="reopen(t.id)"
        @keydown.enter.prevent="reopen(t.id)"
      >
        {{ t.label }}
        <span class="tab-plus" aria-hidden="true">+</span>
      </span>
      <span class="rel">
        <button
          type="button"
          class="strip-more"
          :class="{ on: menuOpen }"
          title="Reopen closed tab"
          aria-label="Reopen closed tab"
          @click="menuOpen = !menuOpen"
        >
          <PhDotsThree :size="14" aria-hidden="true" />
        </button>
        <div v-if="menuOpen" class="strip-menu" role="menu">
          <button
            v-for="t in closedTabs"
            :key="t.id"
            type="button"
            class="strip-item"
            role="menuitem"
            @click="reopen(t.id)"
          >
            {{ t.label }}
          </button>
        </div>
      </span>
    </div>

    <template v-if="showGraph">
      <GraphPane
        class="pane graph-pane-region"
        :class="{ grow: !showTable || closedTabs.length === 0 }"
        :style="showTable ? { flex: `${ratio} 1 0` } : undefined"
        :nodes-by-id="nodesById"
        :selected-id="selectedId"
        :is-mock-scene="isMockScene"
        :realm-name="realmName"
        :overlay-mode="overlayMode"
        :scope="scope"
        :completion-level="completionLevel"
        @select="emit('select', $event)"
        @reparent="(childId, parentId) => emit('reparent', childId, parentId)"
        @open-local="(id) => emit('openLocal', id)"
        @create="(parentId) => emit('create', parentId)"
        @close="emit('update:showGraph', false)"
        @update:overlay-mode="emit('update:overlayMode', $event)"
        @update:scope="emit('update:scope', $event)"
        @update:completion-level="emit('update:completionLevel', $event)"
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
        :can-mutate="canMutate"
        @select="emit('select', $event)"
        @close="emit('update:showTable', false)"
        @toggle-status="emit('toggleStatus', $event)"
        @open-filters="emit('openFilters')"
        @clear-filters="emit('clearFilters')"
        @focus-search="emit('focusSearch')"
        @create="emit('create-any')"
        @delete-node="(id) => emit('delete-node', id)"
      />
    </template>

    <div v-if="!showGraph && !showTable" class="center-empty">
      <p class="empty-title">Nothing open in this pane</p>
      <p class="empty-body">Reopen a view to keep working with the realm.</p>
      <div class="empty-actions">
        <button type="button" class="open-view" @click="reopen('graph')">
          Graph View
        </button>
        <button type="button" class="open-view" @click="reopen('table')">
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

/* ---- reopenable tab strip (A-4) ----------------------------------------- */

.tab-strip {
  flex: none;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-bottom: 1px solid var(--line-1);
  background: var(--bg-1);
}

.tab-closed {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 9px;
  border-radius: 999px;
  border: 1px dashed var(--line-2);
  color: var(--text-3);
  font-size: 11px;
  cursor: pointer;
}

.tab-closed:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.tab-closed .tab-plus {
  font-weight: 600;
}

.rel {
  position: relative;
  margin-left: auto;
}

.strip-more {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 22px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.strip-more:hover,
.strip-more.on {
  background: var(--bg-2);
  color: var(--text-1);
}

.strip-menu {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 40;
  min-width: 150px;
  padding: 4px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  box-shadow: var(--shadow-1);
}

.strip-item {
  display: block;
  width: 100%;
  padding: 4px 9px;
  border-radius: var(--r-s);
  font-size: 12px;
  color: var(--text-2);
  text-align: left;
}

.strip-item:hover {
  background: var(--bg-2);
  color: var(--text-1);
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
