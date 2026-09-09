<script setup lang="ts">
// Editable focused-node workspace (doc 05 §5.1 left panel): the title and
// the Markdown body are edited in place and saved through the shared ops
// layer on blur / Ctrl+Enter (title renames re-derive the file slug per
// doc 03 §3.3). Read-only in the browser fixture preview.
//
// Local state is seeded from the node and re-seeded when the identity or the
// incoming content changes (external edits, snapshot swaps) — never while
// the user is mid-edit on the same content.

import { computed, ref, watch } from 'vue'
import { PhFloppyDisk, PhPencilSimple } from '@phosphor-icons/vue'
import type { NodeView } from '../../../types'
import { mdToHtml } from '../../../lib/markdown'

const props = defineProps<{
  node: NodeView
  canMutate: boolean
  /** Pending save state (IPC round-trip in flight). */
  busy: boolean
}>()

const emit = defineEmits<{
  'save-body': [body: string]
  'save-title': [title: string]
}>()

const editing = ref(false)
const draft = ref('')
const draftTitle = ref('')
/** True while the pointer hovers the rendered Markdown preview (shows the
 * floating Edit affordance). */
const hovered = ref(false)
/** Content the draft was seeded from, to detect external changes. */
let seededBody = ''
let seededTitle = ''
let seededId = ''

function seed(n: NodeView) {
  seededId = n.id
  seededBody = n.body
  seededTitle = n.title
  draft.value = n.body
  draftTitle.value = n.title
  editing.value = false
}

watch(
  () => props.node,
  (n) => {
    if (!n) return
    if (n.id !== seededId) {
      seed(n)
      return
    }
    if (!editing.value) {
      seed(n)
    } else {
      // Mid-edit: only pull external changes that actually differ from the
      // seeded content (avoids clobbering the draft with our own echo).
      if (n.body !== seededBody) {
        seededBody = n.body
        draft.value = n.body
      }
      if (n.title !== seededTitle) {
        seededTitle = n.title
        draftTitle.value = n.title
      }
    }
  },
  { immediate: true },
)

const dirty = computed(
  () => draft.value !== props.node.body || draftTitle.value.trim() !== props.node.title,
)
const titleDirty = computed(() => draftTitle.value.trim() !== props.node.title)

function startEdit() {
  if (!props.canMutate) return
  draft.value = props.node.body
  draftTitle.value = props.node.title
  seededBody = props.node.body
  seededTitle = props.node.title
  editing.value = true
}

function cancelEdit() {
  editing.value = false
  draft.value = props.node.body
  draftTitle.value = props.node.title
}

function saveTitle() {
  const t = draftTitle.value.trim()
  if (!t || t === props.node.title) return
  emit('save-title', t)
}

function saveBody() {
  if (draft.value === props.node.body) return
  emit('save-body', draft.value)
}

/** Ctrl+Enter inside the textarea saves; Escape cancels. */
function onEditorKey(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    e.preventDefault()
    saveBody()
    editing.value = false
  } else if (e.key === 'Escape') {
    e.preventDefault()
    cancelEdit()
  }
}

function onTitleKey(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    ;(e.target as HTMLInputElement | null)?.blur()
  } else if (e.key === 'Escape') {
    e.preventDefault()
    draftTitle.value = props.node.title
  }
}

</script>

<template>
  <section class="body-editor" aria-label="Node notes editor">
    <header class="be-head">
      <h2 class="section-title">
        <PhPencilSimple v-if="editing" :size="11" aria-hidden="true" />
        {{ editing ? 'Editing notes' : 'Notes' }}
      </h2>
      <div class="be-tools">
        <template v-if="canMutate">
          <button
            v-if="!editing"
            type="button"
            class="be-btn"
            @click="startEdit"
          >
            Edit
          </button>
          <template v-else>
            <button type="button" class="be-btn" :disabled="busy" @click="saveBody">
              <PhFloppyDisk :size="11" aria-hidden="true" />
              {{ busy ? 'Saving…' : 'Save' }}
            </button>
            <button type="button" class="be-btn ghost" :disabled="busy" @click="cancelEdit">
              Cancel
            </button>
          </template>
        </template>
      </div>
    </header>

    <!-- title editing (rename → re-derived slug, doc 03 §3.3) -->
    <input
      v-if="canMutate"
      v-model="draftTitle"
      class="be-title"
      :class="{ dirty: titleDirty }"
      type="text"
      aria-label="Node title"
      spellcheck="false"
      @keydown="onTitleKey"
      @blur="saveTitle"
    />
    <h1 v-else class="be-title ro">{{ node.title }}</h1>

    <!-- body: textarea while editing, preview otherwise. The rendered
         Markdown carries a hover Edit affordance (test-feedback A-2: "no
         edit button popup when hovering over MD description"). -->
    <textarea
      v-if="editing"
      v-model="draft"
      class="be-input mono"
      spellcheck="false"
      aria-label="Markdown body editor"
      @keydown="onEditorKey"
    ></textarea>
    <div
      v-else-if="node.body.trim()"
      class="be-preview"
      @mouseenter="hovered = true"
      @mouseleave="hovered = false"
    >
      <div class="markdown" v-html="mdToHtml(node.body)"></div>
      <button
        v-if="canMutate"
        type="button"
        class="be-hover-edit"
        :class="{ on: hovered }"
        title="Edit notes"
        aria-label="Edit notes"
        @click="startEdit"
      >
        <PhPencilSimple :size="11" aria-hidden="true" />
        Edit
      </button>
    </div>
    <p v-else class="be-empty">No notes on this node yet.</p>

    <p v-if="editing" class="be-hint">
      <kbd>Ctrl</kbd>+<kbd>Enter</kbd> save · <kbd>Esc</kbd> cancel
    </p>
    <p v-if="dirty && !editing" class="be-hint">Unsaved changes…</p>
  </section>
</template>

<style scoped>
.body-editor {
  margin-top: 30px;
  padding-top: 18px;
  border-top: 1px solid var(--line-1);
  min-width: 0;
}

.be-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-title {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  margin: 0;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--faint);
  font-weight: 600;
}

.be-tools {
  display: flex;
  gap: 5px;
}

.be-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 9px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--bg-1);
  color: var(--text-2);
  font-size: 11px;
}

.be-btn:hover:not(:disabled) {
  border-color: var(--gold);
  color: var(--gold);
}

.be-btn:disabled {
  opacity: 0.55;
  cursor: default;
}

.be-btn.ghost {
  border-color: transparent;
  background: transparent;
}

.be-title {
  width: 100%;
  margin: 0 0 10px;
  padding: 3px 8px;
  border: 1px solid transparent;
  border-radius: var(--r-m);
  background: transparent;
  color: var(--text-1);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: -0.01em;
  line-height: 1.3;
  overflow-wrap: anywhere;
}

.be-title.ro {
  padding: 0;
}

.be-title:hover {
  border-color: var(--line-1);
}

.be-title:focus {
  outline: none;
  border-color: var(--gold);
  background: var(--inset);
}

.be-title.dirty {
  border-color: var(--gold);
}

.be-input {
  width: 100%;
  min-height: 200px;
  resize: vertical;
  padding: 10px 12px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-2);
  background: var(--inset);
  color: var(--text-1);
  font-size: 12.5px;
  line-height: 1.6;
  outline: none;
}

.be-input:focus {
  border-color: var(--gold);
}

.be-empty {
  margin: 0;
  color: var(--faint);
  font-size: 12.5px;
}

.be-hint {
  display: flex;
  align-items: center;
  gap: 3px;
  margin: 8px 0 0;
  font-size: 10.5px;
  color: var(--faint);
}

.be-hint kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 16px;
  height: 16px;
  padding: 0 3px;
  border-radius: var(--r-s);
  border: 1px solid var(--line-2);
  background: var(--bg-1);
  color: var(--text-2);
  font-size: 9px;
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}

.be-preview {
  position: relative;
  min-width: 0;
}

.be-hover-edit {
  position: absolute;
  top: 6px;
  right: 6px;
  z-index: 4;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--line-2);
  background: var(--bg-1);
  color: var(--text-2);
  font-size: 11px;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.28);
  opacity: 0;
  transform: translateY(-3px);
  pointer-events: none;
  transition: opacity 0.12s ease, transform 0.12s ease;
}

.be-preview:hover .be-hover-edit {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
}

.be-hover-edit:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.markdown {
  font-size: 13.5px;
  line-height: 1.62;
  color: var(--text-2);
  max-width: 860px;
}

.markdown > p {
  max-width: 74ch;
}

.markdown h1,
.markdown h2,
.markdown h3,
.markdown h4 {
  color: var(--text-1);
  line-height: 1.3;
  margin: 1.5em 0 0.5em;
  font-weight: 600;
}

.markdown h1 {
  font-size: 1.35em;
}

.markdown h2 {
  font-size: 1.2em;
}

.markdown h3 {
  font-size: 1.08em;
}

.markdown h4 {
  font-size: 1em;
}

.markdown pre {
  background: var(--code-bg);
  border: 1px solid var(--code-line);
  border-radius: var(--r-m);
  padding: 12px 14px;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-1);
}

.markdown code {
  font-size: 0.9em;
  background: var(--code-bg);
  border: 1px solid var(--line-1);
  border-radius: var(--r-s);
  padding: 0.05em 0.35em;
}

.markdown pre code {
  background: none;
  border: 0;
  padding: 0;
}

.markdown blockquote {
  border-left: 2px solid var(--gold);
  margin: 1em 0;
  padding: 0.1em 0 0.1em 16px;
  color: var(--text-3);
}

.markdown li {
  margin: 0.2em 0;
}

.markdown hr {
  border: 0;
  border-top: 1px solid var(--line-1);
  margin: 1.6em 0;
}
</style>
