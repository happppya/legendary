<script setup lang="ts">
// Reusable quest-point picker (test-feedback Bug3): one-click Fibonacci
// options (the recommended amounts, single-sourced in lib/qp.ts) plus a
// free-form 0–9999 text box. Used by the node create dialog and the
// metadata editor so both estimate entry points look and validate alike.
// The v-model carries the raw text ('' = no estimate); validation errors
// are surfaced by the parent through the `invalid` prop.

import { QP_CHOICES } from '../../lib/qp'

const model = defineModel<string>({ required: true })

defineProps<{
  /** Shows the range error outline + message. */
  invalid?: boolean
  /** Containers aggregate children QP and cannot carry their own estimate. */
  disabled?: boolean
}>()
</script>

<template>
  <div v-if="!disabled" class="qp-row" role="group" aria-label="Quest points">
    <button
      v-for="q in QP_CHOICES"
      :key="q"
      type="button"
      class="qp-choice"
      :class="{ on: model === String(q) }"
      @click="model = String(q)"
    >
      {{ q }}
    </button>
    <input
      v-model="model"
      class="qp-input mono"
      :class="{ invalid }"
      type="text"
      inputmode="numeric"
      placeholder="custom 0–9999"
      spellcheck="false"
      aria-label="Custom quest points (0–9999)"
    />
  </div>
  <span v-else class="qp-none">— aggregate over children —</span>
  <span v-if="invalid && !disabled" class="qp-err">Quest points must be a whole number between 0 and 9999.</span>
</template>

<style scoped>
.qp-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
}

.qp-choice {
  min-width: 26px;
  padding: 2.5px 7px;
  border-radius: 999px;
  border: 1px solid var(--line-1);
  background: transparent;
  color: var(--text-2);
  font-size: 11px;
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}

.qp-choice:hover {
  border-color: var(--gold);
  color: var(--gold);
}

.qp-choice.on {
  background: var(--inset);
  border-color: var(--gold);
  color: var(--gold);
}

.qp-input {
  width: 104px;
  flex: none;
  height: 24px;
  padding: 0 9px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: var(--inset);
  color: var(--text-1);
  font-size: 11px;
  outline: none;
}

.qp-input:focus {
  border-color: var(--gold);
}

.qp-input.invalid {
  border-color: var(--err-line, #c2554f);
}

.qp-none {
  font-size: 11px;
  color: var(--faint);
}

.qp-err {
  font-size: 10.5px;
  color: var(--err-fg, #d07a74);
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
}
</style>
