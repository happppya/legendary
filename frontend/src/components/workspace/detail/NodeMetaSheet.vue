<script setup lang="ts">
// Grouped component metadata of the detail sheet. Values that carry meaning
// elsewhere in the realm are live: clicking a discipline/epic/tag/landmark
// filters the index.

import { computed } from 'vue'
import type { NodeView } from '../../../types'

const props = defineProps<{ node: NodeView }>()

const emit = defineEmits<{
  filter: [{ field: 'discipline' | 'epic' | 'tag' | 'landmark'; value: string }]
}>()

function filterTerm(field: 'discipline' | 'epic' | 'tag' | 'landmark', value: string) {
  emit('filter', { field, value })
}

/* ---- sheet fields ----------------------------------------------------- */

function fmtDate(iso: string | null): string | null {
  if (!iso) return null
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}

const created = computed(() => fmtDate(props.node.createdAt ?? null))
const completed = computed(() => fmtDate(props.node.completedAt ?? null))

/** The sheet hides entirely when only the file row would show. */
const showSheet = computed(
  () =>
    (props.node.disciplines.length ? 1 : 0) +
      (props.node.epics.length ? 1 : 0) +
      (props.node.tags.length ? 1 : 0) +
      (props.node.landmark ? 1 : 0) +
      (created.value ? 1 : 0) +
      (completed.value ? 1 : 0) >
    0,
)
</script>

<template>
  <section v-if="showSheet" class="sheet" aria-label="Node metadata">
    <div v-if="node.disciplines.length" class="field">
      <span class="field-label">Disciplines</span>
      <div class="field-value">
        <button
          v-for="d in node.disciplines"
          :key="d"
          type="button"
          class="tag-chip"
          :title="`Filter nodes by ${d}`"
          @click="filterTerm('discipline', d)"
        >
          {{ d }}
        </button>
      </div>
    </div>

    <div v-if="node.epics.length" class="field">
      <span class="field-label">Epics</span>
      <div class="field-value">
        <button
          v-for="e in node.epics"
          :key="e"
          type="button"
          class="tag-chip"
          :title="`Filter nodes by ${e}`"
          @click="filterTerm('epic', e)"
        >
          {{ e.split('/').pop() }}
        </button>
      </div>
    </div>

    <div v-if="node.tags.length" class="field">
      <span class="field-label">Tags</span>
      <div class="field-value">
        <button
          v-for="t in node.tags"
          :key="t"
          type="button"
          class="tag-chip"
          :title="`Filter nodes tagged ${t}`"
          @click="filterTerm('tag', t)"
        >
          #{{ t }}
        </button>
      </div>
    </div>

    <div v-if="node.landmark" class="field">
      <span class="field-label">Landmark</span>
      <div class="field-value">
        <button
          type="button"
          class="tag-chip landmark"
          :title="`Filter nodes in landmark ${node.landmark}`"
          @click="filterTerm('landmark', node.landmark)"
        >
          {{ node.landmark }}
        </button>
      </div>
    </div>

    <div v-if="created" class="field">
      <span class="field-label">Created</span>
      <div class="field-value"><span class="date mono">{{ created }}</span></div>
    </div>

    <div v-if="completed" class="field">
      <span class="field-label">Completed</span>
      <div class="field-value"><span class="date mono">{{ completed }}</span></div>
    </div>

    <div class="field file">
      <span class="field-label">Source file</span>
      <div class="field-value"><span class="path mono">{{ node.path }}</span></div>
    </div>
  </section>
</template>

<style scoped>
.sheet {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
  gap: 16px 30px;
  margin-top: 22px;
  padding-top: 18px;
  border-top: 1px solid var(--line-1);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.field-label {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--faint);
  font-weight: 600;
}

.field-value {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  align-items: center;
  min-width: 0;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 100%;
  border-radius: var(--r-m);
  padding: 2px 8px;
  font-size: 11.5px;
  border: 1px solid var(--line-1);
  color: var(--text-2);
  background: var(--bg-1);
  overflow-wrap: anywhere;
}

.tag-chip:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.tag-chip.landmark {
  border-style: dashed;
}

.date {
  font-size: 11.5px;
  color: var(--text-2);
}

.field.file .path {
  font-size: 11px;
  color: var(--text-3);
  overflow-wrap: anywhere;
  word-break: break-all;
}
</style>
