<script setup lang="ts">
// Node actions menu (test-feedback Item A): one floating menu with two
// triggers — the graph toolbar's + button (acts on the selection) and
// right-clicking any node (acts on the clicked node). Offers child/sibling
// creation with a kind dropdown (opening the shared New Node dialog) plus
// double-click navigation. The parent owns the actee and the menu anchor;
// this component owns the submenu state, clamping and outside-click close.

import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import type { NodeView } from '../../../types'
import { KIND_LABEL, KINDS } from '../../../lib/kind'

const props = defineProps<{
  /** The node the menu acts on; null hides the menu. */
  node: NodeView | null
  /** Anchor position in viewport coordinates (cursor or button corner). */
  x: number
  y: number
}>()

const emit = defineEmits<{
  'create-child': [kind: string]
  'create-sibling-kind': [kind: string]
  'open-local': []
  close: []
}>()

const menuEl = ref<HTMLElement | null>(null)
/** Open submenu: 'child' | 'sibling', or null when closed. */
const menuSub = ref<string | null>(null)

/** Keep the menu on screen (clamped after first paint). */
const style = computed(() => {
  const w = menuEl.value?.offsetWidth ?? 220
  const h = menuEl.value?.offsetHeight ?? 170
  return {
    left: `${Math.min(props.x, window.innerWidth - w - 8)}px`,
    top: `${Math.min(props.y, window.innerHeight - h - 8)}px`,
  }
})

function openSub(kind: string | null) {
  menuSub.value = kind
}

function done() {
  menuSub.value = null
  emit('close')
}

function onGlobalPointerDown(e: PointerEvent) {
  if (menuEl.value && !menuEl.value.contains(e.target as Node)) done()
}

onMounted(() => document.addEventListener('pointerdown', onGlobalPointerDown))
onBeforeUnmount(() => document.removeEventListener('pointerdown', onGlobalPointerDown))
</script>

<template>
  <!-- Teleported to <body> so it floats above the SVG at any zoom/pan. -->
  <teleport to="body">
    <div v-if="node" ref="menuEl" class="node-actions-menu" role="menu" :style="style">
      <div class="nam-head">
        <span class="nam-tile">{{ KIND_LABEL[node.kind] }}</span>
        <span class="nam-title" :title="node.title">{{ node.title }}</span>
      </div>
      <div class="nam-sep" aria-hidden="true"></div>
      <div class="nam-sub-wrap">
        <div
          class="nam-row"
          :class="{ open: menuSub === 'child' }"
          role="menuitem"
          aria-haspopup="menu"
          @mouseenter="openSub('child')"
        >
          <span>New child node…</span>
          <span class="nam-chev">›</span>
        </div>
        <div v-if="menuSub === 'child'" class="nam-sub" role="menu">
          <div
            v-for="k in KINDS"
            :key="k"
            class="nam-row nam-sub-row"
            role="menuitem"
            @click="emit('create-child', k); done()"
          >
            {{ KIND_LABEL[k] }}
          </div>
        </div>
      </div>
      <div class="nam-sub-wrap">
        <div
          class="nam-row"
          :class="{ open: menuSub === 'sibling' }"
          role="menuitem"
          aria-haspopup="menu"
          @mouseenter="openSub('sibling')"
        >
          <span>New sibling node…</span>
          <span class="nam-chev">›</span>
        </div>
        <div v-if="menuSub === 'sibling'" class="nam-sub" role="menu">
          <div
            v-for="k in KINDS"
            :key="k"
            class="nam-row nam-sub-row"
            role="menuitem"
            @click="emit('create-sibling-kind', k); done()"
          >
            {{ KIND_LABEL[k] }}
            <span v-if="k === node.kind" class="nam-hint">same kind</span>
          </div>
        </div>
      </div>
      <div class="nam-sep" aria-hidden="true"></div>
      <div class="nam-row" role="menuitem" @click="emit('open-local'); done()">
        <span>Open local graph</span>
      </div>
    </div>
  </teleport>
</template>

<style scoped>
.node-actions-menu {
  position: fixed;
  z-index: 1000;
  min-width: 216px;
  padding: 4px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  box-shadow: var(--shadow-1);
  font-size: 12.5px;
}

.nam-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px 6px;
}

.nam-tile {
  flex: none;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: var(--r-s);
  border: 1px solid var(--line-2);
  color: var(--text-2);
}

.nam-title {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-2);
  font-weight: 600;
}

.nam-sep {
  height: 1px;
  margin: 3px 6px;
  background: var(--line-1);
}

.nam-sub-wrap {
  position: relative;
}

.nam-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 4px 8px;
  border-radius: var(--r-s);
  color: var(--text-1);
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
}

.nam-row:hover,
.nam-row.open {
  background: var(--bg-2);
}

.nam-chev {
  color: var(--text-3);
}

.nam-hint {
  font-size: 9.5px;
  color: var(--faint);
}

.nam-sub {
  position: absolute;
  top: -5px;
  left: calc(100% + 6px);
  min-width: 150px;
  padding: 4px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  box-shadow: var(--shadow-1);
}

.nam-sub-row {
  justify-content: flex-start;
}
</style>
