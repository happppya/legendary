<script setup lang="ts">
// Settings page. Only appearance is real today (the menu + this page share
// one theme state); the remaining sections are an honest preview of the
// roadmap so the shell does not feel dead.

import { PhSun } from '@phosphor-icons/vue'
import { inDesktop } from '../../api'
import BrandMark from '../shell/BrandMark.vue'

defineProps<{
  theme: 'dark' | 'light'
  realmName: string
  nodeCount: number
}>()

const emit = defineEmits<{ 'update:theme': [t: 'dark' | 'light'] }>()

const SHORTCUTS = [
  { keys: '⌘K', label: 'Open command palette' },
  { keys: '/', label: 'Search filters' },
  { keys: '⌘O', label: 'Open realm…' },
  { keys: '⌘⇧C', label: 'Copy node ID' },
  { keys: '↑ ↓', label: 'Move through the table' },
] as const

const COMING = [
  'Interactive graph layout & pan',
  'Color overlays: epic / discipline / priority / status',
  'Editable node metadata + wrap / reparent / vanquish flows',
  'File watching + autosave indicator',
  'Landmark matrix & domain swimlanes',
] as const

const env = inDesktop() ? 'desktop shell · live IPC' : 'browser · demo fixture'
</script>

<template>
  <section class="settings" aria-label="Settings">
    <div class="settings-inner">
      <h1 class="settings-title">Settings</h1>

      <div class="card">
        <h2 class="card-title">Appearance</h2>
        <div class="radio-row" role="radiogroup" aria-label="Theme">
          <button
            type="button"
            class="radio"
            :class="{ on: theme === 'dark' }"
            role="radio"
            :aria-checked="theme === 'dark'"
            @click="emit('update:theme', 'dark')"
          >
            <span class="radio-dot" aria-hidden="true"></span>
            Night
          </button>
          <button
            type="button"
            class="radio"
            :class="{ on: theme === 'light' }"
            role="radio"
            :aria-checked="theme === 'light'"
            @click="emit('update:theme', 'light')"
          >
            <span class="radio-dot" aria-hidden="true"></span>
            Map (light)
          </button>
        </div>
      </div>

      <div class="card">
        <h2 class="card-title">Keyboard</h2>
        <ul class="shortcut-list">
          <li v-for="s in SHORTCUTS" :key="s.keys" class="shortcut">
            <kbd>{{ s.keys }}</kbd>
            <span>{{ s.label }}</span>
          </li>
        </ul>
      </div>

      <div class="card">
        <h2 class="card-title">On the roadmap</h2>
        <ul class="coming-list">
          <li v-for="c in COMING" :key="c">{{ c }}</li>
        </ul>
      </div>

      <div class="card about">
        <BrandMark :size="22" />
        <div class="about-meta">
          <p class="about-name">Legendary <span class="about-ver mono">0.1.0</span></p>
          <p class="about-line">Local-first game dev task engine — workspace preview</p>
          <p class="about-line mono about-env">{{ realmName }} · {{ nodeCount }} nodes · {{ env }}</p>
        </div>
        <span class="photon"><PhSun :size="12" aria-hidden="true" /></span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.settings {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  background: var(--bg-0);
}

.settings-inner {
  max-width: 680px;
  margin: 0 auto;
  padding: 22px 24px 60px;
}

.settings-title {
  margin: 0 0 16px;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-1);
}

.card {
  margin-bottom: 14px;
  padding: 14px 16px;
  border: 1px solid var(--line-1);
  border-radius: var(--r-l);
  background: var(--bg-1);
}

.card-title {
  margin: 0 0 10px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.11em;
  text-transform: uppercase;
  color: var(--faint);
}

.radio-row {
  display: flex;
  gap: 6px;
}

.radio {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 5px 13px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  color: var(--text-2);
  font-size: 12.5px;
}

.radio:hover {
  border-color: var(--line-2);
  color: var(--text-1);
}

.radio.on {
  border-color: var(--gold);
  background: var(--sel-bg);
  color: var(--gold-strong);
}

.radio-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  border: 2px solid var(--line-2);
}

.radio.on .radio-dot {
  border-color: var(--gold);
  background: var(--gold);
}

.shortcut-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.shortcut {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12.5px;
  color: var(--text-2);
}

.shortcut kbd {
  min-width: 56px;
  text-align: center;
  padding: 1px 7px;
  border-radius: var(--r-s);
  border: 1px solid var(--line-1);
  background: var(--bg-2);
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
  font-size: 10.5px;
  color: var(--text-1);
}

.coming-list {
  margin: 0;
  padding-left: 18px;
  color: var(--text-2);
  font-size: 12.5px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.about {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.about-meta {
  flex: 1;
}

.about-name {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-1);
}

.about-ver {
  margin-left: 6px;
  font-size: 10px;
  color: var(--faint);
}

.about-line {
  margin: 2px 0 0;
  font-size: 11.5px;
  color: var(--text-3);
}

.photon {
  color: var(--faint);
  margin-top: 4px;
}
</style>
