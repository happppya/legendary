<script setup lang="ts">
// Global command palette (doc 05 §5.7): fuzzy-matches commands and realm
// nodes by title/id. Arrow keys navigate, Enter runs, Esc closes.

import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { NodeView } from '../types'
import { KIND_ICON } from '../lib/kind'

export interface PaletteCommand {
  id: string
  title: string
  hint?: string
}

interface Result {
  kind: 'header' | 'command' | 'node'
  id: string
  title: string
  hint?: string
  node?: NodeView
}

const props = defineProps<{
  nodes: NodeView[]
  commands: PaletteCommand[]
}>()

const emit = defineEmits<{
  close: []
  command: [id: string]
  selectNode: [id: string]
}>()

const query = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLElement | null>(null)
const activeIdx = ref(0)

const results = computed<Result[]>(() => {
  const q = query.value.trim().toLowerCase()
  const out: Result[] = []
  const cmds = props.commands.filter(
    (c) => !q || c.title.toLowerCase().includes(q) || (c.hint ?? '').toLowerCase().includes(q),
  )
  if (cmds.length) {
    out.push({ kind: 'header', id: 'h-commands', title: 'Commands' })
    for (const c of cmds) out.push({ kind: 'command', id: c.id, title: c.title, hint: c.hint })
  }
  const nodes = props.nodes.filter(
    (n) => !q || n.title.toLowerCase().includes(q) || n.id.toLowerCase().includes(q),
  )
  if (nodes.length) {
    out.push({ kind: 'header', id: 'h-nodes', title: nodes.length === props.nodes.length ? 'Nodes' : `Nodes · ${nodes.length}` })
    for (const n of nodes) out.push({ kind: 'node', id: n.id, title: n.title, hint: n.id, node: n })
  }
  return out
})

const actionable = computed(() => results.value.filter((r) => r.kind !== 'header'))

function clampActive() {
  const count = actionable.value.length
  if (activeIdx.value >= count) activeIdx.value = count - 1
  if (activeIdx.value < 0) activeIdx.value = 0
}

watch(query, () => {
  activeIdx.value = 0
  clampActive()
})

function scrollActive() {
  nextTick(() => {
    const el = listRef.value?.querySelector('[data-active="true"]')
    el?.scrollIntoView({ block: 'nearest' })
  })
}

function activate(idx: number) {
  const row = actionable.value[idx]
  if (!row) return
  if (row.kind === 'node') emit('selectNode', row.id)
  else emit('command', row.id)
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
    return
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (actionable.value.length) {
      activeIdx.value = (activeIdx.value + 1) % actionable.value.length
      scrollActive()
    }
    return
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (actionable.value.length) {
      activeIdx.value = (activeIdx.value - 1 + actionable.value.length) % actionable.value.length
      scrollActive()
    }
    return
  }
  if (e.key === 'Enter') {
    e.preventDefault()
    if (actionable.value.length) activate(activeIdx.value)
    return
  }
  if (e.metaKey || e.ctrlKey) {
    // Let global shortcuts (e.g. ⌘K re-open after close) do their thing.
    if (e.key.toLowerCase() === 'k') {
      emit('close')
      return
    }
  }
}

onMounted(() => {
  nextTick(() => inputRef.value?.focus())
  window.addEventListener('keydown', onKey)
})

onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="palette-layer" role="dialog" aria-modal="true" aria-label="Command palette">
    <div class="palette-scrim" @mousedown="emit('close')"></div>
    <div class="palette-box" @mousedown.stop>
      <div class="palette-input-row">
        <span class="prompt" aria-hidden="true">⌘</span>
        <input
          ref="inputRef"
          v-model="query"
          class="palette-input"
          type="text"
          spellcheck="false"
          placeholder="Type a command or search nodes&hellip;"
          aria-label="Command palette"
        />
        <button type="button" class="esc-hint" aria-label="Close palette" @click="emit('close')">
          esc
        </button>
      </div>
      <div ref="listRef" class="palette-results" role="listbox">
        <template v-for="(r, i) in results" :key="r.kind + r.id">
          <div v-if="r.kind === 'header'" class="res-header">{{ r.title }}</div>
          <div
            v-else
            class="res-row"
            :class="{ active: i === activeIdx }"
            :data-active="i === activeIdx || undefined"
            role="option"
            :aria-selected="i === activeIdx"
            @mousemove="activeIdx = i"
            @click="activate(actionable.findIndex((x) => x.id === r.id && x.kind === r.kind))"
          >
            <span v-if="r.kind === 'node' && r.node" class="kind-tile" :class="r.node.kind">
              <component :is="KIND_ICON[r.node.kind]" :size="11" aria-hidden="true" />
            </span>
            <span v-else class="cmd-glyph" aria-hidden="true"></span>
            <span class="res-title">{{ r.title }}</span>
            <span v-if="r.hint" class="res-hint mono">{{ r.hint }}</span>
          </div>
        </template>
        <div v-if="!actionable.length" class="res-empty">
          No commands or nodes match &ldquo;{{ query }}&rdquo;.
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.palette-layer {
  position: fixed;
  inset: 0;
  z-index: 200;
}

.palette-scrim {
  position: absolute;
  inset: 0;
  background: rgba(4, 7, 12, 0.44);
  backdrop-filter: blur(1px);
}

.palette-box {
  position: relative;
  width: min(620px, 88vw);
  margin: 12vh auto 0;
  background: var(--bg-1);
  border: 1px solid var(--line-2);
  border-radius: var(--r-l);
  box-shadow: 0 12px 42px rgba(0, 0, 0, 0.45);
  overflow: hidden;
}

.palette-input-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border-bottom: 1px solid var(--line-1);
}

.prompt {
  font-size: 16px;
  color: var(--gold);
  line-height: 1;
}

.palette-input {
  flex: 1;
  min-width: 0;
  border: 0;
  background: transparent;
  outline: none;
  color: var(--text-1);
  font-size: 14px;
}

.palette-input::placeholder {
  color: var(--faint);
}

.esc-hint {
  flex: none;
  border-radius: var(--r-s);
  border: 1px solid var(--line-1);
  background: var(--bg-2);
  color: var(--faint);
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  padding: 1px 7px;
}

.esc-hint:hover {
  color: var(--text-1);
}

.palette-results {
  max-height: 46vh;
  overflow-y: auto;
  padding: 6px;
}

.res-header {
  padding: 8px 10px 4px;
  font-size: 9.5px;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--faint);
}

.res-row {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 5px 10px;
  border-radius: var(--r-s);
  cursor: pointer;
  color: var(--text-2);
  font-size: 13px;
}

.res-row.active {
  background: var(--sel-bg);
  color: var(--text-1);
}

.res-row.active .res-title {
  color: var(--text-1);
}

.kind-tile {
  flex: none;
  width: 18px;
  height: 18px;
  border-radius: var(--r-s);
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

.cmd-glyph {
  flex: none;
  width: 12px;
  height: 12px;
  border-radius: 3px;
  border: 1px solid var(--line-2);
  opacity: 0.6;
}

.res-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.res-hint {
  flex: none;
  font-size: 10.5px;
  color: var(--faint);
}

.res-empty {
  padding: 18px 10px;
  text-align: center;
  color: var(--faint);
  font-size: 12.5px;
}
</style>
