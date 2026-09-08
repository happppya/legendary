<script setup lang="ts">
// Entities overview: one panel per node kind with the doc-02 description and
// a preview of its nodes. "Browse all" jumps to the workspace table with the
// kind filter applied.

import { computed } from 'vue'
import type { NodeView } from '../../types'
import { KIND_ICON, KIND_LABEL, KINDS, isContainerKind } from '../../lib/kind'
import type { CheckRow } from '../../lib/filters'

const props = defineProps<{ nodes: NodeView[]; kindCounts: CheckRow[] }>()
const emit = defineEmits<{ browse: [kind: string] }>()

const DESCRIPTIONS: Record<string, string> = {
  card: 'Container / group nodes. Hold documentation, system context and child nodes.',
  genre: 'Broad categorization branches grouping whole system families.',
  action: 'Single, short, clearly defined execution goals with quest points.',
  guard: 'Quality gates — code review, QA pass, build verification.',
  idea: 'Speculative pitches and concept exploration.',
}

const groups = computed(() =>
  KINDS.map((k) => ({
    kind: k,
    label: KIND_LABEL[k],
    desc: DESCRIPTIONS[k],
    count: props.kindCounts.find((r) => r.key === k)?.count ?? 0,
    nodes: props.nodes.filter((n) => n.kind === k).slice(0, 6),
    total: props.nodes.filter((n) => n.kind === k).length,
  })),
)
</script>

<template>
  <section class="entities" aria-label="Entities">
    <div class="ent-grid">
      <article v-for="g in groups" :key="g.kind" class="ent-card" :class="g.kind">
        <header class="ent-head">
          <span class="ent-tile" :class="g.kind">
            <component :is="KIND_ICON[g.kind]" :size="15" aria-hidden="true" />
          </span>
          <h2 class="ent-title">{{ g.label }}s</h2>
          <span class="ent-count mono">{{ g.count }}</span>
        </header>
        <p class="ent-desc">{{ g.desc }}</p>
        <ul class="ent-list">
          <li v-for="n in g.nodes" :key="n.id" class="ent-item">
            <i class="sdot" :class="`dot-${n.effectiveStatus}`" aria-hidden="true"></i>
            <span class="ent-name" :title="n.title">{{ n.title }}</span>
            <span v-if="isContainerKind(n.kind) && n.totalQp" class="ent-qp mono">{{ n.totalQp }}</span>
            <span v-else-if="n.questPoints !== null" class="ent-qp mono">{{ n.questPoints }}</span>
          </li>
          <li v-if="!g.nodes.length" class="ent-empty">No {{ g.label.toLowerCase() }}s yet.</li>
        </ul>
        <button v-if="g.total > 6" type="button" class="ent-more" @click="emit('browse', g.kind)">
          Browse all {{ g.total }} {{ g.label.toLowerCase() }}s
        </button>
      </article>
    </div>
  </section>
</template>

<style scoped>
.entities {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  background: var(--bg-0);
  padding: 18px;
}

.ent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 14px;
  align-content: start;
  max-width: 1240px;
  margin: 0 auto;
}

.ent-card {
  border: 1px solid var(--line-1);
  border-radius: var(--r-l);
  background: var(--bg-1);
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
}

.ent-head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.ent-tile {
  width: 28px;
  height: 28px;
  border-radius: var(--r-m);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--badge-ink);
}

.ent-tile.card {
  background: var(--kind-card);
}

.ent-tile.genre {
  background: var(--kind-genre);
}

.ent-tile.action {
  background: var(--kind-action);
}

.ent-tile.guard {
  background: var(--kind-guard);
}

.ent-tile.idea {
  background: var(--kind-idea);
}

.ent-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-1);
}

.ent-count {
  margin-left: auto;
  font-size: 11px;
  color: var(--faint);
}

.ent-desc {
  margin: 8px 0 10px;
  font-size: 12px;
  color: var(--text-3);
  line-height: 1.45;
}

.ent-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ent-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 3px 6px;
  border-radius: var(--r-s);
  font-size: 12.5px;
  color: var(--text-2);
  min-width: 0;
}

.ent-item:hover {
  background: var(--bg-2);
}

.sdot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex: none;
}

.dot-unstarted {
  background: var(--dot-unstarted);
}

.dot-active {
  background: var(--dot-active);
}

.dot-blocked {
  background: var(--dot-blocked);
}

.dot-vanquished {
  background: var(--dot-done);
}

.ent-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ent-qp {
  flex: none;
  font-size: 10px;
  color: var(--gold);
}

.ent-empty {
  color: var(--faint);
  font-size: 12px;
  padding: 4px 6px;
}

.ent-more {
  margin-top: 10px;
  padding: 5px 8px;
  border-radius: var(--r-s);
  color: var(--gold);
  font-size: 11.5px;
  align-self: flex-start;
}

.ent-more:hover {
  background: var(--sel-bg);
}
</style>
