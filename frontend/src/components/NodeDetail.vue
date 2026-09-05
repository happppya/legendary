<script setup lang="ts">
// Detail sheet: breadcrumb path, identity row, status + effort chips, a
// blocked/validation callout, grouped component metadata and the rendered
// Markdown body. Values that carry meaning elsewhere in the realm are live:
// clicking a discipline/epic/tag/landmark filters the index; clicking a
// node id navigates the index to that node.

import { computed } from 'vue'
import { PhLock, PhWarning } from '@phosphor-icons/vue'
import type { NodeView } from '../types'
import { KIND_ICON, KIND_LABEL } from '../lib/kind'
import { mdToHtml } from '../lib/markdown'

const props = defineProps<{
  node: NodeView | null
  nodesById: Map<string, NodeView>
  realmName: string
}>()

const emit = defineEmits<{
  select: [id: string]
  filterBy: [term: string]
}>()

function go(id: string) {
  emit('select', id)
}

function filterTerm(term: string) {
  emit('filterBy', term)
}

/* ---- position in the realm ------------------------------------------- */

const ancestors = computed<NodeView[]>(() => {
  const out: NodeView[] = []
  let parent = props.node?.parent ?? null
  while (parent) {
    const n = props.nodesById.get(parent)
    if (!n) break
    out.unshift(n)
    parent = n.parent
  }
  return out
})

/* ---- status + effort -------------------------------------------------- */

function statusToken(n: NodeView): string {
  return n.effectiveStatus
}

const statusText: Record<string, string> = {
  active: 'Active',
  blocked: 'Blocked',
  unstarted: 'Unstarted',
  vanquished: 'Vanquished',
}

function effChipClass(n: NodeView): string {
  return n.blocked ? 'blocked' : statusToken(n)
}

const priorityText = (p: string) => (p ? `${p} priority` : '')

function hasOwnQp(n: NodeView): boolean {
  return n.questPoints !== null
}

function isAggregateCard(n: NodeView): boolean {
  return n.kind === 'card' && n.totalQp > 0
}

/* ---- related nodes ---------------------------------------------------- */

function labelFor(id: string): string {
  const n = props.nodesById.get(id)
  return n ? n.title : id
}

function blockedByIds(n: NodeView): string[] {
  return n.blockedBy
}

/* ---- sheet fields ----------------------------------------------------- */

function fmtDate(iso: string | null): string | null {
  if (!iso) return null
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}

const created = computed(() => fmtDate(props.node?.createdAt ?? null))
const completed = computed(() => fmtDate(props.node?.completedAt ?? null))

const sheetCount = computed(
  () =>
    (props.node?.disciplines.length ? 1 : 0) +
    (props.node?.epics.length ? 1 : 0) +
    (props.node?.tags.length ? 1 : 0) +
    (props.node?.landmark ? 1 : 0) +
    (created.value ? 1 : 0) +
    (completed.value ? 1 : 0) +
    1, // file row is always present
)

/* ---- notes ------------------------------------------------------------ */

const bodyHtml = computed(() => (props.node ? mdToHtml(props.node.body) : ''))
const hasBody = computed(() => Boolean(props.node?.body.trim()))
</script>

<template>
  <article v-if="node" class="detail" :class="node.kind">
    <!-- position -->
    <nav class="crumbs" aria-label="Position in the realm tree">
      <span class="crumb realm" :title="`Realm root`">{{ realmName }}</span>
      <template v-for="a in ancestors" :key="a.id">
        <span class="sep" aria-hidden="true">/</span>
        <button type="button" class="crumb node" :title="a.id" @click="go(a.id)">
          {{ a.title }}
        </button>
      </template>
    </nav>

    <!-- identity -->
    <header class="identity">
      <div class="title-row">
        <span
          class="kind-tile large"
          :class="node.kind"
          :title="`${KIND_LABEL[node.kind]} node`"
        >
          <component :is="KIND_ICON[node.kind]" :size="14" aria-hidden="true" />
          <span class="sr-only">{{ KIND_LABEL[node.kind] }}</span>
        </span>
        <h1>{{ node.title }}</h1>
      </div>
      <div class="id-row mono">
        <code class="id">{{ node.id }}</code>
        <span class="kind-name">{{ KIND_LABEL[node.kind] }} node</span>
      </div>

      <div v-if="node.validationError" class="callout error" role="alert">
        <PhWarning :size="14" class="callout-icon" aria-hidden="true" />
        <div class="callout-body">
          <span class="lead">Validation issue</span>
          <span class="why">{{ node.validationError }}</span>
        </div>
      </div>
    </header>

    <!-- status + effort -->
    <div class="chips-row">
      <span class="status-chip" :class="effChipClass(node)">
        <span class="status-dot" :class="effChipClass(node)" aria-hidden="true"></span>
        {{ node.blocked ? 'Blocked' : statusText[statusToken(node)] ?? node.effectiveStatus }}
      </span>
      <span v-if="node.priority" class="neutral-chip">{{ priorityText(node.priority) }}</span>
      <span v-if="hasOwnQp(node)" class="neutral-chip qp">
        {{ node.questPoints }} QP
      </span>
      <span v-if="isAggregateCard(node)" class="neutral-chip qp" :title="`Aggregated from all descendant nodes`">
        {{ node.totalQp }} QP aggregate
      </span>
      <span
        v-if="node.lockedQpPercent !== null && node.lockedQpPercent > 0"
        class="neutral-chip"
        :title="`Quest points currently locked behind blocked prerequisites`"
      >
        {{ Math.round(node.lockedQpPercent) }}% locked
      </span>
    </div>

    <div v-if="node.blocked" class="callout blocked" role="status">
      <PhLock :size="14" class="callout-icon" aria-hidden="true" />
      <div class="callout-body">
        <span class="lead">Blocked</span>
        <span class="why">
          waiting on
          <template v-for="(bid, i) in blockedByIds(node)" :key="bid">
            <span v-if="i > 0">, </span>
            <button type="button" class="link-chip mono" :title="labelFor(bid)" @click="go(bid)">
              {{ labelFor(bid) }}
            </button>
          </template>
        </span>
      </div>
    </div>

    <!-- component metadata sheet -->
    <section v-if="sheetCount > 1" class="sheet" aria-label="Node metadata">
      <div v-if="node.disciplines.length" class="field">
        <span class="field-label">Disciplines</span>
        <div class="field-value">
          <button
            v-for="d in node.disciplines"
            :key="d"
            type="button"
            class="tag-chip"
            :title="`Filter nodes by ${d}`"
            @click="filterTerm(d)"
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
            @click="filterTerm(e)"
          >
            {{ e }}
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
            @click="filterTerm(t)"
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
            @click="filterTerm(node.landmark)"
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

    <!-- body -->
    <section class="notes">
      <h2 class="section-title">Notes</h2>
      <div v-if="hasBody" class="markdown" v-html="bodyHtml"></div>
      <p v-else class="no-notes">No notes on this node yet.</p>
    </section>
  </article>

  <article v-else class="detail empty-detail">
    <div class="empty-glyph" aria-hidden="true"></div>
    <p class="empty-title">Select a node to read it.</p>
    <p class="empty-hint">
      <span><kbd>/</kbd> filter</span>
      <span><kbd>↑</kbd><kbd>↓</kbd> move</span>
      <span><kbd>←</kbd><kbd>→</kbd> collapse</span>
      <span><kbd>esc</kbd> clear</span>
    </p>
  </article>
</template>

<style scoped>
.detail {
  min-width: 0;
  overflow-y: auto;
  padding: 18px 26px 60px;
  max-width: 1080px;
}

/* ---- breadcrumb ------------------------------------------------------- */

.crumbs {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 2px;
  font-size: 11.5px;
  color: var(--text-3);
  margin-bottom: 14px;
}

.crumb {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.crumb.realm {
  text-transform: uppercase;
  letter-spacing: 0.09em;
  font-size: 10px;
  color: var(--faint);
  font-weight: 600;
}

button.crumb.node {
  color: var(--text-2);
  border-radius: var(--r-s);
  padding: 0 2px;
}

button.crumb.node:hover {
  color: var(--gold);
}

.sep {
  color: var(--line-2);
  padding: 0 2px;
  user-select: none;
}

/* ---- identity --------------------------------------------------------- */

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

.kind-tile.large {
  width: 30px;
  height: 30px;
  border-radius: var(--r-m);
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

.title-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-row h1 {
  margin: 0;
  font-size: 21px;
  line-height: 1.25;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--text-1);
  overflow-wrap: anywhere;
}

.id-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 6px;
}

.id-row .id {
  font-size: 12px;
  color: var(--gold);
  background: var(--inset);
  border: 1px solid var(--line-1);
  border-radius: var(--r-s);
  padding: 1px 7px;
}

.kind-name {
  font-size: 11px;
  color: var(--faint);
}

/* ---- callouts --------------------------------------------------------- */

.callout {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  border-radius: var(--r-l);
  padding: 10px 14px;
  margin-top: 14px;
  font-size: 12.5px;
}

.callout-icon {
  flex: none;
  margin-top: 1px;
}

.callout-body {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 8px;
  min-width: 0;
}

.callout .lead {
  font-weight: 600;
}

.callout.blocked {
  background: var(--chip-blocked-bg);
  border: 1px solid var(--chip-blocked-line);
  color: var(--chip-blocked-fg);
}

.callout.error {
  background: var(--err-bg);
  border: 1px solid var(--err-line);
  color: var(--err-fg);
}

.callout .why {
  overflow-wrap: anywhere;
  color: inherit;
}

.callout .why .link-chip {
  color: inherit;
  text-decoration: underline;
  text-underline-offset: 2px;
  text-decoration-color: var(--chip-blocked-line);
  padding: 0;
  border-radius: var(--r-s);
}

/* ---- chips ------------------------------------------------------------ */

.chips-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
}

.status-chip,
.neutral-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border-radius: 999px;
  padding: 2.5px 11px;
  font-size: 11.5px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-chip.active {
  background: var(--chip-active-bg);
  border: 1px solid var(--chip-active-line);
  color: var(--chip-active-fg);
}

.status-chip.blocked {
  background: var(--chip-blocked-bg);
  border: 1px solid var(--chip-blocked-line);
  color: var(--chip-blocked-fg);
}

.status-chip.unstarted {
  background: var(--chip-unstarted-bg);
  border: 1px solid var(--chip-unstarted-line);
  color: var(--chip-unstarted-fg);
}

.status-chip.vanquished {
  background: var(--chip-done-bg);
  border: 1px solid var(--chip-done-line);
  color: var(--chip-done-fg);
}

.status-dot.active {
  background: var(--dot-active);
}

.status-dot.blocked {
  background: var(--dot-blocked);
}

.status-dot.unstarted {
  background: var(--dot-unstarted);
}

.status-dot.vanquished {
  background: var(--dot-done);
}

.neutral-chip {
  border: 1px solid var(--line-1);
  color: var(--text-2);
  background: transparent;
}

.neutral-chip.qp {
  color: var(--gold);
  border-color: var(--line-1);
  font-variant-numeric: tabular-nums;
}

/* ---- metadata sheet --------------------------------------------------- */

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

.tag-chip,
.link-chip {
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

.tag-chip:hover,
.link-chip:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.tag-chip.landmark {
  border-style: dashed;
}

.link-chip {
  border-radius: var(--r-s);
  background: var(--inset);
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

/* ---- notes ------------------------------------------------------------ */

.notes {
  margin-top: 30px;
  padding-top: 18px;
  border-top: 1px solid var(--line-1);
}

.section-title {
  margin: 0 0 12px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--faint);
  font-weight: 600;
}

.no-notes {
  color: var(--faint);
  font-size: 12.5px;
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

.markdown a {
  text-decoration: underline;
  text-underline-offset: 2px;
}

/* ---- empty state ------------------------------------------------------ */

.empty-detail {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  padding: 0 48px;
}

.empty-glyph {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: var(--bg-2);
  position: relative;
  margin-bottom: 18px;
}

.empty-glyph::before,
.empty-glyph::after {
  content: '';
  position: absolute;
  background: var(--gold);
  border-radius: 50%;
}

.empty-glyph::before {
  width: 9px;
  height: 9px;
  left: 10px;
  top: 10px;
}

.empty-glyph::after {
  width: 6px;
  height: 6px;
  right: 12px;
  bottom: 12px;
  background: var(--line-2);
}

.empty-title {
  margin: 0 0 10px;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.empty-hint {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 16px;
  margin: 0;
  color: var(--faint);
  font-size: 12px;
}

.empty-hint kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 17px;
  height: 18px;
  margin-right: 4px;
  padding: 0 4px;
  border-radius: var(--r-s);
  border: 1px solid var(--line-2);
  background: var(--bg-1);
  color: var(--text-2);
  font-size: 10px;
}
</style>
