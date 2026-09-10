<script setup lang="ts">
// Graph view (workspace top half). A lightweight force-directed
// simulator (springs along the edges, charge repulsion between nodes, weak
// centre gravity - the shape the design doc's graph engine will take, see
// doc 07) auto-lays the scene out. Pause it to freeze, drag any node to
// re-arrange the neighbourhood, and hit "auto-layout" to scatter it again.
//
// The physics lives in ./physics.ts, scene seeding/layout in ./scene.ts and
// edge discovery/geometry in ./edges.ts; this file only wires them to the
// SVG, drag handling, zoom and breadcrumb chrome.
//
// Node shapes follow design doc 05: Cards are squares, Actions pills,
// Guards diamonds and Ideas dashed pills. Clicking a node drives the shared
// selection, breadcrumb and inspector.
//
// Three scopes (test-feedback A-1): `local` re-centres on the selection,
// `global` shows every node in the realm (Feature A-a), `overview` shows
// the Realm Origin branching into genre subtrees, terminating at the first
// non-genre node (Feature A-b).

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  PhArrowsOut,
  PhArrowClockwise,
  PhCaretDown,
  PhCrosshair,
  PhList,
  PhMinus,
  PhPause,
  PhPlay,
  PhPlus,
  PhX,
} from '@phosphor-icons/vue'
import { OVERLAY_LABEL, OVERLAY_MODES } from '../../../lib/overlay'
import type { NodeView } from '../../../types'
import { KIND_LABEL, isContainerKind } from '../../../lib/kind'
import { ancestorChain, fullyVanquished, type CompletionLevel } from '../../../lib/node'
import {
  categoryColorMap,
  overlayColor,
  overlayLegend,
  type OverlayMode,
} from '../../../lib/overlay'
import { SCENE_HEIGHT, SCENE_WIDTH } from '../../../lib/mockRealm'
import {
  buildSceneItems,
  MAX_SCENE_NODES,
  scatterAll,
  sceneCenter,
  sceneSignature,
  type GraphScope,
  type SceneItem,
} from './scene'
import { applyForces } from './physics'
import { drawEdges, linkedPairs, sceneEdges } from './edges'
import NodeActionsMenu from './NodeActionsMenu.vue'

const props = defineProps<{
  nodesById: Map<string, NodeView>
  selectedId: string | null
  isMockScene: boolean
  realmName: string
  /** Color overlay mode (doc 05 §5.3), owned by App (menu/chord/palette). */
  overlayMode: OverlayMode
  /** Graph scope: local neighbourhood, whole realm, or genre overview. */
  scope: GraphScope
  /** Completed-task visibility level (doc 05 §5.5 B). */
  completionLevel: CompletionLevel
}>()

const emit = defineEmits<{
  select: [id: string]
  reparent: [childId: string, parentId: string]
  'update:overlayMode': [mode: OverlayMode]
  'update:scope': [scope: GraphScope]
  'update:completionLevel': [level: CompletionLevel]
  /** Open the shared New Node dialog with the selection as parent preset. */
  create: [parentId: string | null]
  /** Open the shared New Node dialog with an explicit parent + kind preset
   * (node actions menu: “New child node…”). */
  'create-kind': [parentId: string | null, kind: string]
  /** Close the graph tab (reopenable from the tab strip / menu). */
  close: []
  /** Double-click navigation: focus this node's local graph. */
  openLocal: [id: string]
}>()

/** Node metadata + live physics state, keyed by scene spot key. */
const live = ref<SceneItem[]>([])

const { x: cx, y: cy } = sceneCenter()
void cx
void cy
/** ViewBox centre — the fixed pivot of the zoom/pan transform (the SVG's
 * `0 0 W H` viewBox always maps fully onto the canvas, letterboxed). */
const VB_CX = SCENE_WIDTH / 2
const VB_CY = SCENE_HEIGHT / 2

/** Scene signature - re-seed only when the member set really changes (realm
 * open/reload, or the mock/local scene swaps). Dragging and auto-layout do
 * not touch it, so a settled layout survives navigation. */
let seededSig = ''
function syncScene() {
  const sig = sceneSignature(props)
  if (sig === seededSig) return
  seededSig = sig
  live.value = buildSceneItems(props)
  if (live.value.length >= 2) ensureLoop()
}

onMounted(syncScene)
watch(() => [props.isMockScene, props.nodesById, props.selectedId, props.scope], syncScene)

const sceneItems = computed<SceneItem[]>(() => live.value)

/** Scene member keys (spot keys on the mock, node ids elsewhere) — restricts
 * edge discovery and the linked-pair spring set to the visible scene. */
const keyMap = computed(() => new Map(live.value.map((i) => [i.key, true] as const)))

/* ---- force simulation ---------------------------------------------------- */

const playing = ref(true)
let rafId = 0
let lastStep = 0

// Linked pairs share a Hooke spring toward rest; everything else just
// repels, so connected clusters stay tight while unrelated nodes part.
const linked = computed(() =>
  linkedPairs(sceneEdges(props.isMockScene, props.nodesById, keyMap.value)),
)

function step(timestamp: number) {
  rafId = 0
  const dt = Math.min(0.05, Math.max(0.005, (timestamp - lastStep) / 1000 || 0.016))
  lastStep = timestamp
  if (playing.value) applyForces(live.value, draggingKey.value, linked.value, dt)
  if (playing.value) rafId = requestAnimationFrame(step)
}

function ensureLoop() {
  if (rafId || !playing.value || live.value.length < 2) return
  lastStep = performance.now()
  rafId = requestAnimationFrame(step)
}

/** Scatter the nodes and let the simulator re-settle them. */
function autoLayout() {
  scatterAll(live.value)
  ensureLoop()
}

function togglePlay() {
  playing.value = !playing.value
  if (playing.value) ensureLoop()
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

/** Screen (client) → world coordinates through the full viewport transform:
 * base fit-scale × zoom around the pan centre. The world is unbounded, so
 * this is valid for any pan/zoom. */
function toWorld(clientX: number, clientY: number): { x: number; y: number } {
  const svg = svgRef.value
  if (!svg) return { x: panX.value, y: panY.value }
  const rect = svg.getBoundingClientRect()
  const s = Math.min(rect.width / SCENE_WIDTH, rect.height / SCENE_HEIGHT)
  const offX = (rect.width - SCENE_WIDTH * s) / 2
  const offY = (rect.height - SCENE_HEIGHT * s) / 2
  const u0 = (clientX - rect.left - offX) / s
  const v0 = (clientY - rect.top - offY) / s
  const k = scale.value
  return { x: panX.value + (u0 - VB_CX) / k, y: panY.value + (v0 - VB_CY) / k }
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

/* ---- single vs double click (test-feedback Bug9) -------------------------
 * The native `dblclick` event is unreliable here: `onPointerDown` calls
 * `preventDefault()`, which suppresses the compatibility mouse events the
 * browser uses to reset its click-`detail` counter — after the first
 * double-click, later clicks get miscounted. So we detect the second click
 * ourselves with an explicit per-node timer; a double-click is registered
 * exactly on the *second* click, and every single click stays a selection. */

const DBL_CLICK_MS = 450
let lastClickKey: string | null = null
let lastClickAt = 0

function onClickItem(item: SceneItem) {
  if (suppressClick.value) {
    suppressClick.value = false
    return
  }
  const now = performance.now()
  if (lastClickKey === item.key && now - lastClickAt < DBL_CLICK_MS) {
    lastClickKey = null
    lastClickAt = 0
    onDblClickItem(item)
    return
  }
  lastClickKey = item.key
  lastClickAt = now
  emit('select', item.node.id)
}

/** Node currently under the dragged item (drop target highlight). */
const dropTargetKey = ref<string | null>(null)

/** Mutations are desktop-only; the mock scene cannot reparent. The App
 * layer passes a listener for our `reparent` emit — detect it without a
 * dedicated prop by checking the component's own listeners is not possible
 * in script setup, so reparent is always offered and App no-ops when
 * mutation is unavailable. */
const canReparent = computed(() => !props.isMockScene)

function findDropTarget(clientX: number, clientY: number): SceneItem | null {
  const p = toWorld(clientX, clientY)
  let best: SceneItem | null = null
  let bestD = Infinity
  for (const it of live.value) {
    if (it.key === draggingKey.value) continue
    const d = Math.hypot(it.x - p.x, it.y - p.y)
    if (d < Math.max(it.w, it.h) * 0.75 && d < bestD) {
      best = it
      bestD = d
    }
  }
  return best
}

function onPointerMove(e: PointerEvent) {
  const key = draggingKey.value
  if (!key) return
  const item = live.value.find((n) => n.key === key)
  if (!item) return
  suppressClick.value = true
  const p = toWorld(e.clientX, e.clientY)
  // unbounded plane: dragged nodes go wherever the user drops them
  item.x = p.x - dragOffset.value.x
  item.y = p.y - dragOffset.value.y
  item.vx = 0
  item.vy = 0
  // live drop-target detection while dragging
  const target = canReparent.value ? findDropTarget(e.clientX, e.clientY) : null
  dropTargetKey.value = target && isContainerKind(target.node.kind) ? target.key : null
}

function onPointerUp(e: PointerEvent) {
  const key = draggingKey.value
  const targetKey = dropTargetKey.value
  dropTargetKey.value = null
  draggingKey.value = null
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
  if (key && targetKey && canReparent.value) {
    const child = live.value.find((n) => n.key === key)
    const parent = live.value.find((n) => n.key === targetKey)
    if (child && parent && child.node.parent !== parent.node.id) {
      emit('reparent', child.node.id, parent.node.id)
    }
  }
  void e
}

onBeforeUnmount(() => {
  if (rafId) cancelAnimationFrame(rafId)
  rafId = 0
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
})

/* ---- viewport: pan + zoom over the unbounded world (doc 05 §5.5) ---------- */

/** Zoom scale, 10% → 400%. */
const scale = ref(1)
/** World point displayed at the viewport centre (the pan offset). */
const panX = ref(0)
const panY = ref(0)

const viewTransform = computed(
  () => `translate(${VB_CX} ${VB_CY}) scale(${scale.value}) translate(${-panX.value} ${-panY.value})`,
)

/** Zoom by `f`, keeping the world point under `clientX/Y` (or the viewport
 * centre when omitted) fixed on screen. */
function zoomAt(f: number, clientX?: number, clientY?: number) {
  const svg = svgRef.value
  const next = Math.min(4, Math.max(0.1, scale.value * f))
  if (next === scale.value) return
  if (svg && clientX !== undefined && clientY !== undefined) {
    const before = toWorld(clientX, clientY)
    scale.value = next
    const after = toWorld(clientX, clientY)
    panX.value += before.x - after.x
    panY.value += before.y - after.y
  } else {
    scale.value = next
  }
}

function zoomBy(f: number) {
  zoomAt(f)
}

function resetZoom() {
  scale.value = 1
  panX.value = 0
  panY.value = 0
}

/** Fit the viewport to the bounding box of all scene items (with padding). */
function fitToNodes() {
  if (!live.value.length) {
    resetZoom()
    return
  }
  let minX = Infinity
  let minY = Infinity
  let maxX = -Infinity
  let maxY = -Infinity
  for (const it of live.value) {
    minX = Math.min(minX, it.x - it.w / 2)
    minY = Math.min(minY, it.y - it.h / 2)
    maxX = Math.max(maxX, it.x + it.w / 2)
    maxY = Math.max(maxY, it.y + it.h / 2)
  }
  const pad = 80
  const w = Math.max(1, maxX - minX + pad * 2)
  const h = Math.max(1, maxY - minY + pad * 2)
  // The whole viewBox (0..W, 0..H) is always visible, and the transform
  // maps the pan target onto the viewBox centre — so the bbox half-width
  // at scale k must fit within W/2 and H/2.
  scale.value = Math.min(4, Math.max(0.1, Math.min(SCENE_WIDTH / w, SCENE_HEIGHT / h)))
  panX.value = (minX + maxX) / 2
  panY.value = (minY + maxY) / 2
}

/* ---- canvas panning (drag empty space) ------------------------------------- */

const panning = ref(false)
let panStart = { x: 0, y: 0, px: 0, py: 0 }

function onCanvasPointerDown(e: PointerEvent) {
  if (e.button !== 0) return
  panning.value = true
  panStart = { x: e.clientX, y: e.clientY, px: panX.value, py: panY.value }
  window.addEventListener('pointermove', onCanvasPointerMove)
  window.addEventListener('pointerup', onCanvasPointerUp)
}

function onCanvasPointerMove(e: PointerEvent) {
  if (!panning.value) return
  const svg = svgRef.value
  if (!svg) return
  const rect = svg.getBoundingClientRect()
  const s = Math.min(rect.width / SCENE_WIDTH, rect.height / SCENE_HEIGHT)
  panX.value = panStart.px - (e.clientX - panStart.x) / (s * scale.value)
  panY.value = panStart.py - (e.clientY - panStart.y) / (s * scale.value)
}

function onCanvasPointerUp() {
  panning.value = false
  window.removeEventListener('pointermove', onCanvasPointerMove)
  window.removeEventListener('pointerup', onCanvasPointerUp)
}

/** Wheel zooms toward the cursor; ctrl+wheel (pinch) also lands here. */
function onWheel(e: WheelEvent) {
  e.preventDefault()
  const f = e.deltaY < 0 ? 1.12 : 1 / 1.12
  zoomAt(f, e.clientX, e.clientY)
}

/* ---- double-click navigation (global / overview → local) ------------------- */

function onDblClickItem(item: SceneItem) {
  if (isOrigin(item)) return
  emit('openLocal', item.node.id)
}

/* ---- edges ------------------------------------------------------------------- */

const edges = computed(() =>
  drawEdges(live.value, sceneEdges(props.isMockScene, props.nodesById, keyMap.value)),
)

/** Why the canvas is empty (if it is) - drives the fallback copy. */
const sceneHolds = computed<'mock' | 'realm' | 'empty' | 'large' | 'nogenre'>(() => {
  if (props.isMockScene) return 'mock'
  if (props.nodesById.size === 0) return 'empty'
  if (props.nodesById.size > MAX_SCENE_NODES) return 'large'
  if (props.scope === 'overview' && ![...props.nodesById.values()].some((n) => n.kind === 'genre')) {
    return 'nogenre'
  }
  return 'realm'
})

function trunc(title: string, n: number): string {
  return title.length > n ? `${title.slice(0, n - 1)}…` : title
}

/* ---- breadcrumb + legend ------------------------------------------------------- */

const crumbs = computed<{ id: string | null; label: string }[]>(() => {
  const out: { id: string | null; label: string }[] = [{ id: null, label: props.realmName }]
  const chain = ancestorChain(props.nodesById, props.selectedId)
  if (!chain.length) return out
  const shown = chain.length > 4 ? chain.slice(chain.length - 4) : chain
  if (chain.length > 4) out.push({ id: null, label: '…' })
  for (const n of shown) out.push({ id: n.id, label: n.title })
  return out
})

const LEGEND_SHAPES = [
  { key: 'card', label: 'Card' },
  { key: 'genre', label: 'Genre' },
  { key: 'action', label: 'Action' },
  { key: 'guard', label: 'Guard' },
  { key: 'idea', label: 'Idea' },
] as const

/* ---- legend visibility (test-feedback Bug6): closeable, reopenable ------- */

/** Component-local so a refresh restores it; the legend stays per-session
 * state rather than a realm setting. */
const legendOpen = ref(true)

function toggleLegend() {
  legendOpen.value = !legendOpen.value
}

/* ---- node actions menu (test-feedback Item A) -----------------------------
 * One floating menu (NodeActionsMenu.vue), two triggers: the toolbar +
 * button (acts on the selection) and right-clicking any node (acts on the
 * clicked node). The menu offers the shared New Node dialog with a kind
 * preset plus double-click navigation. */

interface ContextMenuState {
  x: number
  y: number
  nodeId: string
}

const contextMenu = ref<ContextMenuState | null>(null)

/** The node the menu currently acts on (selection for the toolbar trigger). */
const menuNode = computed<NodeView | null>(() =>
  contextMenu.value ? (props.nodesById.get(contextMenu.value.nodeId) ?? null) : null,
)

/** Right-click on a node: anchor the actions menu at the cursor. */
function onNodeContextMenu(e: MouseEvent, item: SceneItem) {
  if (props.isMockScene || isOrigin(item)) return
  e.preventDefault()
  emit('select', item.node.id)
  contextMenu.value = { x: e.clientX, y: e.clientY, nodeId: item.node.id }
}

function closeContextMenu() {
  contextMenu.value = null
}

/** Toolbar “new node” button: open the menu anchored to the button itself
 * (the selection is the actee). Falls back to the old one-click create when
 * nothing is selected. */
function onNewNodeButton(e: MouseEvent) {
  if (!props.selectedId) {
    emit('create', null)
    return
  }
  const btn = (e.currentTarget as HTMLElement | null)?.getBoundingClientRect()
  contextMenu.value = {
    x: btn ? btn.left : e.clientX,
    y: btn ? btn.bottom + 2 : e.clientY,
    nodeId: props.selectedId,
  }
}

function menuCreateChild(kind: string) {
  if (!contextMenu.value) return
  emit('create-kind', contextMenu.value.nodeId, kind)
  closeContextMenu()
}

/** Sibling creation: same parent as the actee, chosen kind. */
function menuCreateSiblingKind(kind: string) {
  if (!contextMenu.value) return
  const n = props.nodesById.get(contextMenu.value.nodeId)
  emit('create-kind', n?.parent ?? null, kind)
  closeContextMenu()
}

function menuOpenLocal() {
  if (!contextMenu.value) return
  emit('openLocal', contextMenu.value.nodeId)
  closeContextMenu()
}

/* ---- overlays (doc 05 §5.3) -------------------------------------------- */

/** Categorical color maps, built from the realm's declared branch paths. */
const epicColors = computed(() => {
  const paths = new Set<string>()
  for (const n of props.nodesById.values()) for (const p of n.epics) paths.add(p)
  return categoryColorMap([...paths])
})
const discColors = computed(() => {
  const paths = new Set<string>()
  for (const n of props.nodesById.values()) for (const p of n.disciplines) paths.add(p)
  return categoryColorMap([...paths])
})

/** Stroke color override for a scene item under the active overlay. */
function strokeFor(item: SceneItem): string | null {
  if (props.isMockScene) return null
  return overlayColor(props.overlayMode, item.node, epicColors.value, discColors.value)
}

/** Style binding for shape strokes (typed for Vue's StyleValue). */
function strokeStyleFor(item: SceneItem): { stroke: string } | undefined {
  const c = strokeFor(item)
  return c ? { stroke: c } : undefined
}

const legendEntries = computed(() =>
  props.isMockScene ? [] : overlayLegend(props.overlayMode, epicColors.value, discColors.value),
)

/* ---- depth scale (doc 05 §5.3) ----------------------------------------- */

/** Depth in the parent tree (roots = 0). Drives visual scale/emphasis. */
function depthOf(n: NodeView): number {
  let d = 0
  let cur: NodeView | undefined = n
  const seen = new Set<string>()
  while (cur?.parent && !seen.has(cur.id)) {
    seen.add(cur.id)
    cur = props.nodesById.get(cur.parent)
    d += 1
    if (d > 24) break
  }
  return d
}

const depthScale = computed(() => {
  const m = new Map<string, number>()
  if (props.isMockScene) return m
  for (const n of props.nodesById.values()) {
    const d = depthOf(n)
    m.set(n.id, Math.max(0.8, 1.15 - d * 0.12))
  }
  return m
})

/** Render scale for an item (multiplies its base size). */
function scaleFor(item: SceneItem): number {
  return depthScale.value.get(item.node.id) ?? 1
}

/* ---- progress ring + locked-QP indicator (doc 05 §5.2 Cards) ------------ */

/** Fraction of descendant leaf QP that is vanquished (0..1), or null. */
function cardProgress(n: NodeView): number | null {
  if (!isContainerKind(n.kind) || n.totalQp <= 0) return null
  const byId = props.nodesById
  const stack = [...(n.parent === null ? [] : [])]
  void stack
  let done = 0
  const ids = [n.id]
  while (ids.length) {
    const cur = ids.pop()
    for (const c of byId.values()) {
      if (c.parent === cur) {
        if (!isContainerKind(c.kind)) {
          if (c.effectiveStatus === 'vanquished') done += c.questPoints ?? 0
        } else {
          ids.push(c.id)
        }
      }
    }
  }
  return Math.min(1, done / n.totalQp)
}

/** SVG arc path for the progress ring (r = 7, centred top-right). */
function ringArc(cx0: number, cy0: number, frac: number): string {
  const r = 7
  const a = frac * Math.PI * 2 - Math.PI / 2
  const x = cx0 + r * Math.cos(a)
  const y = cy0 + r * Math.sin(a)
  const large = frac > 0.5 ? 1 : 0
  return `M ${cx0} ${cy0 - r} A ${r} ${r} 0 ${large} 1 ${x.toFixed(2)} ${y.toFixed(2)}`
}

/* ---- Realm Origin (doc 05 §5.5 / Feature A-b overview) -------------------- */

const ORIGIN_KEY = '__origin__'

/** Overview scope injects a synthetic [ Realm Origin ] item; every root
 * node gets an origin edge. */
const originItem = computed<SceneItem | null>(() => {
  if (props.scope !== 'overview' || !sceneItems.value.length) return null
  return {
    key: ORIGIN_KEY,
    node: {
      id: 'ORIGIN',
      kind: 'card',
      title: 'Realm Origin',
      status: 'unstarted',
      effectiveStatus: 'unstarted',
      blocked: false,
      priority: 'low',
      disciplines: [],
      epics: [],
      questPoints: null,
      landmark: null,
      explicitLandmark: null,
      parent: null,
      blockedBy: [],
      totalQp: 0,
      lockedQpPercent: null,
      completedAt: null,
      createdAt: null,
      tags: [],
      path: '',
      validationError: null,
      body: '',
    } as NodeView,
    x: 0,
    y: -260,
    w: 150,
    h: 40,
    vx: 0,
    vy: 0,
  }
})

/** Scene items + the synthetic origin, in render order. */
const renderItems = computed<SceneItem[]>(() =>
  originItem.value ? [originItem.value, ...sceneItems.value] : sceneItems.value,
)

/** Completed-task visibility over the scope's members. */
const visibleItems = computed<SceneItem[]>(() => {
  if (props.isMockScene || props.completionLevel === 2) return renderItems.value
  const byId = props.nodesById
  const keep = new Set<string>()
  for (const n of byId.values()) {
    if (n.effectiveStatus === 'vanquished') {
      if (props.completionLevel === 0) continue
      // level 1: keep if inside a kept (partial) top-level card
      let cur: NodeView | undefined = n
      const seen = new Set<string>()
      let kept = false
      while (cur?.parent && !seen.has(cur.id)) {
        seen.add(cur.id)
        const p = byId.get(cur.parent)
        if (!p) break
        if (p.parent === null && isContainerKind(p.kind) && !fullyVanquished(byId, p)) {
          kept = true
          break
        }
        cur = p
      }
      if (!kept) continue
    }
    keep.add(n.id)
  }
  const out = renderItems.value.filter((it) => it.key === ORIGIN_KEY || keep.has(it.key))
  return out
})

/** Origin edges: origin → every top-level member (overview scope only). */
const originEdges = computed(() => {
  if (!originItem.value) return []
  const o = originItem.value
  const out: { x1: number; y1: number; x2: number; y2: number }[] = []
  for (const n of props.nodesById.values()) {
    if (n.parent === null) {
      const it = visibleItems.value.find((i) => i.key === n.id)
      if (it) out.push({ x1: o.x, y1: o.y + o.h / 2, x2: it.x, y2: it.y - it.h / 2 })
    }
  }
  return out
})

function isOrigin(item: SceneItem): boolean {
  return item.key === ORIGIN_KEY
}
</script>

<template>
  <section class="graph-pane" aria-label="Graph view">
    <div class="graph-head">
      <span class="pane-tab" title="Graph view">
        Graph View
        <button
          type="button"
          class="tab-close"
          aria-label="Close graph view"
          @click="emit('close')"
        >
          <PhX :size="10" aria-hidden="true" />
        </button>
      </span>
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
        <select
          class="overlay-select scope-select"
          :value="scope"
          aria-label="Graph scope"
          title="Graph scope: local neighbourhood, whole realm, or genre overview"
          @change="$emit('update:scope', ($event.target as HTMLSelectElement).value as GraphScope)"
        >
          <option value="local">Local graph</option>
          <option value="global">Global graph (whole realm)</option>
          <option value="overview">Genre overview</option>
        </select>
        <span class="tool-divider" aria-hidden="true"></span>
        <!-- Item A: the + opens the node actions menu (create child/sibling
             with a kind dropdown) when a node is selected; bare create when
             nothing is selected. -->
        <span v-if="!isMockScene" class="rel">
          <button
            type="button"
            class="tool-btn"
            :class="{ 'tool-on': contextMenu !== null }"
            :title="selectedId ? 'Node actions: create a child or sibling node' : 'New node (no parent)'"
            aria-label="New node options"
            aria-haspopup="menu"
            @pointerdown.stop
            @click="onNewNodeButton($event)"
          >
            <PhPlus :size="12" aria-hidden="true" />
            <PhCaretDown v-if="selectedId" :size="8" class="plus-caret" aria-hidden="true" />
          </button>
        </span>
        <span v-if="!isMockScene" class="tool-divider" aria-hidden="true"></span>
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
        <button type="button" class="tool-btn" aria-label="Zoom to fit" title="Zoom to fit (shows every node)" @click="fitToNodes">
          <PhArrowsOut :size="12" aria-hidden="true" />
        </button>
        <button type="button" class="tool-btn" aria-label="Reset view" title="Reset view (recentre on world origin, 100%)" @click="resetZoom">
          <PhCrosshair :size="12" aria-hidden="true" />
        </button>
        <span class="tool-divider" aria-hidden="true"></span>
        <button
          type="button"
          class="tool-btn"
          :class="{ 'tool-on': legendOpen }"
          :title="legendOpen ? 'Hide legend' : 'Show legend'"
          :aria-label="legendOpen ? 'Hide legend' : 'Show legend'"
          :aria-pressed="legendOpen"
          @click="toggleLegend"
        >
          <PhList :size="12" aria-hidden="true" />
        </button>
        <select
          class="overlay-select"
          :value="overlayMode"
          aria-label="Color overlay mode"
          title="Color overlay mode (doc 05 §5.3)"
          @change="$emit('update:overlayMode', ($event.target as HTMLSelectElement).value as OverlayMode)"
        >
          <option v-for="m in OVERLAY_MODES" :key="m" :value="m">{{ OVERLAY_LABEL[m] }}</option>
        </select>
        <template v-if="!isMockScene">
          <select
            class="overlay-select"
            :value="completionLevel"
            aria-label="Completed task visibility"
            title="Completed-task visibility (doc 05 §5.5 B)"
            @change="$emit('update:completionLevel', Number(($event.target as HTMLSelectElement).value) as CompletionLevel)"
          >
            <option :value="2">Show all completed</option>
            <option :value="1">Hide completed subgraphs</option>
            <option :value="0">Hide all completed</option>
          </select>
        </template>
      </div>
    </div>      <div class="graph-canvas">
      <div v-if="sceneItems.length" class="scene-wrap">
        <svg
          ref="svgRef"
          class="scene"
          :viewBox="`0 0 ${SCENE_WIDTH} ${SCENE_HEIGHT}`"
          preserveAspectRatio="xMidYMid meet"
          role="img"
          aria-label="Dependency graph canvas — drag empty space to pan, scroll to zoom, double-click a node to open its local graph"
          :class="{ dragging: draggingKey, panning }"
          @pointerdown.self.prevent="onCanvasPointerDown"
          @wheel="onWheel"
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
              v-for="(e, i) in originEdges"
              :key="`o${i}`"
              :x1="e.x1"
              :y1="e.y1"
              :x2="e.x2"
              :y2="e.y2"
              class="edge edge-origin"
            />

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
              v-for="item in visibleItems"
              :key="item.node.id"
              class="gnode"
              :class="{ sel: item.node.id === selectedId, held: draggingKey === item.key, droptarget: dropTargetKey === item.key, origin: isOrigin(item) }"
              :transform="`scale(${isOrigin(item) ? 1 : scaleFor(item)})`"
              :style="isOrigin(item) ? undefined : { transformOrigin: `${item.x}px ${item.y}px`, transformBox: 'fill-box' }"
              @pointerdown="onPointerDown($event, item)"
              @contextmenu.prevent="onNodeContextMenu($event, item)"
              @click="onClickItem(item)"
            >
              <title>{{ KIND_LABEL[item.node.kind] }} — {{ item.node.title }} ({{ item.node.effectiveStatus }})</title>

              <g v-if="isOrigin(item)">
                <rect
                  class="shape shape-origin"
                  :x="item.x - item.w / 2"
                  :y="item.y - item.h / 2"
                  :width="item.w"
                  :height="item.h"
                  rx="10"
                />
                <text class="node-title card origin-title" :x="item.x" :y="item.y + 4">
                  [ Realm Origin ]
                </text>
              </g>

              <g v-else-if="isContainerKind(item.node.kind)">
                <rect
                  class="shape"
                  :class="item.node.kind === 'genre' ? 'shape-genre' : 'shape-card'"
                  :x="item.x - item.w / 2"
                  :y="item.y - item.h / 2"
                  :width="item.w"
                  :height="item.h"
                  rx="12"
                  :style="strokeStyleFor(item)"
                />
                <text class="node-title card" :x="item.x - item.w / 2 + 16" :y="item.y - 2">
                  {{ trunc(item.node.title, 26) }}
                </text>
                <text class="node-sub mono" :x="item.x - item.w / 2 + 16" :y="item.y + 14">
                  {{ item.node.id }}
                </text>
                <g v-if="item.node.totalQp > 0" class="qp-chip">
                  <rect :x="item.x + item.w / 2 - 54" :y="item.y - item.h / 2 + 7" width="44" height="16" rx="8" />
                  <text class="qp-text mono" :x="item.x + item.w / 2 - 32" :y="item.y - item.h / 2 + 18">
                    {{ item.node.totalQp }} QP
                  </text>
                </g>
                <g
                  v-if="cardProgress(item.node) !== null"
                  class="progress-ring"
                  :transform="`translate(${item.x + item.w / 2 - 66} ${item.y - item.h / 2 + 15})`"
                >
                  <circle class="ring-track" r="7" />
                  <path class="ring-arc" :d="ringArc(0, 0, cardProgress(item.node) ?? 0)" />
                </g>
                <g
                  v-if="item.node.lockedQpPercent !== null && item.node.lockedQpPercent > 0"
                  class="locked-ind"
                  :transform="`translate(${item.x - item.w / 2 + 62} ${item.y - item.h / 2 + 15})`"
                >
                  <circle class="locked-dot" r="4" />
                  <text class="locked-pct mono" y="3.4">{{ Math.round(item.node.lockedQpPercent) }}</text>
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
                  :style="strokeStyleFor(item)"
                />
                <text class="node-title" :x="item.x - item.w / 2 + 15" :y="item.y + 3.6">
                  {{ trunc(item.node.title, 25) }}
                </text>
              </g>

              <g v-else>
                <path
                  class="shape shape-guard"
                  :d="`M ${item.x} ${item.y - item.h / 2} L ${item.x + item.w / 2} ${item.y} L ${item.x} ${item.y + item.h / 2} L ${item.x - item.w / 2} ${item.y} Z`"
                  :style="strokeStyleFor(item)"
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

        <!-- Node actions menu (Item A): floats above the canvas at any
             zoom/pan; kind dropdown opens the shared New Node dialog. -->
        <NodeActionsMenu
          :node="menuNode"
          :x="contextMenu?.x ?? 0"
          :y="contextMenu?.y ?? 0"
          @create-child="menuCreateChild"
          @create-sibling-kind="menuCreateSiblingKind"
          @open-local="menuOpenLocal"
          @close="closeContextMenu"
        />

        <div v-if="legendOpen" class="legend" aria-label="Legend">
          <button
            type="button"
            class="legend-close"
            title="Hide legend"
            aria-label="Hide legend"
            @click="legendOpen = false"
          >
            <PhX :size="10" aria-hidden="true" />
          </button>
          <div class="legend-shapes">
            <span v-for="s in LEGEND_SHAPES" :key="s.key" class="legend-row">
              <i class="glyph g-shape" :class="`g-${s.key}`" aria-hidden="true"></i>
              <span>{{ s.label }}</span>
            </span>
          </div>
          <div v-if="legendEntries.length" class="legend-edges">
            <span v-for="e in legendEntries.slice(0, 8)" :key="e.label" class="legend-row">
              <i class="glyph g-swatch" :style="{ background: e.color }" aria-hidden="true"></i>
              <span>{{ e.label }}</span>
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
        <p v-else-if="sceneHolds === 'nogenre'" class="fallback-body">
          No genre nodes yet &mdash; create a Genre (a broad categorization
          like &ldquo;Combat Engine&rdquo;) and parent root Cards beneath it.
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
  gap: 10px;
  padding: 5px 10px;
  border-bottom: 1px solid var(--line-1);
  min-height: 30px;
}

/* ---- pane tab (matches the Table pane's tab strip) ---------------------- */

.pane-tab {
  flex: none;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 4px 3px 8px;
  border-radius: var(--r-m) 0 0 var(--r-m);
  border: 1px solid var(--line-2);
  border-right: 0;
  background: var(--bg-2);
  color: var(--text-1);
  font-size: 11.5px;
  font-weight: 600;
  white-space: nowrap;
}

.tab-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.tab-close:hover {
  color: var(--text-1);
  background: var(--bg-3);
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

.scene.panning {
  cursor: grabbing;
}

.scene {
  cursor: grab;
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

.gnode.droptarget .shape {
  stroke-width: 2.4;
  stroke: var(--gold);
  stroke-dasharray: 5 4;
}

.gnode.droptarget {
  filter: drop-shadow(0 0 6px var(--gold));
}

.shape {
  stroke-width: 1.5;
}

.shape-card {
  fill: var(--graph-card-fill);
  stroke: var(--kind-card);
}

.shape-genre {
  fill: var(--graph-card-fill);
  stroke: var(--kind-genre);
  stroke-width: 1.8;
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

.mm-node.genre {
  fill: var(--kind-genre);
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

.legend-close {
  position: absolute;
  top: 3px;
  right: 3px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: var(--r-s);
  color: var(--text-3);
}

.legend-close:hover {
  color: var(--text-1);
  background: var(--bg-3);
}

.tool-btn.tool-on {
  color: var(--gold);
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

.g-genre {
  background: transparent;
  border: 1.8px solid var(--kind-genre);
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

.g-swatch {
  width: 10px;
  height: 10px;
  border-radius: 3px;
  border: 1px solid var(--line-1);
}

/* ---- overlays, origin, rings ---------------------------------------------- */

.overlay-select {
  height: 22px;
  max-width: 150px;
  border-radius: var(--r-s);
  border: 1px solid var(--line-1);
  background: var(--bg-2);
  color: var(--text-2);
  font-size: 10.5px;
  outline: none;
}

.overlay-select.scope-select {
  max-width: 190px;
}

.overlay-select:focus {
  border-color: var(--gold);
}

.edge-origin {
  stroke: var(--graph-line);
  stroke-dasharray: 2 4;
  opacity: 0.6;
}

.shape-origin {
  fill: var(--graph-card-fill);
  stroke: var(--faint);
  stroke-dasharray: 3 3;
}

.origin-title {
  text-anchor: middle;
  fill: var(--text-2);
  font-size: 12px;
  font-weight: 600;
}

.gnode.origin {
  cursor: default;
}

.ring-track {
  fill: none;
  stroke: var(--line-2);
  stroke-width: 2;
}

.ring-arc {
  fill: none;
  stroke: var(--dot-done);
  stroke-width: 2;
  stroke-linecap: round;
}

.locked-dot {
  fill: none;
  stroke: var(--chip-blocked-fg);
  stroke-width: 1.4;
}

.locked-pct {
  font-size: 6.5px;
  text-anchor: middle;
  fill: var(--chip-blocked-fg);
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

/* ---- node actions menu trigger (Item A) ------------------------------------ */

.rel {
  position: relative;
}

.plus-caret {
  margin-left: -3px;
  opacity: 0.7;
}

@media (prefers-reduced-motion: reduce) {
  .scene {
    animation: none;
  }
}
</style>
