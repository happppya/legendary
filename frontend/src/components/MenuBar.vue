<script setup lang="ts">
// Horizontal app menu bar with dropdown menus and one level of submenus,
// styled like a native desktop menu bar. Each leaf item carries a unique
// `id`; activating it emits `run(id)` so the shell can route it.
//
// Interaction model matches desktop menus: click opens a menu, hovering a
// neighbouring top-level menu while one is open moves the open menu, Escape
// or an outside click closes.

import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { PhCaretRight } from '@phosphor-icons/vue'

export interface MenuItem {
  id: string
  label?: string
  accel?: string
  disabled?: boolean
  checked?: boolean
  separator?: boolean
  title?: string
  /** Presence of items turns this row into a submenu. */
  items?: MenuItem[]
}

export interface MenuSpec {
  id: string
  label: string
  items: MenuItem[]
}

const props = defineProps<{ menus: MenuSpec[] }>()
const emit = defineEmits<{ run: [id: string] }>()

const openId = ref<string | null>(null)
const subOf = ref<string | null>(null)
const popLeft = ref(0)
const rootEls = ref<HTMLElement[]>([])

function anchorFor(index: number): number {
  return rootEls.value[index]?.offsetLeft ?? 0
}

function toggleMenu(id: string, index: number) {
  if (openId.value === id) {
    closeMenus()
    return
  }
  openId.value = id
  subOf.value = null
  popLeft.value = anchorFor(index)
}

function closeMenus() {
  openId.value = null
  subOf.value = null
}

function onRowEnter(m: MenuSpec, index: number) {
  // Once a menu is open, hovering its neighbours moves the open menu there.
  if (openId.value && openId.value !== m.id) {
    openId.value = m.id
    subOf.value = null
    popLeft.value = anchorFor(index)
  }
}

function run(item: MenuItem) {
  if (item.disabled || item.separator || item.items) return
  emit('run', item.id)
  closeMenus()
}

function onKeydown(e: KeyboardEvent) {
  if (!openId.value) return
  if (e.key === 'Escape') {
    if (subOf.value) {
      subOf.value = null
    } else {
      closeMenus()
    }
    e.preventDefault()
  }
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))

const openMenu = computed(() => props.menus.find((m) => m.id === openId.value) ?? null)

function setRootEl(el: unknown, i: number) {
  if (el instanceof HTMLElement) rootEls.value[i] = el
}
</script>

<template>
  <nav class="menu-bar" role="menubar" aria-label="Application menu">
    <div
      v-for="(m, i) in menus"
      :key="m.id"
      :ref="(el) => setRootEl(el, i)"
      class="menu-root"
      :class="{ open: openId === m.id }"
    >
      <button
        type="button"
        class="menu-label"
        :aria-haspopup="'menu'"
        :aria-expanded="openId === m.id"
        @click="toggleMenu(m.id, i)"
        @mouseenter="onRowEnter(m, i)"
      >
        {{ m.label }}
      </button>
    </div>
    <div v-if="openMenu" class="menu-scrim" @mousedown="closeMenus"></div>

    <transition name="pop">
      <div
        v-if="openMenu"
        class="menu-pop"
        role="menu"
        :style="{ left: `${popLeft}px` }"
        @mouseleave="subOf = null"
      >
        <template v-for="item in openMenu.items" :key="item.id">
          <div v-if="item.separator" class="sep"></div>
          <div
            v-else
            class="menu-row"
            :class="{
              disabled: item.disabled,
              submenu: !!item.items,
              open: subOf === item.id,
              checked: item.checked,
            }"
            role="menuitem"
            :aria-disabled="item.disabled || undefined"
            @mouseenter="item.items && (subOf = item.id)"
            @click="run(item)"
          >
            <span class="row-label">
              <span v-if="item.checked !== undefined" class="check" aria-hidden="true">
                {{ item.checked ? '✓' : '' }}
              </span>
              <span class="label-text">{{ item.label }}</span>
            </span>
            <span v-if="item.items" class="chev"><PhCaretRight :size="11" aria-hidden="true" /></span>
            <span v-else-if="item.accel" class="accel">{{ item.accel }}</span>

            <div v-if="item.items && subOf === item.id" class="sub-pop" role="menu">
              <template v-for="sub in item.items" :key="sub.id">
                <div v-if="sub.separator" class="sep"></div>
                <div
                  v-else
                  class="menu-row"
                  :class="{ disabled: sub.disabled, checked: sub.checked }"
                  role="menuitem"
                  :aria-disabled="sub.disabled || undefined"
                  @click="run(sub)"
                >
                  <span class="row-label">
                    <span v-if="sub.checked !== undefined" class="check" aria-hidden="true">
                      {{ sub.checked ? '✓' : '' }}
                    </span>
                    <span class="label-text">{{ sub.label }}</span>
                  </span>
                  <span v-if="sub.accel" class="accel">{{ sub.accel }}</span>
                </div>
              </template>
            </div>
          </div>
        </template>
      </div>
    </transition>
  </nav>
</template>

<style scoped>
.menu-bar {
  position: relative;
  display: flex;
  align-items: center;
  gap: 2px;
  z-index: 40;
}

.menu-label {
  appearance: none;
  border: 0;
  background: transparent;
  color: var(--text-2);
  font-size: 12.5px;
  font-weight: 500;
  letter-spacing: 0.005em;
  padding: 4px 9px;
  border-radius: var(--r-m);
  transition: background-color 0.12s ease, color 0.12s ease;
}

.menu-label:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.menu-root.open > .menu-label {
  background: var(--bg-2);
  color: var(--text-1);
}

/* Invisible layer that swallows clicks outside an open menu */
.menu-scrim {
  position: fixed;
  inset: 0;
  z-index: 0;
}

.menu-pop {
  position: absolute;
  top: calc(100% + 4px);
  z-index: 50;
  min-width: 244px;
  padding: 4px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  box-shadow: var(--shadow-1);
}

.pop-enter-active,
.pop-leave-active {
  transition: opacity 0.08s ease, transform 0.08s ease;
}

.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: translateY(-2px);
}

.sep {
  height: 1px;
  margin: 4px 6px;
  background: var(--line-1);
}

.menu-row {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 4px 8px;
  border-radius: var(--r-s);
  font-size: 12.5px;
  color: var(--text-1);
  cursor: pointer;
  user-select: none;
}

.menu-row:hover:not(.disabled),
.menu-row.open {
  background: var(--bg-2);
}

.menu-row.disabled {
  cursor: default;
}

.menu-row.disabled .label-text {
  color: var(--faint);
}

.menu-row:not(.disabled) .row-label:hover {
  color: inherit;
}

.row-label {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  min-width: 0;
}

.check {
  width: 12px;
  flex: none;
  text-align: center;
  font-size: 10px;
  color: var(--gold);
}

.label-text {
  white-space: nowrap;
  color: inherit;
}

.accel {
  flex: none;
  font-size: 10.5px;
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
  color: var(--faint);
}

.chev {
  flex: none;
  color: var(--text-3);
  display: inline-flex;
}

.menu-row.checked .label-text {
  color: var(--gold);
}

.sub-pop {
  position: absolute;
  top: -5px;
  left: calc(100% + 6px);
  min-width: 232px;
  padding: 4px;
  background: var(--bg-1);
  border: 1px solid var(--line-1);
  border-radius: var(--r-m);
  box-shadow: var(--shadow-1);
}

.muted {
  color: var(--faint);
}

@media (prefers-reduced-motion: reduce) {
  .pop-enter-active,
  .pop-leave-active {
    transition: none;
  }
}
</style>
