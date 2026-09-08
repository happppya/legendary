<script setup lang="ts">
// Slim application status strip along the bottom edge: realm node/link
// counts on the left, environment state on the right.

import { computed } from 'vue'

const props = defineProps<{
  nodes: number
  links: number
  realmName: string
  isMock: boolean
  /** True when mutations are available (desktop shell + live realm). */
  live: boolean
  error: string
}>()

const linkWord = computed(() => (props.links === 1 ? 'link' : 'links'))
const nodeWord = computed(() => (props.nodes === 1 ? 'node' : 'nodes'))
</script>

<template>
  <footer class="statusbar">
    <div class="sb-left">
      <span class="sb-counts mono">{{ nodes }} {{ nodeWord }} · {{ links }} {{ linkWord }}</span>
      <span v-if="error" class="sb-error" title="Last error">
        <span class="dot" aria-hidden="true"></span>
        <span class="truncate">{{ error }}</span>
      </span>
    </div>
    <div class="sb-right">
      <span
        v-if="isMock"
        class="sb-save"
        title="The demo scene is read-only — open a real realm to edit it"
      >
        <span class="dot" aria-hidden="true"></span>
        demo scene
      </span>
      <span v-else-if="live" class="sb-save" title="Edits write through to the realm's node files and index">
        <span class="dot" aria-hidden="true"></span>
        live · {{ realmName }}
      </span>
      <span v-else class="sb-env mono">
        <span class="dot muted" aria-hidden="true"></span>
        read-only preview · {{ realmName }}
      </span>
    </div>
  </footer>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  height: 24px;
  flex: none;
  padding: 0 10px 0 12px;
  background: var(--status-bg);
  border-top: 1px solid var(--titlebar-line);
  color: var(--text-3);
  font-size: 10.5px;
}

.sb-left,
.sb-right {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  color: var(--faint);
}

.sb-save {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-2);
}

.sb-env {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.sb-error {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--err-fg);
  max-width: 40vw;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--dot-active);
  flex: none;
}

.dot.muted {
  background: var(--dot-unstarted);
}

.sb-error .dot {
  background: var(--dot-blocked);
}
</style>
