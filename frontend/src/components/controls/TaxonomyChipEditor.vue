<script setup lang="ts">
// Reusable assigned-chips editor for string-vocabulary components (epics,
// disciplines; test-feedback Bug4). Assigned values render as chips with a
// hover ✕ to remove; new values are added through a ➕ dropdown listing only
// values not yet assigned, so duplicates are impossible by construction.
// Used twice inside NodeMetaEditor; keep it here if another view needs the
// same interaction.

import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { PhCaretDown, PhPlus, PhX } from '@phosphor-icons/vue'

const props = defineProps<{
  /** Assigned values (the draft being edited). */
  values: string[]
  /** Realm vocabulary: every value the user could assign. */
  options: { key: string; label: string }[]
  /** aria-label prefix for the remove buttons / add button. */
  noun: string
}>()

const emit = defineEmits<{
  add: [key: string]
  remove: [key: string]
}>()

/** Vocabulary not yet assigned — the dropdown entries. */
const choices = computed(() => {
  const have = new Set(props.values)
  return props.options.filter((o) => !have.has(o.key))
})

function labelOf(key: string): string {
  return props.options.find((o) => o.key === key)?.label ?? key
}

const menuOpen = ref(false)

function toggleMenu() {
  menuOpen.value = !menuOpen.value
}

function add(key: string) {
  emit('add', key)
  menuOpen.value = false
}

/** Close the dropdown on any outside click (per-instance document listener). */
function onDocClick(e: MouseEvent) {
  const el = e.target instanceof HTMLElement ? e.target : null
  if (!el?.closest('.chip-adder')) menuOpen.value = false
}
onMounted(() => document.addEventListener('click', onDocClick))
onBeforeUnmount(() => document.removeEventListener('click', onDocClick))
</script>

<template>
  <div class="chip-set">
    <span v-for="v in values" :key="v" class="chip assigned" :title="v">
      {{ labelOf(v) }}
      <button
        type="button"
        class="chip-x"
        :aria-label="`Remove ${noun} ${v}`"
        @click="emit('remove', v)"
      >
        <PhX :size="9" aria-hidden="true" />
      </button>
    </span>
    <span v-if="!values.length" class="chip-none">none</span>
    <span class="chip-adder">
      <button
        type="button"
        class="add-btn"
        :class="{ on: menuOpen }"
        :disabled="!choices.length"
        :title="choices.length ? `Add ${noun}` : `All declared ${noun}s are assigned`"
        :aria-label="`Add ${noun}`"
        @click.stop="toggleMenu"
      >
        <PhPlus :size="11" aria-hidden="true" />
        <PhCaretDown :size="8" aria-hidden="true" />
      </button>
      <span v-if="menuOpen && choices.length" class="add-menu" role="menu">
        <button
          v-for="o in choices"
          :key="o.key"
          type="button"
          role="menuitem"
          class="add-item"
          :title="o.key"
          @click="add(o.key)"
        >
          {{ o.label }}
        </button>
      </span>
    </span>
  </div>
</template>

<style scoped>
.chip-set {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
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

.chip.assigned {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border-color: var(--gold);
  color: var(--gold);
  background: var(--inset);
}

.chip-x {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  flex: none;
  margin-right: -3px;
  border-radius: 50%;
  color: inherit;
  opacity: 0;
  transition: opacity 0.1s ease;
}

.chip.assigned:hover .chip-x,
.chip-x:focus-visible {
  opacity: 0.85;
}

.chip-x:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.12);
}

.chip-none {
  font-size: 11px;
  color: var(--faint);
}

.chip-adder {
  position: relative;
  display: inline-flex;
}

.add-btn {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  width: auto;
  padding: 2px 6px;
  border-radius: 999px;
  border: 1px dashed var(--line-2);
  background: transparent;
  color: var(--text-3);
}

.add-btn:hover:not(:disabled),
.add-btn.on {
  border-color: var(--gold);
  color: var(--gold);
}

.add-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

.add-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 30;
  display: flex;
  flex-direction: column;
  min-width: 180px;
  max-height: 220px;
  overflow-y: auto;
  padding: 4px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-2);
  background: var(--bg-1);
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.35);
}

.add-item {
  padding: 4px 8px;
  border-radius: var(--r-s);
  text-align: left;
  color: var(--text-2);
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.add-item:hover {
  background: var(--bg-2);
  color: var(--gold);
}
</style>
