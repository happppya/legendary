<script setup lang="ts">
// Status + effort row of the detail sheet: the effective-status chip, any
// priority / QP chips, and the live "Blocked — waiting on…" callout whose
// prerequisite chips navigate to those nodes.

import { PhLock } from '@phosphor-icons/vue'
import type { NodeView } from '../../../types'
import { STATUS_LABEL } from '../../../lib/status'
import { isContainerKind } from '../../../lib/kind'

defineProps<{
  node: NodeView
  /** blocked_by ids resolved to titles for the callout links. */
  blockedRefs: { id: string; label: string }[]
}>()

const emit = defineEmits<{ select: [id: string] }>()

function effChipClass(n: NodeView): string {
  return n.blocked ? 'blocked' : n.effectiveStatus
}

const priorityText = (p: string) => (p ? `${p} priority` : '')

function hasOwnQp(n: NodeView): boolean {
  return n.questPoints !== null
}

function isAggregateCard(n: NodeView): boolean {
  return isContainerKind(n.kind) && n.totalQp > 0
}
</script>

<template>
  <div>
    <div class="chips-row">
      <span class="status-chip" :class="effChipClass(node)">
        <span class="status-dot" :class="effChipClass(node)" aria-hidden="true"></span>
        {{ node.blocked ? 'Blocked' : (STATUS_LABEL[node.effectiveStatus] ?? node.effectiveStatus) }}
      </span>
      <span v-if="node.priority" class="neutral-chip">{{ priorityText(node.priority) }}</span>
      <span v-if="hasOwnQp(node)" class="neutral-chip qp">
        {{ node.questPoints }} QP
      </span>
      <span v-if="isAggregateCard(node)" class="neutral-chip qp" title="Aggregated from all descendant nodes">
        {{ node.totalQp }} QP aggregate
      </span>
      <span
        v-if="node.lockedQpPercent !== null && node.lockedQpPercent > 0"
        class="neutral-chip"
        title="Quest points currently locked behind blocked prerequisites"
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
          <template v-for="(bid, i) in blockedRefs" :key="bid.id">
            <span v-if="i > 0">, </span>
            <button type="button" class="link-chip mono" :title="bid.label" @click="emit('select', bid.id)">
              {{ bid.label }}
            </button>
          </template>
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
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
</style>
