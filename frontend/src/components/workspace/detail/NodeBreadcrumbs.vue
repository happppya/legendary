<script setup lang="ts">
// Position strip inside the detail sheet: realm root, then the parent chain
// down to (but excluding) the node itself. Ancestor crumbs navigate.

import type { NodeView } from '../../../types'

defineProps<{
  realmName: string
  ancestors: NodeView[]
}>()

const emit = defineEmits<{ select: [id: string] }>()
</script>

<template>
  <nav class="crumbs" aria-label="Position in the realm tree">
    <span class="crumb realm" title="Realm root">{{ realmName }}</span>
    <template v-for="a in ancestors" :key="a.id">
      <span class="sep" aria-hidden="true">/</span>
      <button type="button" class="crumb node" :title="a.id" @click="emit('select', a.id)">
        {{ a.title }}
      </button>
    </template>
  </nav>
</template>

<style scoped>
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
</style>
