<script setup lang="ts">
// Local graph view (workspace top half). A lightweight force-directed
// simulator (springs along the edges, charge repulsion between nodes, weak
// centre gravity - the shape the design doc's graph engine will take, see
// doc 07) auto-lays the scene out. Pause it to freeze, drag any node to
// re-arrange the neighbourhood, and hit "auto-layout" to scatter it again.
//
// Node shapes follow design doc 05: Cards are squares, Actions pills,
// Guards diamonds and Ideas dashed pills. Clicking a node drives the shared
// selection, breadcrumb and inspector.

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  PhArrowsOut,
  PhArrowClockwise,
  PhMinus,
  PhPause,
  PhPlay,
  PhPlus,
  PhDotsThreeOutlineVertical,
} from '@phosphor-icons/vue'
import type { NodeView } from '../types'
import { KIND_LABEL } from '../lib/kind'
import { MOCK_BY_KEY, SCENE_HEIGHT, SCENE_LINKS, SCENE_SPOTS, SCENE_WIDTH } from '../lib/mockRealm'

const props = defineProps<{
  nodesById: Map<string, NodeView>
  selectedId: string | null
  isMockScene: boolean
  realmName: string
}>()

const emit = defineEmits<{ select: [id: string] }>()

/* ---- sizes per kind (centred at x,y) ---------------------------------- */

const KIND_SIZE: Record<string, { w: number; h: number }> = {
  card: { w: 190, h: 56 },
  action: { w: 172, h: 30 },
  idea: { w: 172, h: 30 },
  guard: { w: 46, h: 46 },
}

interface SceneItem {
  key: string
  node: NodeView
  x: number
  y: number
  w: number
  h: number
  vx: number
  vy: number
}

/** Node metadata + live physics state, keyed by scene spot key. */
const live = ref<SceneItem[]>([])

const cx = SCENE_WIDTH / 2
const cy = SCENE_HEIGHT / 2

/** Whole-realm scenes above this node count wait for the Wasm physics
 * engine (doc 07 / Milestone 5) instead of the prototype simulator. */
const MAX_SCENE_NODES = 200

/** Deterministic scatter for realm-wide scenes (ring around the centre). */
function scatterPosition(key: string, index: number, total: number) {
  const base = Math.min(SCENE_WIDTH, SCENE_HEIGHT) * 0.42
  const ang = (index / total) * Math.PI * 2 + seedN(key) * 1.4
  const rad = base * (0.5 + 0.9 * seedN(key + 'r'))
  return {
    x: cx + Math.cos(ang) * rad,
    y: cy + Math.sin(ang) * rad,
    vx: (seedN(key + 'vx') - 0.5) * 2.4,
    vy: (seedN(key + 'vy') - 0.5) * 2.4,
  }
}

function seed(): SceneItem[] {
  const out: SceneItem[] = []
  if (props.isMockScene) {
    // Curated local scene around Character Movement Core.
    for (const spot of SCENE_SPOTS) {
      const id = MOCK_BY_KEY.get(spot.key)?.id ?? spot.key
      const node = props.nodesById.get(id)
      if (!node) continue
      const size = KIND_SIZE[node.kind] ?? KIND_SIZE.action
      // deterministic tiny jitter so the first auto-layout visibly settles
      const jitter = (seedN(spot.key) - 0.5) * 14
      out.push({
        key: spot.key,
        node,
        x: spot.x,
        y: spot.y + jitter,
        w: size.w,
        h: size.h,
        vx: (seedN(spot.key + 'x') - 0.5) * 1.6,
        vy: (seedN(spot.key + 'y') - 0.5) * 1.6,
      })
    }
    return out
  }
  // Real realm: force-lay the whole realm tree, seeded on a ring so the
  // simulator visibly settles into the dependency structure.
  const size = props.nodesById.size
  if (size === 0 || size > MAX_SCENE_NODES) return out
  let i = 0
  for (const node of props.nodesById.values()) {
    const dim = KIND_SIZE[node.kind] ?? KIND_SIZE.action
    const p = scatterPosition(node.id, i, size)
    out.push({
      key: node.id,
      node,
      x: p.x,
      y: p.y,
      w: dim.w,
      h: dim.h,
      vx: p.vx,
      vy: p.vy,
    })
    i += 1
  }
  return out
}

/** Cheap deterministic pseudo-random for stable-but-varied starts. */
function seedN(s: string): number {
  let h = 2166136261
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return ((h >>> 0) % 1000) / 1000
}

/** Scene signature - re-seed only when the member set really changes (realm
 * open/reload, or the mock/local scene swaps). Dragging and auto-layout do
 * not touch it, so a settled layout survives navigation. */
function sceneSig(): string {
  if (props.isMockScene) {
    return 'm|' + SCENE_SPOTS.map((s) => s.key).sort().join('|')
  }
  return 'r|' + [...props.nodesById.keys()].sort().join('|')
}

let seededSig = ''
function syncScene() {
  const sig = sceneSig()
  if (sig === seededSig) return
  seededSig = sig
  live.value = seed()
  if (live.value.length >= 2) ensureLoop()
}

onMounted(syncScene)
watch(() => [props.isMockScene, props.nodesById], syncScene)

const sceneItems = computed<SceneItem[]>(() => live.value)

/* ---- force simulation ---------------------------------------------------- */

const playing = ref(true)
let rafId = 0
let lastStep = 0

// Spring / repulsion tuning for a calm, readable settle.
const REST = 165
const SPRING = 0.006
const REPULSION = 26000
const GRAVITY = 0.0009
const DAMPING = 0.88
const MAX_SPEED = 5

interface SceneLinkDef {
  from: string
  to: string
  dashed?: boolean
  /** A true prerequisite edge (blocked_by), drawn with a dependency arrow. */
  dep?: boolean
}

/** Directed edges for the current scene: the curated mock links, or - for a
 * real realm - every parent edge plus every blocked_by dependency edge that
 * points at a node present in the payload. */
function sceneEdges(): SceneLinkDef[] {
  if (props.isMockScene) return SCENE_LINKS
  const out: SceneLinkDef[] = []
  for (const [id, node] of props.nodesById) {
    if (node.parent && props.nodesById.has(node.parent)) out.push({ from: node.parent, to: id })
    for (const b of node.blockedBy) {
      if (props.nodesById.has(b)) out.push({ from: b, to: id, dep: true })
    }
  }
  return out
}

// Linked pairs share a Hooke spring toward REST; everything else just
// repels, so connected clusters stay tight while unrelated nodes part.
const LINKED = computed(() => {
  const s = new Set<string>()
  for (const l of sceneEdges()) s.add(`${l.from}␟${l.to}`)
  return s
})

function step(timestamp: number) {
  rafId = 0
  const dt = Math.min(0.05, Math.max(0.005, (timestamp - lastStep) / 1000 || 0.016))
  lastStep = timestamp
  if (playing.value) applyForces(dt)
  if (playing.value) rafId = requestAnimationFrame(step)
}

function applyForces(dt: number) {
  const items = live.value
  if (items.length < 2) return
  const dragKey = draggingKey.value

  // pair forces (O(n^2) fine for the local scene; the Wasm engine in doc 07
  // is where this goes Barnes–Hut for whole-realm canvases)
  for (let i = 0; i < items.length; i++) {
    const a = items[i]
    if (a.key === dragKey) continue
    for (let j = i + 1; j < items.length; j++) {
      const b = items[j]
      if (b.key === dragKey) continue
      let dx = a.x - b.x
      let dy = a.y - b.y
      let d = Math.hypot(dx, dy)
      if (d < 1) {
        dx = (seedN(a.key + b.key) - 0.5) * 2
        dy = (seedN(b.key + a.key) - 0.5) * 2
        d = Math.hypot(dx, dy) || 1
      }
      // charge: inverse-square repulsion (capped to stay stable)
      const fr = Math.min(1.4, REPULSION / (d * d))
      // edge spring: when this pair is linked, tug it back toward REST
      const linked = LINKED.value.has(`${a.key}␟${b.key}`) || LINKED.value.has(`${b.key}␟${a.key}`)
      const fs = linked && d > REST ? (d - REST) * SPRING : 0
      const f = fr - fs
      const ux = dx / d
      const uy = dy / d
      a.vx += ux * f
      a.vy += uy * f
      b.vx -= ux * f
      b.vy -= uy * f
    }
  }

  for (const n of items) {
    if (n.key === dragKey) continue

    // weak pull toward canvas centre keeps the cluster on-screen
    n.vx += (cx - n.x) * GRAVITY
    n.vy += (cy - n.y) * GRAVITY

    const sp = Math.hypot(n.vx, n.vy)
    if (sp > MAX_SPEED) {
      n.vx = (n.vx / sp) * MAX_SPEED
      n.vy = (n.vy / sp) * MAX_SPEED
    }
    n.vx *= DAMPING
    n.vy *= DAMPING
    n.x += n.vx * dt * 60
    n.y += n.vy * dt * 60

    // keep the whole shape (plus guard captions) inside the canvas
    const hw = n.w / 2 + (n.node.kind === 'guard' ? 44 : 14)
    const hh = n.h / 2 + (n.node.kind === 'guard' ? 22 : 10)
    if (n.x < hw) {
      n.x = hw
      n.vx *= -0.3
    } else if (n.x > SCENE_WIDTH - hw) {
      n.x = SCENE_WIDTH - hw
      n.vx *= -0.3
    }
    if (n.y < hh) {
      n.y = hh
      n.vy *= -0.3
    } else if (n.y > SCENE_HEIGHT - hh) {
      n.y = SCENE_HEIGHT - hh
      n.vy *= -0.3
    }
  }
}

/** Scatter the nodes and let the simulator re-settle them. */
function autoLayout() {
  const items = live.value
  const base = Math.min(SCENE_WIDTH, SCENE_HEIGHT) * 0.42
  for (let i = 0; i < items.length; i++) {
    const ang = (i / items.length) * Math.PI * 2 + 0.6
    const rad = base * (0.55 + 0.9 * seedN(items[i].key + 'r'))
    items[i].x = cx + Math.cos(ang) * rad
    items[i].y = cy + Math.sin(ang) * rad
    items[i].vx = (seedN(items[i].key + 'vx') - 0.5) * 3
    items[i].vy = (seedN(items[i].key + 'vy') - 0.5) * 3
  }
  ensureLoop()
}

function togglePlay() {
  playing.value = !playing.value
  if (playing.value) ensureLoop()
}

function ensureLoop() {
  if (rafId || !playing.value || live.value.length < 2) return
  lastStep = performance.now()
  rafId = requestAnimationFrame(step)
}

watch(playing, (v) => {
  if (v) ensureLoop()
  else if (rafId) {
    cancelAnimationFrame(rafId)
    rafId = 0
  }
})

/* ---- node dragging --------------------------------------------------------- */

const draggingKey = ref<string | null>(null)
const dragOffset = ref({ x: 0, y: 0 })
const svgRef = ref<SVGSVGElement | null>(null)
const suppressClick = ref(false)

function toWorld(clientX: number, clientY: number): { x: number; y: number } {
  const svg = svgRef.value
  if (!svg) return { x: cx, y: cy }
  const rect = svg.getBoundingClientRect()
  const s = Math.min(rect.width / SCENE_WIDTH, rect.height / SCENE_HEIGHT)
  const offX = (rect.width - SCENE_WIDTH * s) / 2
  const offY = (rect.height - SCENE_HEIGHT * s) / 2
  const u0 = (clientX - rect.left - offX) / s
  const v0 = (clientY - rect.top - offY) / s
  // invert the zoom group transform around the canvas centre
  const k = scale.value
  return { x: cx + (u0 - cx) / k, y: cy + (v0 - cy) / k }
}

function onPointerDown(e: PointerEvent, item: SceneItem) {
  e.preventDefault()
  suppressClick.value = false
  draggingKey.value = item.key
  const p = toWorld(e.clientX, e.clientY)
  dragOffset.value = { x: p.x - item.x, y: p.y - item.y }
  item.vx = 0
  item.vy = 0
  ensureLoop()
  window.addEventListener('pointermove', onPointerMove)
  window.addEventListener('pointerup', onPointerUp)
}

function onPointerMove(e: PointerEvent) {
  const key = draggingKey.value
  if (!key) return
  const item = live.value.find((n) => n.key === key)
  if (!item) return
  suppressClick.value = true
  const p = toWorld(e.clientX, e.clientY)
  item.x = Math.min(SCENE_WIDTH - 60, Math.max(60, p.x - dragOffset.value.x))
  item.y = Math.min(SCENE_HEIGHT - 60, Math.max(60, p.y - dragOffset.value.y))
  item.vx = 0
  item.vy = 0
}

function onPointerUp() {
  draggingKey.value = null
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
}

function onClickItem(item: SceneItem) {
  if (suppressClick.value) {
    suppressClick.value = false
    return
  }
  emit('select', item.node.id)
}onBeforeUnmount(() => {
  if (rafId) cancelAnimationFrame(rafId)
  rafId = 0
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
})

/* ---- zoom ----------------------------------------------------------------- */

const scale = ref(1)
const viewTransform = computed(
  () => `translate(${cx} ${cy}) scale(${scale.value}) translate(${-cx} ${-cy})`,
)

function zoomBy(f: number) {
  scale.value = Math.min(1.8, Math.max(0.5, scale.value * f))
}

function resetZoom() {
  scale.value = 1
}

/* ---- edges ------------------------------------------------------------------- */

interface DrawnEdge {
  dashed: boolean
  dep: boolean
  x1: number
  y1: number
  x2: number
  y2: number
}

const edges = computed<DrawnEdge[]>(() => {
  const byKey = new Map(live.value.map((s) => [s.key, s]))
  const out: DrawnEdge[] = []
  for (const link of sceneEdges()) {
    const a = byKey.get(link.from)
    const b = byKey.get(link.to)
    if (!a || !b) continue
    const ax = a.x + (a.x < b.x ? a.w / 2 : a.x > b.x ? -a.w / 2 : 0)
    const ay = a.y + (a.y < b.y ? a.h / 2 : a.y > b.y ? -a.h / 2 : 0)
    const bx = b.x + (b.x > a.x ? -b.w / 2 : b.x < a.x ? b.w / 2 : 0)
    const by = b.y + (b.y > a.y ? -b.h / 2 : b.y < a.y ? b.h / 2 : 0)
    out.push({
      dashed: !!link.dashed,
      dep: !!link.dep,
      x1: ax,
      y1: ay,
      x2: bx,
      y2: by,
    })
  }
  return out
})

/** Why the canvas is empty (if it is) - drives the fallback copy. */
const sceneHolds = computed<'mock' | 'realm' | 'empty' | 'large'>(() => {
  if (props.isMockScene) return 'mock'
  if (props.nodesById.size === 0) return 'empty'
  if (props.nodesById.size > MAX_SCENE_NODES) return 'large'
  return 'realm'
})

function trunc(title: string, n: number): string {
  return title.length > n ? `${title.slice(0, n - 1)}…` : title
}

/* ---- breadcrumb + legend ------------------------------------------------------- */

const crumbs = computed<{ id: string | null; label: string }[]>(() => {
  const out: { id: string | null; label: string }[] = [{ id: null, label: props.realmName }]
  let cur = props.selectedId ? props.nodesById.get(props.selectedId) : null
  const chain: NodeView[] = []
  while (cur) {
    chain.unshift(cur)
    cur = cur.parent ? (props.nodesById.get(cur.parent) ?? null) : null
  }
  const shown = chain.length > 4 ? chain.slice(chain.length - 4) : chain
  if (chain.length > 4) out.push({ id: null, label: '…' })
  for (const n of shown) out.push({ id: n.id, label: n.title })
  return out
})

const LEGEND_SHAPES = [
  { key: 'card', label: 'Card' },
  { key: 'action', label: 'Action' },
  { key: 'guard', label: 'Guard' },
  { key: 'idea', label: 'Idea' },
] as const

function statusDotClass(status: string): string {
  return `st-${status}`
}
</script>

<template>
  <section class="graph-pane" aria-label="Graph view">
    <div class="graph-head">
      <nav class="crumbs" aria-label="Breadcrumb">
        <template v-for="(c, i) in crumbs" :key="i">
          <span v-if="i > 0" class="crumb-sep" aria-hidden="true">›</span>
          <button
            v-if="c.id"
            type="button"
            class="crumb crumb-node"
            :title="c.label"
            @click="emit('select', c.id)"
          >
            {{ c.label }}
          </button>
          <span v-else class="crumb crumb-root" :title="c.label">{{ c.label }}</span>
        </template>
      </nav>
      <div class="graph-tools">
        <button
          type="button"
          class="tool-btn"
          :title="playing ? 'Pause auto-layout' : 'Resume auto-layout'"
          :aria-label="playing ? 'Pause auto-layout' : 'Resume auto-layout'"
          @click="togglePlay"
        >
          <PhPause v-if="playing" :size="12" aria-hidden="true" />
          <PhPlay v-else :size="12" aria-hidden="true" />
        </button>
        <button
          type="button"
          class="tool-btn"
          title="Auto-layout: scatter and re-settle"
          aria-label="Auto-layout"
          @click="autoLayout"
        >
          <PhArrowClockwise :size="12" aria-hidden="true" />
        </button>
        <span class="tool-divider" aria-hidden="true"></span>
        <button type="button" class="tool-btn" aria-label="Zoom out" title="Zoom out" @click="zoomBy(1 / 1.25)">
          <PhMinus :size="12" aria-hidden="true" />
        </button>
        <button type="button" class="tool-btn zoom-pct mono" aria-label="Zoom level" title="Reset zoom" @click="resetZoom">
          {{ Math.round(scale * 100) }}%
        </button>
        <button type="button" class="tool-btn" aria-label="Zoom in" title="Zoom in" @click="zoomBy(1.25)">
          <PhPlus :size="12" aria-hidden="true" />
        </button>
        <button type="button" class="tool-btn" aria-label="Fit to screen" title="Fit to screen" @click="resetZoom">
          <PhArrowsOut :size="12" aria-hidden="true" />
        </button>
        <span class="tool-divider" aria-hidden="true"></span>
        <button type="button" class="tool-btn" aria-label="Graph options" title="View options (coming soon)">
          <PhDotsThreeOutlineVertical :size="13" aria-hidden="true" />
        </button>
      </div>
    </div>

    <div class="graph-canvas">
      <div v-if="sceneItems.length" class="scene-wrap">
        <svg
          ref="svgRef"
          class="scene"
          :viewBox="`0 0 ${SCENE_WIDTH} ${SCENE_HEIGHT}`"
          preserveAspectRatio="xMidYMid meet"
          role="img"
          aria-label="Local dependency graph"
          :class="{ dragging: draggingKey }"
        >
          <defs>
            <marker id="arrow-dep" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
              <path d="M0,0 L8,4 L0,8 z" class="mk-dep" />
            </marker>
            <marker id="arrow-opt" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
              <path d="M0,0 L8,4 L0,8 z" class="mk-opt" />
            </marker>
          </defs>

          <g :transform="viewTransform">
            <line
              v-for="(e, i) in edges"
              :key="i"
              :x1="e.x1"
              :y1="e.y1"
              :x2="e.x2"
              :y2="e.y2"
              class="edge"
              :class="[e.dashed ? 'edge-dashed' : e.dep ? 'edge-dep' : 'edge-parent']"
              :marker-end="e.dashed ? 'url(#arrow-opt)' : e.dep ? 'url(#arrow-dep)' : undefined"
            />

            <g
              v-for="item in sceneItems"
              :key="item.node.id"
              class="gnode"
              :class="{ sel: item.node.id === selectedId, held: draggingKey === item.key }"
              @pointerdown="onPointerDown($event, item)"
              @click="onClickItem(item)"
            >
              <title>{{ KIND_LABEL[item.node.kind] }} — {{ item.node.title }} ({{ item.node.effectiveStatus }})</title>

              <g v-if="item.node.kind === 'card'">
                <rect
                  class="shape shape-card"
                  :x="item.x - item.w / 2"
                  :y="item.y - item.h / 2"
                  :width="item.w"
                  :height="item.h"
                  rx="12"
                />
                <circle
                  class="shape-dot"
                  :class="statusDotClass(item.node.effectiveStatus)"
                  :cx="item.x - item.w / 2 + 15"
                  :cy="item.y"
                  r="3.6"
                />
                <text class="node-title card" :x="item.x - item.w / 2 + 27" :y="item.y - 2">
                  {{ trunc(item.node.title, 26) }}
                </text>
                <text class="node-sub mono" :x="item.x - item.w / 2 + 27" :y="item.y + 14">
                  {{ item.node.id }}
                </text>
                <g v-if="item.node.totalQp > 0" class="qp-chip">
                  <rect :x="item.x + item.w / 2 - 54" :y="item.y - item.h / 2 + 7" width="44" height="16" rx="8" />
                  <text class="qp-text mono" :x="item.x + item.w / 2 - 32" :y="item.y - item.h / 2 + 18">
                    {{ item.node.totalQp }} QP
                  </text>
                </g>
              </g>

              <g v-else-if="item.node.kind === 'action' || item.node.kind === 'idea'">
                <rect
                  class="shape"
                  :class="[item.node.kind === 'idea' ? 'shape-idea' : 'shape-action']"
                  :x="item.x - item.w / 2"
                  :y="item.y - item.h / 2"
                  :width="item.w"
                  :height="item.h"
                  rx="15"
                />
                <circle
                  class="shape-dot"
                  :class="statusDotClass(item.node.effectiveStatus)"
                  :cx="item.x - item.w / 2 + 15"
                  :cy="item.y"
                  r="3"
                />
                <text class="node-title" :x="item.x - item.w / 2 + 26" :y="item.y + 3.6">
                  {{ trunc(item.node.title, 25) }}
                </text>
              </g>

              <g v-else>
                <path
                  class="shape shape-guard"
                  :d="`M ${item.x} ${item.y - item.h / 2} L ${item.x + item.w / 2} ${item.y} L ${item.x} ${item.y + item.h / 2} L ${item.x - item.w / 2} ${item.y} Z`"
                />
                <circle
                  class="shape-dot guard-dot"
                  :class="statusDotClass(item.node.effectiveStatus)"
                  :cx="item.x"
                  :cy="item.y - 8"
                  r="2.6"
                />
                <text class="node-guard mono" :x="item.x" :y="item.y + item.h / 2 + 12">
                  {{ trunc(item.node.title, 14).toUpperCase() }}
                </text>
              </g>
            </g>
          </g>
        </svg>

        <div class="minimap" aria-hidden="true">
          <svg :viewBox="`0 0 ${SCENE_WIDTH} ${SCENE_HEIGHT}`" preserveAspectRatio="xMidYMid meet">
            <g class="mm-edges">
              <line
                v-for="(e, i) in edges"
                :key="i"
                :x1="e.x1"
                :y1="e.y1"
                :x2="e.x2"
                :y2="e.y2"
              />
            </g>
            <circle
              v-for="item in sceneItems"
              :key="item.node.id"
              class="mm-node"
              :class="item.node.kind"
              :cx="item.x"
              :cy="item.y"
              r="34"
            />
          </svg>
        </div>

        <div class="legend" aria-label="Legend">
          <div class="legend-shapes">
            <span v-for="s in LEGEND_SHAPES" :key="s.key" class="legend-row">
              <i class="glyph g-shape" :class="`g-${s.key}`" aria-hidden="true"></i>
              <span>{{ s.label }}</span>
            </span>
          </div>
          <div class="legend-edges">
            <span class="legend-row">
              <i class="glyph g-line solid" aria-hidden="true"></i>
              <span>Dependency</span>
            </span>
            <span class="legend-row">
              <i class="glyph g-line dashed" aria-hidden="true"></i>
              <span>Pitch / optional</span>
            </span>
          </div>
        </div>
      </div>

      <div v-else class="graph-fallback">
        <p class="fallback-title">Graph canvas</p>
        <p v-if="sceneHolds === 'large'" class="fallback-body">
          This realm has {{ props.nodesById.size }} nodes &mdash; whole-realm
          physics arrive with the Wasm engine milestone.
        </p>
        <p v-else-if="sceneHolds === 'empty'" class="fallback-body">
          No nodes in this realm yet. Create one from File &rarr; New Realm.
        </p>
        <p v-else class="fallback-body">
          Nothing to draw here yet. The realm&rsquo;s
          {{ props.nodesById.size }} nodes stay browsable in the Table view
          below.
        </p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.graph-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
  background: var(--graph-bg);
  flex: 1;
}

.graph-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 5px 10px;
  border-bottom: 1px solid var(--line-1);
  min-height: 30px;
}

.crumbs {
  display: flex;
  align-items: center;
  gap: 3px;
  min-width: 0;
  font-size: 11.5px;
}

.crumb {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.crumb-root {
  color: var(--faint);
  font-weight: 600;
  letter-spacing: 0.02em;
}

.crumb-node {
  color: var(--text-2);
  padding: 1px 3px;
  border-radius: var(--r-s);
}

.crumb-node:hover {
  color: var(--gold);
  background: var(--bg-2);
}

.crumb-sep {
  color: var(--line-2);
  user-select: none;
}

.graph-tools {
  display: flex;
  align-items: center;
  gap: 1px;
  flex: none;
}

.tool-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 22px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.tool-btn:hover {
  background: var(--bg-2);
  color: var(--text-1);
}

.tool-btn.zoom-pct {
  width: auto;
  padding: 0 6px;
  font-size: 10px;
}

.tool-divider {
  width: 1px;
  height: 14px;
  margin: 0 5px;
  background: var(--line-1);
}

.graph-canvas {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  background-color: var(--graph-canvas);
  background-image: radial-gradient(var(--graph-grid) 1px, transparent 1px);
  background-size: 22px 22px;
  background-position: 11px 11px;
}

.scene-wrap {
  position: absolute;
  inset: 0;
}

.scene {
  width: 100%;
  height: 100%;
}

.scene.dragging {
  cursor: grabbing;
}

.gnode {
  cursor: grab;
}

.gnode.held {
  cursor: grabbing;
}

/* ---- edges & node shapes ------------------------------------------------ */

.edge {
  stroke-width: 1.4;
  vector-effect: non-scaling-stroke;
}

.edge-parent {
  stroke: var(--graph-line);
  opacity: 0.85;
}

.edge-dashed {
  stroke: var(--graph-optional);
  stroke-dasharray: 6 5;
  opacity: 0.95;
}

.edge-dep {
  stroke: var(--kind-action);
  opacity: 0.7;
}

.mk-dep {
  fill: var(--graph-line);
}

.mk-opt {
  fill: var(--graph-optional);
}

.gnode:hover .shape {
  stroke-width: 2;
}

.gnode.sel {
  filter: drop-shadow(0 0 4px var(--gold));
}

.gnode.held .shape {
  stroke-width: 2;
  stroke: var(--gold);
}

.shape {
  stroke-width: 1.5;
}

.shape-card {
  fill: var(--graph-card-fill);
  stroke: var(--kind-card);
}

.shape-action {
  fill: var(--graph-pill-fill);
  stroke: var(--kind-action);
}

.shape-idea {
  fill: var(--graph-idea-fill);
  stroke: var(--kind-idea);
  stroke-dasharray: 7 5;
  stroke-width: 1.6;
}

.shape-guard {
  fill: var(--graph-guard-fill);
  stroke: var(--kind-guard);
  stroke-width: 1.8;
}

.shape-dot {
  opacity: 0.95;
}

.guard-dot {
  stroke: var(--graph-canvas);
  stroke-width: 1.5;
}

.node-title {
  fill: var(--text-1);
  font-size: 12px;
  font-weight: 500;
}

.node-title.card {
  font-weight: 600;
}

.node-sub {
  fill: var(--faint);
  font-size: 9px;
  letter-spacing: 0.03em;
}

.node-guard {
  fill: var(--kind-guard);
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-anchor: middle;
}

.qp-chip rect {
  fill: var(--inset);
  stroke: var(--line-1);
}

.qp-text {
  fill: var(--gold);
  font-size: 9px;
  text-anchor: middle;
}

.st-unstarted {
  fill: var(--dot-unstarted);
}

.st-active {
  fill: var(--dot-active);
}

.st-blocked {
  fill: var(--dot-blocked);
}

.st-vanquished {
  fill: var(--dot-done);
}

/* ---- overlays ------------------------------------------------------------- */

.minimap {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 118px;
  height: 78px;
  padding: 3px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: rgba(8, 12, 18, 0.72);
  backdrop-filter: blur(2px);
}

.minimap svg {
  width: 100%;
  height: 100%;
  display: block;
}

.mm-edges line {
  stroke: var(--line-2);
  stroke-width: 8;
}

.mm-node {
  stroke-width: 0;
}

.mm-node.card {
  fill: var(--kind-card);
}

.mm-node.action {
  fill: var(--kind-action);
}

.mm-node.guard {
  fill: var(--kind-guard);
}

.mm-node.idea {
  fill: var(--kind-idea);
}

.legend {
  position: absolute;
  left: 12px;
  bottom: 12px;
  display: flex;
  align-items: flex-start;
  gap: 18px;
  padding: 8px 12px;
  border-radius: var(--r-m);
  border: 1px solid var(--line-1);
  background: rgba(10, 14, 21, 0.8);
  backdrop-filter: blur(2px);
  font-size: 10px;
  color: var(--text-3);
}

.legend-shapes,
.legend-edges {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.legend-edges {
  border-left: 1px solid var(--line-1);
  padding-left: 16px;
}

.legend-row {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  white-space: nowrap;
}

.glyph {
  flex: none;
}

.g-shape {
  width: 12px;
  height: 12px;
  border-radius: 2px;
}

.g-card {
  background: transparent;
  border: 1.5px solid var(--kind-card);
  border-radius: 3px;
}

.g-action {
  background: var(--graph-pill-fill);
  border: 1.5px solid var(--kind-action);
  border-radius: 6px;
}

.g-guard {
  width: 11px;
  height: 11px;
  background: transparent;
  border: 1.5px solid var(--kind-guard);
  transform: rotate(45deg);
  margin: 0 1.5px;
}

.g-idea {
  background: transparent;
  border: 1.5px dashed var(--kind-idea);
  border-radius: 6px;
}

.g-line {
  position: relative;
  width: 18px;
  height: 0;
}

.g-line::after {
  content: '';
  position: absolute;
  right: 0;
  top: -3px;
  width: 0;
  height: 0;
  border-top: 3px solid transparent;
  border-bottom: 3px solid transparent;
  border-left: 4px solid currentColor;
}

.g-line.solid {
  border-top: 1.5px solid var(--graph-line);
  color: var(--graph-line);
}

.g-line.dashed {
  border-top: 1.5px dashed var(--graph-optional);
  color: var(--graph-optional);
}

/* ---- fallback -------------------------------------------------------------- */

.graph-fallback {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 20px;
  text-align: center;
  gap: 8px;
}

.fallback-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-2);
}

.fallback-body {
  margin: 0;
  max-width: 420px;
  font-size: 12px;
  color: var(--faint);
}

@media (prefers-reduced-motion: reduce) {
  .scene {
    animation: none;
  }
}
</style>
