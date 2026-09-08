<script setup lang="ts">
// "Move Subgraph To…" dialog (doc 04 §4.5 Batch Re-parenting Action):
// pick a new parent Card for the selected node — or promote it to root —
// and the write goes through the same engine `reparent` op the CLI uses
// (cycle-checked; candidates that would close a cycle are disabled here,
// with the engine as the final authority).
//
// Candidates are every Card in the realm except the node's own subtree and
// its current parent. A search box keeps large realms navigable.

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { PhCornersOut, PhX } from '@phosphor-icons/vue'
import type { NodeView } from '../../types'
import { isContainerKind } from '../../lib/kind'

const props = defineProps<{
  /** Node being moved. */
  node: NodeView
  /** Full realm snapshot (candidates + subtree computation). */
  nodes: NodeView[]
  busy: boolean
}>()

const emit = defineEmits<{ close: []; move: [parent: string | null] }>()

/* ---- subtree (excluded from candidates) ------------------------------- */

function descendantIds(rootId: string): Set<string> {
  const out = new Set<string>([rootId])
  let grew = true
  while (grew) {
    grew = false
    for (const n of props.nodes) {
      if (n.parent && out.has(n.parent) && !out.has(n.id)) {
        out.add(n.id)
        grew = true
      }
    }
  }
  return out
}

const subtree = computed(() => descendantIds(props.node.id))

/* ---- candidates --------------------------------------------------------- */

const query = ref('')

interface Candidate {
  id: string
  title: string
  path: string
  current: boolean
  ancestor: boolean
}

const candidates = computed<Candidate[]>(() => {
  const byId = new Map(props.nodes.map((n) => [n.id, n]))
  const q = query.value.trim().toLowerCase()

  const chainOf = (id: string): NodeView[] => {
    const chain: NodeView[] = []
    let cur = byId.get(id) ?? null
    const seen = new Set<string>()
    while (cur && !seen.has(cur.id)) {
      seen.add(cur.id)
      chain.unshift(cur)
      cur = cur.parent ? (byId.get(cur.parent) ?? null) : null
    }
    return chain
  }

  const pathOf = (n: NodeView): string => {
    const chain = n.parent ? chainOf(n.parent) : []
    return chain.map((c) => c.title).join(' / ')
  }

  const out: Candidate[] = []
  for (const n of props.nodes) {
    if (!isContainerKind(n.kind)) continue
    if (subtree.value.has(n.id)) continue
    const current = n.id === props.node.parent
    if (current) continue
    out.push({
      id: n.id,
      title: n.title,
      path: pathOf(n),
      current,
      ancestor: false,
    })
  }
  const filtered = q
    ? out.filter((c) => c.title.toLowerCase().includes(q) || c.id.toLowerCase().includes(q))
    : out
  return filtered.sort((a, b) => a.path.localeCompare(b.path) || a.title.localeCompare(b.title))
})

const canPromoteRoot = computed(() => props.node.parent !== null)

/* ---- selection + keys ----------------------------------------------------- */

const activeIdx = ref(0)
const listRef = ref<HTMLElement | null>(null)

const activeCandidate = computed<Candidate | null>(
  () => candidates.value[activeIdx.value] ?? null,
)

watch(
  () => [candidates.value.length, query.value],
  () => {
    if (activeIdx.value >= candidates.value.length) activeIdx.value = 0
  },
)

function choose(c: Candidate) {
  if (props.busy) return
  emit('move', c.id)
}

function promoteRoot() {
  if (props.busy || !canPromoteRoot.value) return
  emit('move', null)
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
    return
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (candidates.value.length) {
      activeIdx.value = (activeIdx.value + 1) % candidates.value.length
      scrollActive()
    }
    return
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (candidates.value.length) {
      activeIdx.value =
        (activeIdx.value - 1 + candidates.value.length) % candidates.value.length
      scrollActive()
    }
    return
  }
  if (e.key === 'Enter') {
    e.preventDefault()
    if (activeCandidate.value) choose(activeCandidate.value)
  }
}

function scrollActive() {
  const el = listRef.value?.querySelector('[data-active="true"]')
  el?.scrollIntoView({ block: 'nearest' })
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <div class="dlg-layer" role="dialog" aria-modal="true" aria-label="Move subgraph to parent">
    <div class="dlg-scrim" @click="emit('close')"></div>
    <div class="dlg-box">
      <header class="dlg-head">
        <span class="dlg-glyph" aria-hidden="true"><PhCornersOut :size="14" /></span>
        <h2 class="dlg-title">Move Subgraph To…</h2>
        <button type="button" class="dlg-x" aria-label="Close" @click="emit('close')">
          <PhX :size="13" aria-hidden="true" />
        </button>
      </header>

      <p class="dlg-target">
        <span class="mono id">{{ node.id }}</span>
        <span class="t">{{ node.title }}</span>
      </p>
      <p class="dlg-note">
        Moving moves the node and its whole subtree ({{ subtree.size - 1 }}
        descendant{{ subtree.size - 1 === 1 ? '' : 's' }} stay beneath it).
      </p>

      <input
        v-model="query"
        class="dlg-search mono"
        type="text"
        placeholder="Filter cards…"
        spellcheck="false"
        aria-label="Filter candidate cards"
      />

      <div ref="listRef" class="cand-list" role="listbox" aria-label="Candidate parents">
        <button
          v-for="(c, i) in candidates"
          :key="c.id"
          type="button"
          class="cand-row"
          :class="{ active: i === activeIdx }"
          :data-active="i === activeIdx || undefined"
          role="option"
          :aria-selected="i === activeIdx"
          @mousemove="activeIdx = i"
          @click="choose(c)"
        >
          <span class="cand-main">
            <span class="cand-title">{{ c.title }}</span>
            <span v-if="c.path" class="cand-path">{{ c.path }}</span>
          </span>
          <span class="cand-id mono">{{ c.id }}</span>
        </button>
        <p v-if="!candidates.length" class="cand-empty">No other Cards in this realm.</p>
      </div>

      <div class="dlg-actions">
        <button
          type="button"
          class="btn-root"
          :disabled="!canPromoteRoot || busy"
          :title="canPromoteRoot ? 'Set parent: null' : 'Already a root node'"
          @click="promoteRoot"
        >
          Move to root
        </button>
        <span class="action-hint">engine cycle-check applies</span>
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
  width: min(500px, 90vw);
  margin: 13vh auto 0;
  background: var(--bg-1);
  border: 1px solid var(--line-2);
  border-radius: var(--r-l);
  box-shadow: 0 14px 44px rgba(0, 0, 0, 0.4);
  padding: 16px 18px;
}

.dlg-head {
  display: flex;
  align-items: center;
  gap: 9px;
}

.dlg-glyph {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: var(--r-m);
  color: var(--gold);
  background: var(--inset);
  border: 1px solid var(--line-2);
}

.dlg-title {
  flex: 1;
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-1);
}

.dlg-x {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.dlg-x:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.dlg-target {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 12px 0 0;
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

.dlg-note {
  margin: 6px 0 0;
  font-size: 11.5px;
  color: var(--faint);
}

.dlg-search {
  width: 100%;
  height: 28px;
  margin-top: 12px;
  padding: 0 10px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--inset);
  color: var(--text-1);
  font-size: 12px;
  outline: none;
}

.dlg-search:focus {
  border-color: var(--gold);
}

.cand-list {
  max-height: 250px;
  overflow-y: auto;
  margin-top: 8px;
  padding: 3px;
}

.cand-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 6px 9px;
  border-radius: var(--r-m);
  color: var(--text-2);
  text-align: left;
}

.cand-row:hover,
.cand-row.active {
  background: var(--sel-bg);
  color: var(--text-1);
}

.cand-row.active .cand-title {
  color: var(--text-1);
}

.cand-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.cand-title {
  font-size: 12.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cand-path {
  font-size: 10.5px;
  color: var(--faint);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cand-id {
  flex: none;
  font-size: 10px;
  color: var(--gold);
  opacity: 0.85;
}

.cand-empty {
  padding: 14px 8px;
  text-align: center;
  font-size: 12px;
  color: var(--faint);
}

.dlg-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
  border-top: 1px solid var(--line-1);
  padding-top: 12px;
}

.btn-root {
  padding: 4px 13px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-2);
  background: var(--bg-2);
  color: var(--text-2);
  font-size: 12px;
}

.btn-root:hover:not(:disabled) {
  border-color: var(--gold);
  color: var(--gold);
}

.btn-root:disabled {
  opacity: 0.45;
  cursor: default;
}

.action-hint {
  flex: 1;
  text-align: right;
  font-size: 10.5px;
  color: var(--faint);
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}
</style>
