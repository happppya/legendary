<script setup lang="ts">
// The Notes body of the detail sheet: renders the node's Markdown through
// the shared dependency-free preview renderer (lib/markdown.ts).

import { computed } from 'vue'
import type { NodeView } from '../../../types'
import { mdToHtml } from '../../../lib/markdown'

const props = defineProps<{ node: NodeView }>()

const bodyHtml = computed(() => mdToHtml(props.node.body))
const hasBody = computed(() => Boolean(props.node.body.trim()))
</script>

<template>
  <section class="notes">
    <h2 class="section-title">Notes</h2>
    <div v-if="hasBody" class="markdown" v-html="bodyHtml"></div>
    <p v-else class="no-notes">No notes on this node yet.</p>
  </section>
</template>

<style scoped>
.notes {
  margin-top: 30px;
  padding-top: 18px;
  border-top: 1px solid var(--line-1);
}

.section-title {
  margin: 0 0 12px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--faint);
  font-weight: 600;
}

.no-notes {
  color: var(--faint);
  font-size: 12.5px;
}

.markdown {
  font-size: 13.5px;
  line-height: 1.62;
  color: var(--text-2);
  max-width: 860px;
}

.markdown > p {
  max-width: 74ch;
}

.markdown h1,
.markdown h2,
.markdown h3,
.markdown h4 {
  color: var(--text-1);
  line-height: 1.3;
  margin: 1.5em 0 0.5em;
  font-weight: 600;
}

.markdown h1 {
  font-size: 1.35em;
}

.markdown h2 {
  font-size: 1.2em;
}

.markdown h3 {
  font-size: 1.08em;
}

.markdown h4 {
  font-size: 1em;
}

.markdown pre {
  background: var(--code-bg);
  border: 1px solid var(--code-line);
  border-radius: var(--r-m);
  padding: 12px 14px;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-1);
}

.markdown code {
  font-size: 0.9em;
  background: var(--code-bg);
  border: 1px solid var(--line-1);
  border-radius: var(--r-s);
  padding: 0.05em 0.35em;
}

.markdown pre code {
  background: none;
  border: 0;
  padding: 0;
}

.markdown blockquote {
  border-left: 2px solid var(--gold);
  margin: 1em 0;
  padding: 0.1em 0 0.1em 16px;
  color: var(--text-3);
}

.markdown li {
  margin: 0.2em 0;
}

.markdown hr {
  border: 0;
  border-top: 1px solid var(--line-1);
  margin: 1.6em 0;
}

.markdown a {
  text-decoration: underline;
  text-underline-offset: 2px;
}
</style>
