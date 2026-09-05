<script setup lang="ts">
// The Legendary logomark: a three-node graph glyph (matches desktop/icons).
defineProps<{ size?: number }>()
</script>

<template>
  <svg
    class="brand-mark"
    :width="size ?? 26"
    :height="size ?? 26"
    viewBox="0 0 64 64"
    fill="none"
    aria-hidden="true"
  >
    <line x1="28" y1="20" x2="18.5" y2="41" stroke="var(--mark-line)" stroke-width="6" stroke-linecap="round" />
    <line x1="36" y1="20" x2="45.5" y2="41" stroke="var(--mark-line)" stroke-width="6" stroke-linecap="round" />
    <line x1="20" y1="42.5" x2="44" y2="42.5" stroke="var(--mark-line)" stroke-width="6" stroke-linecap="round" />
    <circle cx="32" cy="17" r="9" fill="var(--mark-accent)" />
    <circle cx="17" cy="44" r="8" fill="var(--mark-node)" />
    <circle cx="47" cy="44" r="8" fill="var(--mark-node)" />
  </svg>
</template>

<style scoped>
.brand-mark {
  display: block;
  flex: none;
}
</style>
