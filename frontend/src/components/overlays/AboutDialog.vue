<script setup lang="ts">
// Help → About Legendary overlay.

import { PhX } from '@phosphor-icons/vue'
import BrandMark from '../shell/BrandMark.vue'

defineProps<{ isDesktop: boolean }>()
const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <div class="dlg-layer" role="dialog" aria-modal="true" aria-label="About Legendary">
    <div class="dlg-scrim" @click="emit('close')"></div>
    <div class="dlg-box about-box">
      <button type="button" class="dlg-x" aria-label="Close" @click="emit('close')">
        <PhX :size="13" aria-hidden="true" />
      </button>
      <BrandMark :size="30" />
      <h2 class="about-title">Legendary</h2>
      <p class="about-sub">Local-first, Markdown-native graph task engine for game development.</p>
      <p class="about-line mono">
        version 0.1.0 · {{ isDesktop ? 'desktop shell (live IPC)' : 'web preview (demo fixture)' }}
      </p>
      <p class="about-notes">
        The workspace shell is a preview: read-only realm data, mock graph
        scene, editor milestone features coming next.
      </p>
      <div class="about-actions">
        <button type="button" class="btn-primary" @click="emit('close')">Done</button>
      </div>
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

.about-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 4px;
}

.about-box .brand-mark {
  margin-bottom: 6px;
}

.about-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.about-sub {
  margin: 0;
  font-size: 12.5px;
  color: var(--text-2);
  max-width: 360px;
}

.about-line {
  font-size: 10.5px;
  color: var(--faint);
}

.about-notes {
  font-size: 11.5px;
  color: var(--text-3);
  max-width: 380px;
  margin: 4px 0 10px;
}

.about-actions {
  margin-top: 4px;
}
</style>
