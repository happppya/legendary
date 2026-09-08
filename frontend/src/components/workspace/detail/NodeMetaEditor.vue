<script setup lang="ts">
// Metadata component editor (doc 05 §5.1 side panel): dedicated controls for
// the editable ECS components — status, priority, quest points, disciplines,
// epics, landmark, dependencies — saving through the store → ops layer →
// synchronous index. Vocabulary options come from the realm rows passed in;
// validation (Fibonacci QP, known branches) is enforced by the engine.
//
// Status of a Card follows the vanquish rule (doc 04 §4.3): switching a Card
// to vanquished routes through the confirmation modal in App, not here.

import { computed, ref, watch } from 'vue'
import type { NodeView } from '../../../types'
import { PRIORITIES } from '../../../lib/priority'

const props = defineProps<{  node: NodeView
  canMutate: boolean
  busy: boolean
  /** Vocabulary rows from the realm (filters.ts builders). */
  disciplineOptions: { key: string; label: string }[]
  epicOptions: { key: string; label: string }[]
  landmarkOptions: { key: string; label: string }[]
}>()

const emit = defineEmits<{
  'set-status': [status: 'unstarted' | 'active' | 'vanquished']
  'save-meta': [edits: MetaEdits]
}>()

export interface MetaEdits {
  priority?: string
  questPoints?: number
  clearQuestPoints?: boolean
  disciplines?: string[]
  epics?: string[]
  landmark?: string
  clearLandmark?: boolean
  blockedBy?: string[]
}

/* ---- draft state, re-seeded on node swap ------------------------------- */

const priority = ref<string>(props.node.priority)
const qpText = ref(props.node.questPoints === null ? '' : String(props.node.questPoints))
const disciplines = ref<string[]>([...props.node.disciplines])
const epics = ref<string[]>([...props.node.epics])
const landmark = ref(props.node.explicitLandmark ?? '')
const blockedByText = ref(props.node.blockedBy.join(', '))

watch(
  () => props.node.id,
  () => {
    priority.value = props.node.priority
    qpText.value = props.node.questPoints === null ? '' : String(props.node.questPoints)
    disciplines.value = [...props.node.disciplines]
    epics.value = [...props.node.epics]
    landmark.value = props.node.explicitLandmark ?? ''
    blockedByText.value = props.node.blockedBy.join(', ')
  },
)

/* ---- option lists ------------------------------------------------------ */

const disciplineOptionsC = computed(() => {
  const have = new Set(disciplines.value)
  return props.disciplineOptions.map((o) => ({ ...o, on: have.has(o.key) }))
})
const epicOptionsC = computed(() => {
  const have = new Set(epics.value)
  return props.epicOptions.map((o) => ({ ...o, on: have.has(o.key) }))
})
const landmarkOptionsC = computed(() => props.landmarkOptions.map((o) => o.key))

function toggleDiscipline(key: string) {
  disciplines.value = disciplines.value.includes(key)
    ? disciplines.value.filter((d) => d !== key)
    : [...disciplines.value, key]
}
function toggleEpic(key: string) {
  epics.value = epics.value.includes(key)
    ? epics.value.filter((e) => e !== key)
    : [...epics.value, key]
}

/* ---- dirty tracking + save ---------------------------------------------- */

const qpDirty = computed(() => {
  const parsed = qpText.value.trim()
  if (parsed === '') return props.node.questPoints !== null
  const n = Number(parsed)
  return n !== props.node.questPoints
})

const landmarkDirty = computed(() => landmark.value !== (props.node.explicitLandmark ?? ''))

const depsDirty = computed(() => {
  const parsed = parseDeps(blockedByText.value)
  if (parsed.length !== props.node.blockedBy.length) return true
  const a = [...parsed].sort().join('␟')
  const b = [...props.node.blockedBy].sort().join('␟')
  return a !== b
})

function parseDeps(text: string): string[] {
  return [...new Set(
    text
      .split(/[,\s]+/)
      .map((s) => s.trim().toUpperCase())
      .filter(Boolean),
  )]
}

const metaDirty = computed(
  () =>
    priority.value !== props.node.priority ||
    qpDirty.value ||
    landmarkDirty.value ||
    depsDirty.value ||
    joinSorted(disciplines.value) !== joinSorted(props.node.disciplines) ||
    joinSorted(epics.value) !== joinSorted(props.node.epics),
)

function joinSorted(v: string[]): string {
  return [...v].sort().join('␟')
}

function saveMeta() {
  if (!metaDirty.value) return
  const edits: MetaEdits = {}
  if (priority.value !== props.node.priority) edits.priority = priority.value
  if (qpDirty.value) {
    const parsed = qpText.value.trim()
    if (parsed === '') {
      edits.clearQuestPoints = true
    } else {
      const n = Number(parsed)
      if (Number.isInteger(n) && n > 0) edits.questPoints = n
    }
  }
  if (joinSorted(disciplines.value) !== joinSorted(props.node.disciplines)) {
    edits.disciplines = [...disciplines.value]
  }
  if (joinSorted(epics.value) !== joinSorted(props.node.epics)) {
    edits.epics = [...epics.value]
  }
  if (landmarkDirty.value) {
    if (landmark.value === '') edits.clearLandmark = true
    else edits.landmark = landmark.value
  }
  if (depsDirty.value) edits.blockedBy = parseDeps(blockedByText.value)
  emit('save-meta', edits)
}

defineExpose({ metaDirty, saveMeta })
</script>

<template>
  <section class="meta-editor" aria-label="Node metadata editor">
    <h2 class="section-title">Components</h2>

    <fieldset :disabled="!canMutate || busy" class="me-fields">
      <!-- lifecycle (vanquish on a Card routes through the cascade modal) -->
      <div class="me-field">
        <span class="me-label">Status</span>
        <div class="seg">
          <button
            v-for="s in (['unstarted', 'active', 'vanquished'] as const)"
            :key="s"
            type="button"
            class="seg-btn"
            :class="{ on: node.status === s }"
            @click="emit('set-status', s)"
          >
            {{ s === 'unstarted' ? 'Unstarted' : s === 'active' ? 'Active' : 'Vanquished' }}
          </button>
        </div>
      </div>

      <!-- priority -->
      <div class="me-field">
        <span class="me-label">Priority</span>
        <div class="seg">
          <button
            v-for="p in PRIORITIES"
            :key="p.key"
            type="button"
            class="seg-btn"
            :class="{ on: priority === p.key }"
            @click="priority = p.key"
          >
            {{ p.label }}
          </button>
        </div>
      </div>

      <!-- quest points -->
      <div class="me-field">
        <span class="me-label">Quest points</span>
        <input
          v-model="qpText"
          class="me-input mono"
          type="text"
          inputmode="numeric"
          placeholder="1 2 3 5 8 13 21"
          spellcheck="false"
        />
      </div>

      <!-- landmark -->
      <div class="me-field">
        <span class="me-label">Landmark</span>
        <select v-model="landmark" class="me-input">
          <option value="">— none —</option>
          <option v-for="l in landmarkOptionsC" :key="l" :value="l">{{ l }}</option>
        </select>
      </div>

      <!-- disciplines -->
      <div class="me-field">
        <span class="me-label">Disciplines</span>
        <div class="chip-set">
          <button
            v-for="o in disciplineOptionsC"
            :key="o.key"
            type="button"
            class="chip"
            :class="{ on: o.on }"
            :title="o.key"
            @click="toggleDiscipline(o.key)"
          >
            {{ o.label }}
          </button>
          <span v-if="!disciplineOptionsC.length" class="me-none">none declared</span>
        </div>
      </div>

      <!-- epics -->
      <div class="me-field">
        <span class="me-label">Epics</span>
        <div class="chip-set">
          <button
            v-for="o in epicOptionsC"
            :key="o.key"
            type="button"
            class="chip"
            :class="{ on: o.on }"
            :title="o.key"
            @click="toggleEpic(o.key)"
          >
            {{ o.label }}
          </button>
          <span v-if="!epicOptionsC.length" class="me-none">none declared</span>
        </div>
      </div>

      <!-- blocked_by -->
      <div class="me-field">
        <span class="me-label">Blocked by (node IDs)</span>
        <input
          v-model="blockedByText"
          class="me-input mono"
          type="text"
          placeholder="ACT-3X7P, GRD-7M2Q"
          spellcheck="false"
        />
      </div>
    </fieldset>

    <div v-if="canMutate" class="me-actions">
      <button
        type="button"
        class="me-save"
        :disabled="!metaDirty || busy"
        @click="saveMeta"
      >
        {{ busy ? 'Saving…' : metaDirty ? 'Save components' : 'Saved' }}
      </button>
    </div>
    <p v-else class="me-none">Read-only preview — open a realm in the desktop shell to edit.</p>
  </section>
</template>

<style scoped>
.meta-editor {
  margin-top: 24px;
  padding-top: 18px;
  border-top: 1px solid var(--line-1);
  min-width: 0;
}

.section-title {
  margin: 0 0 12px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--faint);
  font-weight: 600;
}

.me-fields {
  display: flex;
  flex-direction: column;
  gap: 14px;
  margin: 0;
  padding: 0;
  border: 0;
  min-width: 0;
}

.me-fields:disabled {
  opacity: 0.6;
}

.me-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.me-label {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--faint);
  font-weight: 600;
}

.seg {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.seg-btn {
  padding: 2.5px 10px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  background: var(--bg-1);
  color: var(--text-2);
  font-size: 11px;
  text-transform: capitalize;
}

.seg-btn:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.seg-btn.on {
  background: var(--inset);
  border-color: var(--gold);
  color: var(--gold);
}

.me-input {
  width: 100%;
  height: 26px;
  padding: 0 9px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--inset);
  color: var(--text-1);
  font-size: 11.5px;
  outline: none;
}

.me-input:focus {
  border-color: var(--gold);
}

.chip-set {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.chip {
  padding: 2px 9px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  background: transparent;
  color: var(--text-3);
  font-size: 11px;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chip:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.chip.on {
  border-color: var(--gold);
  color: var(--gold);
  background: var(--inset);
}

.me-none {
  font-size: 11px;
  color: var(--faint);
}

.me-actions {
  margin-top: 14px;
  display: flex;
  justify-content: flex-end;
}

.me-save {
  padding: 4px 14px;
  border-radius: var(--r-m);
  border: 1px solid var(--gold);
  background: var(--inset);
  color: var(--gold);
  font-size: 11.5px;
  font-weight: 600;
}

.me-save:disabled {
  opacity: 0.45;
  cursor: default;
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}
</style>
