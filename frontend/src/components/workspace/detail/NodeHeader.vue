<script setup lang="ts">
// Identity block of the detail sheet: kind tile + title, the immutable id,
// and the live validation-error callout when the engine flagged the node.

import { PhWarning } from '@phosphor-icons/vue'
import type { NodeView } from '../../../types'
import { KIND_ICON, KIND_LABEL } from '../../../lib/kind'

defineProps<{ node: NodeView }>()
</script>

<template>
  <header class="identity">
    <div class="title-row">
      <span class="kind-tile large" :class="node.kind" :title="`${KIND_LABEL[node.kind]} node`">
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
</template>

<style scoped>
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

.callout.error {
  background: var(--err-bg);
  border: 1px solid var(--err-line);
  color: var(--err-fg);
}

.callout .why {
  overflow-wrap: anywhere;
  color: inherit;
}
</style>
