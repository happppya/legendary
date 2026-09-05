<script setup lang="ts">
// File → Open Realm… overlay: recent-realm picker (including the demo realm)
// plus a path input for realms opened live from the desktop shell.

import { computed } from 'vue'
import { PhX } from '@phosphor-icons/vue'

const props = defineProps<{
  /** Current realm path (marks the row that is open right now). */
  realmPath: string
  /** True when the demo realm is the current source. */
  isMock: boolean
  mockName: string
  mockPath: string
  recents: { label: string; path: string }[]
  busy: boolean
  error: string
  isDesktop: boolean
  path: string
}>()

const emit = defineEmits<{
  close: []
  'use-mock': []
  'open-recent': [recent: { label: string; path: string }]
  submit: []
  'update:path': [value: string]
}>()

const path = computed({
  get: () => props.path,
  set: (v: string) => emit('update:path', v),
})
</script>

<template>
  <div class="dlg-layer" role="dialog" aria-modal="true" aria-label="Open realm">
    <div class="dlg-scrim" @click="emit('close')"></div>
    <div class="dlg-box">
      <header class="dlg-head">
        <h2 class="dlg-title">Open Realm</h2>
        <button type="button" class="dlg-x" aria-label="Close" @click="emit('close')">
          <PhX :size="13" aria-hidden="true" />
        </button>
      </header>

      <div class="recent-list">
        <p class="recent-label">Recent realms</p>
        <button
          type="button"
          class="recent-row"
          :class="{ current: isMock }"
          @click="emit('use-mock')"
        >
          <span class="recent-name">Demo realm — {{ mockName }}</span>
          <span class="recent-path mono">{{ mockPath }}</span>
          <span v-if="isMock" class="current-tag">current</span>
        </button>
        <button
          v-for="r in recents"
          :key="r.path"
          type="button"
          class="recent-row"
          :class="{ current: !isMock && realmPath === r.path }"
          @click="emit('open-recent', r)"
        >
          <span class="recent-name">{{ r.label }}</span>
          <span class="recent-path mono">{{ r.path }}</span>
          <span v-if="!isMock && realmPath === r.path" class="current-tag">current</span>
        </button>
        <p v-if="!recents.length" class="recent-empty">
          No other realms opened yet.
        </p>
      </div>

      <form class="path-form" @submit.prevent="emit('submit')">
        <label class="path-label" for="realm-path">Realm path</label>
        <div class="path-row">
          <input
            id="realm-path"
            v-model="path"
            class="path-input mono"
            type="text"
            spellcheck="false"
            placeholder="path/to/realm"
          />
          <button type="submit" class="btn-primary" :disabled="busy">
            {{ busy ? 'Opening…' : 'Open' }}
          </button>
        </div>
        <p v-if="error" class="dlg-error">{{ error }}</p>
        <p v-if="!isDesktop" class="dlg-hint">
          In the browser preview only the checked-in demo fixture is available.
        </p>
      </form>
    </div>
  </div>
</template>

<style scoped>
.dlg-layer {
  position: fixed;
  inset: 0;
  z-index: 300;
}

.dlg-scrim {
  position: absolute;
  inset: 0;
  background: rgba(4, 7, 12, 0.46);
  backdrop-filter: blur(1px);
}

.dlg-box {
  position: relative;
  width: min(520px, 90vw);
  margin: 14vh auto 0;
  background: var(--bg-1);
  border: 1px solid var(--line-2);
  border-radius: var(--r-l);
  box-shadow: 0 14px 44px rgba(0, 0, 0, 0.4);
  padding: 18px 20px;
}

.dlg-x {
  position: absolute;
  top: 12px;
  right: 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.dlg-x:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.dlg-head {
  margin-bottom: 12px;
}

.dlg-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.recent-label {
  margin: 4px 0 6px;
  font-size: 9.5px;
  font-weight: 600;
  letter-spacing: 0.11em;
  text-transform: uppercase;
  color: var(--faint);
}

.recent-list {
  max-height: 190px;
  overflow-y: auto;
  margin-bottom: 14px;
}

.recent-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 7px 9px;
  border-radius: var(--r-m);
  color: var(--text-2);
  text-align: left;
}

.recent-row:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.recent-row.current {
  background: var(--sel-bg);
}

.recent-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12.5px;
}

.recent-path {
  flex: none;
  font-size: 10px;
  color: var(--faint);
}

.current-tag {
  flex: none;
  font-size: 9px;
  color: var(--gold);
  border: 1px solid var(--gold);
  border-radius: 999px;
  padding: 0 6px;
}

.recent-empty {
  margin: 2px 0;
  font-size: 11.5px;
  color: var(--faint);
}

.path-form {
  border-top: 1px solid var(--line-1);
  padding-top: 12px;
}

.path-label {
  display: block;
  font-size: 11px;
  color: var(--text-3);
  margin-bottom: 5px;
}

.path-row {
  display: flex;
  gap: 8px;
}

.path-input {
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 10px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--inset);
  color: var(--text-1);
  font-size: 12px;
  outline: none;
}

.path-input:focus {
  border-color: var(--gold);
}

.dlg-error {
  margin: 8px 0 0;
  color: var(--err-fg);
  font-size: 12px;
  overflow-wrap: anywhere;
}

.dlg-hint {
  margin: 8px 0 0;
  color: var(--faint);
  font-size: 11px;
}
</style>
