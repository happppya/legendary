<script setup lang="ts">
// "New Node" overlay (test-feedback A-2): create any node kind from any
// view. The write goes through the shared ops layer (`legend create`
// semantics): the engine assigns a fresh Base36 id, stamps `created_at` and
// seeds the per-kind body template (doc 04 §4.1).
//
// Callers preset the kind and/or parent (board column, entities card,
// graph toolbar with the selection as parent); everything stays editable.
// Engine rejections surface verbatim in the inline error line.

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { PhPlus, PhX } from '@phosphor-icons/vue'
import type { NodeView } from '../../types'
import { KIND_ICON, KIND_LABEL, KINDS } from '../../lib/kind'
import { PRIORITY_KEYS, PRIORITY_LABEL } from '../../lib/priority'

const props = defineProps<{
  /** Full realm snapshot (parent candidates). */
  nodes: NodeView[]
  /** Initial kind (preset callers: board column, entities card, …). */
  initialKind?: string | null
  /** Initial parent id (preset callers: graph toolbar, board header). */
  initialParent?: string | null
  busy: boolean
  error?: string
}>()

const emit = defineEmits<{
  close: []
  create: [args: { kind: string; title: string; parent: string | null; priority: string; questPoints: number | null }]
}>()

const kind = ref(props.initialKind && KINDS.includes(props.initialKind as never) ? props.initialKind : 'action')
const title = ref('')
const parent = ref<string | ''>(props.initialParent ?? '')
const priority = ref('medium')
const qpRaw = ref('')

/** Parents: every container kind (Card / Genre) in the realm. */
const parents = computed(() =>
  props.nodes
    .filter((n) => n.kind === 'card' || n.kind === 'genre')
    .sort((a, b) => a.title.localeCompare(b.title)),
)

/** A leaf kind is required under the doc-02 vocabulary; keep the default
 * honest when the caller presets a container kind without a parent. */
const qpRequired = computed(() => !isContainer(kind.value))
const qpInvalid = computed(() => {
  if (!qpRequired.value || qpRaw.value.trim() === '') return false
  return ![1, 2, 3, 5, 8, 13, 21].includes(Number(qpRaw.value))
})

function isContainer(k: string): boolean {
  return k === 'card' || k === 'genre'
}

const titleInvalid = computed(() => title.value.trim() === '')

const canSubmit = computed(() => !titleInvalid.value && !qpInvalid.value)

function submit() {
  if (!canSubmit.value || props.busy) return
  const raw = qpRaw.value.trim()
  emit('create', {
    kind: kind.value,
    title: title.value.trim(),
    parent: parent.value === '' ? null : parent.value,
    priority: priority.value,
    questPoints: raw === '' ? null : Number(raw),
  })
}

const titleInput = ref<HTMLInputElement | null>(null)

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
    return
  }
  if (e.key === 'Enter' && (e.target as HTMLElement)?.tagName !== 'BUTTON') {
    e.preventDefault()
    submit()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKey)
  titleInput.value?.focus()
})
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

watch(kind, (k) => {
  if (!isContainer(k) && qpRaw.value.trim() === '') qpRaw.value = '3'
})
</script>

<template>
  <div class="dlg-layer" role="dialog" aria-modal="true" aria-label="Create node">
    <div class="dlg-scrim" @click="emit('close')"></div>
    <div class="dlg-box">
      <header class="dlg-head">
        <span class="dlg-glyph" aria-hidden="true"><PhPlus :size="14" /></span>
        <h2 class="dlg-title">New Node</h2>
        <button type="button" class="dlg-x" aria-label="Close" @click="emit('close')">
          <PhX :size="13" aria-hidden="true" />
        </button>
      </header>

      <label class="fld">
        <span class="fld-label">Kind</span>
        <span class="kind-row" role="radiogroup" aria-label="Node kind">
          <button
            v-for="k in KINDS"
            :key="k"
            type="button"
            class="kind-choice"
            :class="{ on: kind === k }"
            role="radio"
            :aria-checked="kind === k"
            @click="kind = k"
          >
            <component :is="KIND_ICON[k]" :size="12" aria-hidden="true" />
            {{ KIND_LABEL[k] }}
          </button>
        </span>
      </label>

      <label class="fld">
        <span class="fld-label">Title</span>
        <input
          ref="titleInput"
          v-model="title"
          class="fld-input"
          type="text"
          placeholder="Short, clearly defined title…"
          spellcheck="false"
          aria-label="Node title"
        />
      </label>

      <div class="fld-row">
        <label class="fld">
          <span class="fld-label">Parent</span>
          <select v-model="parent" class="fld-input" aria-label="Parent node">
            <option value="">— root node (no parent) —</option>
            <option v-for="p in parents" :key="p.id" :value="p.id">{{ p.title }} ({{ p.id }})</option>
          </select>
        </label>
        <label class="fld">
          <span class="fld-label">Priority</span>
          <select v-model="priority" class="fld-input" aria-label="Priority">
            <option v-for="p in PRIORITY_KEYS" :key="p" :value="p">{{ PRIORITY_LABEL[p] }}</option>
          </select>
        </label>
      </div>

      <label class="fld">
        <span class="fld-label">
          Quest points
          <span v-if="isContainer(kind)" class="fld-hint">(aggregate over children — leave empty)</span>
        </span>
        <input
          v-model="qpRaw"
          class="fld-input mono"
          :class="{ invalid: qpInvalid }"
          type="text"
          inputmode="numeric"
          placeholder="1 · 2 · 3 · 5 · 8 · 13 · 21"
          aria-label="Quest points"
        />
        <span v-if="qpInvalid" class="fld-err">Quest points must be Fibonacci: 1, 2, 3, 5, 8, 13, 21.</span>
      </label>

      <p v-if="error" class="dlg-error">{{ error }}</p>

      <div class="dlg-actions">
        <span class="action-hint">a fresh node ID is assigned on create</span>
        <button type="button" class="btn-ghost" :disabled="busy" @click="emit('close')">Cancel</button>
        <button type="button" class="btn-primary" :disabled="!canSubmit || busy" @click="submit">
          {{ busy ? 'Working…' : 'Create node' }}
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
  width: min(480px, 90vw);
  margin: 12vh auto 0;
  background: var(--bg-1);
  border: 1px solid var(--line-2);
  border-radius: var(--r-l);
  box-shadow: 0 14px 44px rgba(0, 0, 0, 0.4);
  padding: 16px 18px;
  max-height: 78vh;
  overflow-y: auto;
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

/* ---- fields ----------------------------------------------------------- */

.fld {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 12px;
  min-width: 0;
}

.fld-label {
  display: inline-flex;
  align-items: baseline;
  gap: 7px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--faint);
}

.fld-hint {
  text-transform: none;
  letter-spacing: 0;
  font-weight: 400;
}

.fld-row {
  display: flex;
  gap: 12px;
}

.fld-row .fld {
  flex: 1;
}

.fld-input {
  width: 100%;
  height: 28px;
  padding: 0 10px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--inset);
  color: var(--text-1);
  font-size: 12.5px;
  outline: none;
}

.fld-input:focus {
  border-color: var(--gold);
}

.fld-input.invalid {
  border-color: var(--err-line);
}

.fld-input.mono {
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}

.fld-err {
  font-size: 10.5px;
  color: var(--err-fg);
}

.kind-row {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.kind-choice {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  background: transparent;
  color: var(--text-2);
  font-size: 11.5px;
}

.kind-choice:hover {
  border-color: var(--line-2);
  color: var(--text-1);
}

.kind-choice.on {
  background: var(--inset);
  border-color: var(--gold);
  color: var(--gold);
}

.dlg-error {
  margin: 12px 0 0;
  font-size: 11.5px;
  color: var(--err-fg);
}

/* ---- actions ------------------------------------------------------------ */

.dlg-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 18px;
  border-top: 1px solid var(--line-1);
  padding-top: 12px;
}

.action-hint {
  flex: 1;
  font-size: 10.5px;
  color: var(--faint);
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

.btn-primary {
  padding: 5px 14px;
  border-radius: var(--r-m);
  border: 1px solid var(--gold);
  background: var(--inset);
  color: var(--gold);
  font-size: 12px;
  font-weight: 600;
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.12);
}

.btn-primary:disabled {
  opacity: 0.55;
  cursor: default;
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}
</style>
