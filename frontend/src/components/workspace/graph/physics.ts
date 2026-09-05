// The lightweight force-directed simulator (springs along the edges, charge
// repulsion between nodes, weak centre gravity — the shape the design doc's
// graph engine will take, see doc 07). Auto-lays the scene out when playing;
// pausing freezes it, dragging overrides an item's physics.
//
// O(n²) pair forces are fine for the local scene; the Wasm engine is where
// this goes Barnes–Hut for whole-realm canvases.

import type { SceneItem } from './scene'
import { sceneCenter, seedN } from './scene'
import { SCENE_HEIGHT, SCENE_WIDTH } from '../../../lib/mockRealm'

// Spring / repulsion tuning for a calm, readable settle.
export const FORCES = {
  rest: 165,
  spring: 0.006,
  repulsion: 26000,
  gravity: 0.0009,
  damping: 0.88,
  maxSpeed: 5,
} as const

/** Advance the whole scene by one fixed-step dt: pair forces, centre
 * gravity, speed cap + damping, then integrate positions and clamp every
 * item (plus guard captions) inside the canvas. Mutates `items`. */
export function applyForces(
  items: SceneItem[],
  dragKey: string | null,
  linked: Set<string>,
  dt: number,
): void {
  if (items.length < 2) return
  const { x: cx, y: cy } = sceneCenter()

  // pair forces
  for (let i = 0; i < items.length; i++) {
    const a = items[i]!
    if (a.key === dragKey) continue
    for (let j = i + 1; j < items.length; j++) {
      const b = items[j]!
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
      const fr = Math.min(1.4, FORCES.repulsion / (d * d))
      // edge spring: when this pair is linked, tug it back toward rest
      const isLinked =
        linked.has(`${a.key}␟${b.key}`) || linked.has(`${b.key}␟${a.key}`)
      const fs = isLinked && d > FORCES.rest ? (d - FORCES.rest) * FORCES.spring : 0
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
    n.vx += (cx - n.x) * FORCES.gravity
    n.vy += (cy - n.y) * FORCES.gravity

    const sp = Math.hypot(n.vx, n.vy)
    if (sp > FORCES.maxSpeed) {
      n.vx = (n.vx / sp) * FORCES.maxSpeed
      n.vy = (n.vy / sp) * FORCES.maxSpeed
    }
    n.vx *= FORCES.damping
    n.vy *= FORCES.damping
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
