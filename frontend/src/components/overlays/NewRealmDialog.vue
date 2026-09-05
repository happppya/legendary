<script setup lang="ts">
// File → New Realm… overlay. Creating realms lands with the file-watcher
// milestone, so the form is a disabled preview that points at `legend init`.

import { PhX } from '@phosphor-icons/vue'

const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <div class="dlg-layer" role="dialog" aria-modal="true" aria-label="New realm">
    <div class="dlg-scrim" @click="emit('close')"></div>
    <div class="dlg-box">
      <header class="dlg-head">
        <h2 class="dlg-title">New Realm</h2>
        <button type="button" class="dlg-x" aria-label="Close" @click="emit('close')">
          <PhX :size="13" aria-hidden="true" />
        </button>
      </header>
      <div class="new-fields">
        <label class="path-label" for="new-name">Realm name</label>
        <input id="new-name" class="path-input" type="text" placeholder="My Game" disabled />
        <label class="path-label" for="new-loc">Location</label>
        <input id="new-loc" class="path-input mono" type="text" placeholder="choose a folder…" disabled />
      </div>
      <div class="new-note">
        <p>
          Creating realms from the app lands with the file-watcher milestone.
          From a terminal today:
        </p>
        <pre class="new-cmd">legend init path/to/my-realm</pre>
      </div>
      <div class="dlg-actions">
        <button type="button" class="btn-secondary" @click="emit('close')">Cancel</button>
        <button type="button" class="btn-primary" disabled title="Available in a later milestone">Create Realm</button>
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

.dlg-head {
  margin-bottom: 12px;
}

.dlg-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.new-fields {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.new-fields .path-input {
  margin-bottom: 8px;
}

.path-label {
  display: block;
  font-size: 11px;
  color: var(--text-3);
  margin-bottom: 5px;
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

.path-input:disabled {
  opacity: 0.5;
}

.new-note {
  border-top: 1px solid var(--line-1);
  margin-top: 4px;
  padding-top: 10px;
}

.new-note p {
  margin: 0 0 6px;
  font-size: 12px;
  color: var(--text-3);
}

.new-cmd {
  margin: 0;
  background: var(--code-bg);
  border: 1px solid var(--code-line);
  border-radius: var(--r-m);
  padding: 8px 12px;
  font-size: 11.5px;
  color: var(--text-1);
}

.dlg-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}
</style>
