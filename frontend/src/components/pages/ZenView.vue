<script setup lang="ts">
// Zen mode (test-feedback Item C): a read-only, full-bleed visual dashboard
// for the whole realm. The shared force-directed scene (scene.ts / physics.ts
// / edges.ts) is reused as-is — no duplicated layout code — but rendered in a
// minimalist high-tech style: geometric node shapes (squares, diamonds,
// circles), centre-to-centre edges, and a soft glow on every stroke.
//
// Read-only by design: no selection sync, no mutations, no editors. Hovering
// a node reveals its title; the corner HUD carries the realm's vital
// statistics. Escape or the × exits back to the workspace.

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { PhX } from '@phosphor-icons/vue'
import type { NodeView } from '../../types'
import { KIND_LABEL, isContainerKind } from '../../lib/kind'
import { applyForces } from '../workspace/graph/physics'
import {
  buildSceneItems,
  sceneSignature,
  type SceneItem,
} from '../workspace/graph/scene'
import { drawEdges, linkedPairs, sceneEdges } from '../workspace/graph/edges'

const props = defineProps<{
  nodes: NodeView[]
  nodesById: Map<string, NodeView>
  realmName: string
}>()

const emit = defineEmits<{
  close: []
}>()

/* ---- scene: reuse the shared simulator verbatim ------------------------- */

const live = ref<SceneItem[]>([])
let seededSig = ''

function syncScene() {
  const sig = sceneSignature({
    isMockScene: false,
    nodesById: props.nodesById,
    selectedId: null, // whole-realm scope: never re-centre on a selection
    scope: 'global',
  })
  if (sig === seededSig) return
  seededSig = sig
  live.value = buildSceneItems({
    isMockScene: false,
    nodesById: props.nodesById,
    selectedId: null,
    scope: 'global',
  })
}

onMounted(syncScene)
watch(() => props.nodesById, syncScene)

const sceneItems = computed<SceneItem[]>(() => live.value)

/** Scene member keys — restricts edge discovery to the visible scene. */
const keyMap = computed(() => new Map(live.value.map((i) => [i.key, true] as const)))

/* The shared simulator keeps the layout alive so edits elsewhere re-settle
 * here too. Zen mode is read-only: run it, don't interact. */
let rafId = 0
let lastStep = 0
const hoverKey = ref<string | null>(null)

const linked = computed(() =>
  linkedPairs(sceneEdges(false, props.nodesById, keyMap.value)),
)

function step(timestamp: number) {
  rafId = 0
  const dt = Math.min(0.05, Math.max(0.005, (timestamp - lastStep) / 1000 || 0.016))
  lastStep = timestamp
  applyForces(live.value, null, linked.value, dt)
  rafId = requestAnimationFrame(step)
}

function ensureLoop() {
  if (rafId || live.value.length < 2) return
  lastStep = performance.now()
  rafId = requestAnimationFrame(step)
}

watch(sceneItems, (items) => {
  if (items.length >= 2) ensureLoop()
})

onBeforeUnmount(() => {
  if (rafId) cancelAnimationFrame(rafId)
  rafId = 0
})

/* ---- edges: centre-to-centre (drawEdges' zen mode) ----------------------- */

const edges = computed(() =>
  drawEdges(live.value, sceneEdges(false, props.nodesById, keyMap.value), true),
)

/* ---- zen geometry: one shape family per kind ----------------------------- */

/** Zen stroke color per kind, from the shared kind palette. */
const KIND_VAR: Record<string, string> = {
  card: 'var(--kind-card)',
  genre: 'var(--kind-genre)',
  action: 'var(--kind-action)',
  guard: 'var(--kind-guard)',
  idea: 'var(--kind-idea)',
}

/** Zen stroke color per kind, from the shared kind palette; blocked nodes
 * glow warm instead. Set as `color` on each node group — the shapes and
 * their glow both derive from `currentColor`, so stroke + halo always match. */
function statusColor(n: NodeView): string {
  if (n.effectiveStatus === 'blocked') return 'var(--chip-blocked-fg)'
  return KIND_VAR[n.kind] ?? 'var(--graph-line)'
}

/** Status tint class: vanquished dims, blocked glows warm, the rest stay
 * neutral — the only per-node data zen surfaces. */
function statusClass(n: NodeView): string {
  if (n.effectiveStatus === 'vanquished') return 'is-done'
  if (n.effectiveStatus === 'blocked') return 'is-blocked'
  if (n.effectiveStatus === 'active') return 'is-active'
  return 'is-unstarted'
}

/** Guards are diamonds; actions and ideas are circles; cards and genres are
 * squares (genres slightly larger and soft-filled). */
function isDiamond(n: NodeView): boolean {
  return n.kind === 'guard'
}

function isCircle(n: NodeView): boolean {
  return n.kind === 'action' || n.kind === 'idea'
}

/** Half-extent (square) or radius (circle/diamond) of the zen shape. */
function rOf(n: NodeView): number {
  if (isCircle(n)) return 15
  if (isDiamond(n)) return 20
  return isContainerKind(n.kind) ? 26 : 22
}

/** Dynamic viewBox: the scene world is unbounded, so fit the viewport to
 * the members' bounding box (with padding) every layout tick. */
const fit = computed(() => {
  const items = live.value
  if (!items.length) return { x: -500, y: -350, w: 1000, h: 700 }
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity
  for (const it of items) {
    const r = rOf(it.node) + 6
    minX = Math.min(minX, it.x - r)
    minY = Math.min(minY, it.y - r)
    maxX = Math.max(maxX, it.x + r)
    maxY = Math.max(maxY, it.y + r)
  }
  // square-ish padding so the breathing vignette stays centred
  const pad = 90
  return {
    x: minX - pad,
    y: minY - pad,
    w: Math.max(200, maxX - minX + pad * 2),
    h: Math.max(160, maxY - minY + pad * 2),
  }
})

const viewBox = computed(() => `${fit.value.x} ${fit.value.y} ${fit.value.w} ${fit.value.h}`)

/* ---- HUD: minimal realm vitals ------------------------------------------ */

const HUD = computed(() => {
  const byStatus = new Map<string, number>()
  for (const n of props.nodes) {
    byStatus.set(n.effectiveStatus, (byStatus.get(n.effectiveStatus) ?? 0) + 1)
  }
  const totalQp = props.nodes.reduce(
    (s, n) => s + (isContainerKind(n.kind) ? 0 : (n.questPoints ?? 0)),
    0,
  )
  return {
    nodes: props.nodes.length,
    links: edges.value.length,
    totalQp,
    active: byStatus.get('active') ?? 0,
    blocked: byStatus.get('blocked') ?? 0,
  }
})

/** Truncation for the hover caption. */
function trunc(title: string, n: number): string {
  return title.length > n ? `${title.slice(0, n - 1)}…` : title
}

/* ---- exit: × button or Escape -------------------------------------------- */

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
  }
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <section class="zen" aria-label="Zen mode dashboard">
    <!-- vignette backdrop -->
    <div class="zen-bg" aria-hidden="true"></div>

    <!-- the shared scene, re-lit -->
    <div class="zen-canvas">
      <svg
        class="zen-svg"
        :viewBox="viewBox"
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="Zen realm graph — read-only"
      >
        <!-- centre-to-centre edges (drawn under the nodes) -->
        <g class="zen-edges">
          <line
            v-for="(e, i) in edges"
            :key="i"
            :x1="e.x1"
            :y1="e.y1"
            :x2="e.x2"
            :y2="e.y2"
            class="z-edge"
            :class="{ 'z-edge-dep': e.dep, 'z-edge-dashed': e.dashed }"
          />
        </g>

        <!-- geometric node shapes -->
        <g
          v-for="item in sceneItems"
          :key="item.key"
          class="z-node"
          :class="[statusClass(item.node), { hovered: hoverKey === item.key }]"
          :style="{ color: statusColor(item.node) }"
          @pointerenter="hoverKey = item.key"
          @pointerleave="hoverKey = null"
        >
          <title>{{ KIND_LABEL[item.node.kind] }} — {{ item.node.title }} ({{ item.node.effectiveStatus }})</title>

          <!-- diamonds (guards) -->
          <path
            v-if="isDiamond(item.node)"
            class="z-shape"
            :d="`M ${item.x} ${item.y - rOf(item.node)} L ${item.x + rOf(item.node)} ${item.y} L ${item.x} ${item.y + rOf(item.node)} L ${item.x - rOf(item.node)} ${item.y} Z`"
          />
          <!-- circles (actions + ideas) -->
          <circle
            v-else-if="isCircle(item.node)"
            class="z-shape"
            :cx="item.x"
            :cy="item.y"
            :r="rOf(item.node)"
          />
          <!-- squares (cards + genres) -->
          <rect
            v-else
            class="z-shape"
            :x="item.x - rOf(item.node)"
            :y="item.y - rOf(item.node)"
            :width="rOf(item.node) * 2"
            :height="rOf(item.node) * 2"
            rx="10"
          />

          <!-- hover caption: rendered in-scene so it tracks the node even
               while the layout breathes -->
          <g v-if="hoverKey === item.key" class="z-caption">
            <text class="zc-kind" :x="item.x" :y="item.y - rOf(item.node) - 24" text-anchor="middle">
              {{ KIND_LABEL[item.node.kind] }} · {{ item.node.effectiveStatus }}
            </text>
            <text class="zc-title" :x="item.x" :y="item.y - rOf(item.node) - 9" text-anchor="middle">
              {{ trunc(item.node.title, 44) }}
            </text>
          </g>
        </g>
      </svg>
    </div>

    <!-- corner HUD: realm name + exit, minimal vitals -->
    <header class="zen-top">
      <div class="zt-left">
        <span class="zt-realm">{{ realmName }}</span>
        <span class="zt-mode">zen</span>
      </div>
      <button type="button" class="zt-close" aria-label="Exit zen mode" @click="emit('close')">
        <PhX :size="14" aria-hidden="true" />
      </button>
    </header>

    <footer class="zen-hud">
      <div class="zh-stat">
        <span class="zh-num">{{ HUD.nodes }}</span>
        <span class="zh-label">nodes</span>
      </div>
      <span class="zh-sep" aria-hidden="true"></span>
      <div class="zh-stat">
        <span class="zh-num">{{ HUD.links }}</span>
        <span class="zh-label">links</span>
      </div>
      <span class="zh-sep" aria-hidden="true"></span>
      <div class="zh-stat">
        <span class="zh-num">{{ HUD.totalQp }}</span>
        <span class="zh-label">QP</span>
      </div>
      <span class="zh-sep" aria-hidden="true"></span>
      <div class="zh-stat">
        <span class="zh-num zh-active">{{ HUD.active }}</span>
        <span class="zh-label">active</span>
      </div>
      <span class="zh-sep" aria-hidden="true"></span>
      <div class="zh-stat">
        <span class="zh-num zh-blocked">{{ HUD.blocked }}</span>
        <span class="zh-label">blocked</span>
      </div>
    </footer>
  </section>
</template>

<style scoped>
.zen {
  position: absolute;
  inset: 0;
  z-index: 30;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: radial-gradient(
    120% 90% at 50% 42%,
    #101825 0%,
    #0a0e15 58%,
    #060a10 100%
  );
  color: var(--text-2);
}

/* Slow breathing vignette keeps the dark field from feeling flat. */
.zen-bg {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(42% 34% at 50% 44%, rgba(224, 169, 92, 0.05), transparent 70%),
    radial-gradient(70% 55% at 50% 50%, transparent 55%, rgba(3, 5, 9, 0.55) 100%);
}

.zen-canvas {
  position: absolute;
  inset: 0;
}

.zen-svg {
  width: 100%;
  height: 100%;
  display: block;
}

/* ---- glow ---------------------------------------------------------------- */

/* The node group carries the kind/status color; shapes + glow both derive
 * from currentColor, so stroke and halo always match. */
.z-node {
  cursor: default;
}

.z-shape {
  fill: rgba(10, 14, 21, 0.55);
  stroke: currentColor;
  stroke-width: 1.6;
  filter: drop-shadow(0 0 6px currentColor);
  transition: filter 0.18s ease, stroke-width 0.18s ease;
}

/* Hover: intensify the glow, never the data. */
.z-node.hovered .z-shape {
  stroke-width: 2.4;
  filter: drop-shadow(0 0 12px currentColor) drop-shadow(0 0 3px currentColor);
}

.z-edge {
  stroke: rgba(148, 166, 198, 0.35);
  stroke-width: 1.1;
  filter: drop-shadow(0 0 3px rgba(148, 166, 198, 0.35));
}

.z-edge-dep {
  stroke: rgba(224, 169, 92, 0.45);
  filter: drop-shadow(0 0 3px rgba(224, 169, 92, 0.4));
}

.z-edge-dashed {
  stroke-dasharray: 3 6;
}

/* Status tints: dim the finished; blocked warmth comes from statusColor. */
.z-node.is-done {
  opacity: 0.38;
}

.z-node.is-active .z-shape {
  stroke-width: 1.9;
}

/* ---- in-scene hover caption ------------------------------------------------ */

.z-caption text {
  font-family: inherit;
  paint-order: stroke;
  stroke: rgba(6, 10, 16, 0.9);
  stroke-width: 3px;
}

.zc-kind {
  fill: currentColor;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.zc-title {
  fill: var(--text-1);
  font-size: 12px;
  font-weight: 600;
}

/* ---- HUD chrome ------------------------------------------------------------ */

.zen-top {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  pointer-events: none;
}

.zt-left {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  pointer-events: auto;
}

.zt-realm {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--text-3);
}

.zt-mode {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--gold);
  padding: 2px 8px;
  border: 1px solid rgba(224, 169, 92, 0.35);
  border-radius: 999px;
}

.zt-close {
  pointer-events: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: var(--r-m);
  border: 1px solid rgba(148, 166, 198, 0.22);
  color: var(--text-3);
  background: rgba(8, 12, 18, 0.5);
}

.zt-close:hover {
  color: var(--text-1);
  border-color: rgba(224, 169, 92, 0.5);
}

.zen-hud {
  position: absolute;
  bottom: 18px;
  left: 50%;
  transform: translateX(-50%);
  display: inline-flex;
  align-items: center;
  gap: 16px;
  padding: 8px 18px;
  border-radius: 999px;
  border: 1px solid rgba(148, 166, 198, 0.16);
  background: rgba(8, 12, 18, 0.62);
  backdrop-filter: blur(4px);
  pointer-events: none;
}

.zh-stat {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
}

.zh-num {
  font-size: 14px;
  font-weight: 700;
  font-family: 'IBM Plex Mono', ui-monospace, Consolas, monospace;
  color: var(--text-1);
}

.zh-num.zh-active {
  color: var(--chip-active-fg);
}

.zh-num.zh-blocked {
  color: var(--chip-blocked-fg);
}

.zh-label {
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--faint);
}

.zh-sep {
  width: 1px;
  height: 14px;
  background: rgba(148, 166, 198, 0.18);
}

@media (prefers-reduced-motion: reduce) {
  .z-shape,
  .z-edge {
    filter: none;
  }
}
</style>
