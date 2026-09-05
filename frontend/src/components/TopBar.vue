<script setup lang="ts">
// The window titlebar: logomark only (no wordmark), the app menu bar, a
// centred command-palette pill and window controls on the right. The bar is
// marked as a drag region for the frameless desktop window; the buttons
// below it stay interactive.

import { onMounted, ref } from 'vue'
import { inDesktop } from '../api'
import BrandMark from './BrandMark.vue'
import MenuBar, { type MenuSpec } from './MenuBar.vue'

defineProps<{ menus: MenuSpec[] }>()
const emit = defineEmits<{ run: [id: string]; palette: [] }>()

const isDesktop = ref(inDesktop())

onMounted(() => {
  isDesktop.value = inDesktop()
})

async function windowApi() {
  if (!inDesktop()) return null
  const mod = await import('@tauri-apps/api/window')
  return mod.getCurrentWindow()
}

async function onMinimize() {
  const win = await windowApi()
  await win?.minimize()
}

async function onMaximize() {
  const win = await windowApi()
  if (!win) return
  if (await win.isMaximized()) await win.unmaximize()
  else await win.maximize()
}

async function onClose() {
  const win = await windowApi()
  await win?.close()
}

function onDblClick(e: MouseEvent) {
  if ((e.target as HTMLElement | null)?.closest('button, input, [role="menuitem"]')) return
  if (isDesktop.value) void onMaximize()
}
</script>

<template>
  <header class="topbar" data-tauri-drag-region @dblclick="onDblClick">
    <div class="topbar-left">
      <button
        type="button"
        class="brand-btn"
        title="Legendary — local-first game dev task engine"
        aria-label="Legendary"
        @click="emit('palette')"
      >
        <BrandMark :size="24" />
      </button>
      <MenuBar :menus="menus" @run="emit('run', $event)" />
    </div>

    <div class="topbar-mid">
      <button type="button" class="palette" @click="emit('palette')">
        <span class="palette-kbd" aria-hidden="true">⌘K</span>
        <span class="palette-label">Command Palette&hellip;</span>
        <span class="palette-grip" aria-hidden="true"></span>
      </button>
    </div>

    <div class="topbar-right" data-tauri-drag-region>
      <div class="window-controls" :class="{ web: !isDesktop }">
        <button
          type="button"
          class="wc"
          :title="isDesktop ? 'Minimize' : 'Window control (desktop shell)'"
          aria-label="Minimize"
          @click="isDesktop ? onMinimize() : undefined"
        >
          <i class="glyph min"></i>
        </button>
        <button
          type="button"
          class="wc"
          :title="isDesktop ? 'Maximize' : 'Window control (desktop shell)'"
          aria-label="Maximize"
          @click="isDesktop ? onMaximize() : undefined"
        >
          <i class="glyph max"></i>
        </button>
        <button
          type="button"
          class="wc close"
          :title="isDesktop ? 'Close' : 'Window control (desktop shell)'"
          aria-label="Close"
          @click="isDesktop ? onClose() : undefined"
        >
          <i class="glyph close-x"></i>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  height: 44px;
  flex: none;
  padding: 0 10px;
  background: var(--titlebar-bg);
  border-bottom: 1px solid var(--titlebar-line);
  user-select: none;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  z-index: 45;
}

.brand-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border-radius: var(--r-m);
  color: inherit;
}

.brand-btn:hover {
  background: var(--bg-2);
}

.topbar-mid {
  flex: 1;
  display: flex;
  justify-content: center;
  min-width: 0;
}

.palette {
  display: flex;
  align-items: center;
  gap: 8px;
  width: min(340px, 44vw);
  height: 27px;
  padding: 0 9px;
  border-radius: var(--r-m);
  background: var(--palette-bg);
  border: 1px solid var(--line-1);
  color: var(--palette-fg);
  font-size: 12px;
  transition: border-color 0.12s ease, background-color 0.12s ease;
}

.palette:hover {
  border-color: var(--line-2);
  background: var(--palette-bg-hover);
}

.palette-kbd {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 5px;
  height: 16px;
  border-radius: var(--r-s);
  border: 1px solid var(--line-1);
  background: var(--bg-2);
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  color: var(--faint);
}

.palette-label {
  color: var(--palette-fg);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.palette-grip {
  margin-left: auto;
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  border-top: 5px solid var(--faint);
  opacity: 0.7;
}

.topbar-right {
  display: flex;
  justify-content: flex-end;
  min-width: 0;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 2px;
}

.window-controls.web {
  opacity: 0.75;
}

.wc {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 26px;
  border-radius: var(--r-s);
  color: var(--text-3);
  background: transparent;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.wc:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.wc.close:hover {
  background: var(--danger-soft-bg);
  color: var(--danger-soft-fg);
}

.glyph {
  position: relative;
  display: block;
  width: 10px;
  height: 10px;
}

.glyph.min::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  top: 4px;
  height: 1px;
  background: currentColor;
}

.glyph.max {
  width: 9px;
  height: 9px;
  border: 1px solid currentColor;
  border-radius: 1px;
}

.glyph.close-x::before,
.glyph.close-x::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 0;
  width: 1.5px;
  height: 10px;
  background: currentColor;
  border-radius: 1px;
}

.glyph.close-x::before {
  transform: rotate(45deg);
}

.glyph.close-x::after {
  transform: rotate(-45deg);
}
</style>
