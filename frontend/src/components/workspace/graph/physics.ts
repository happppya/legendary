// The lightweight force-directed simulator (springs along the edges, charge
// repulsion between nodes, weak centre gravity — the shape the design doc's
// graph engine will take, see doc 07). Auto-lays the scene out when playing;
// pausing freezes it, dragging overrides an item's physics.
//
// The world is UNBOUNDED: nodes may settle anywhere on the infinite plane
// (users pan/zoom the viewport to move around). Only a very soft leash
// toward the origin keeps the cluster from drifting away over time.
//
// O(n²) pair forces are fine for the local scene; the Wasm engine is where
// this goes Barnes–Hut for whole-realm canvases.

import type { SceneItem } from './scene'
import { seedN } from './scene'

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

    // very soft pull toward the world origin — a leash, not a wall, so the
    // layout stays centred-ish without clamping anything to a rectangle
    n.vx += (0 - n.x) * FORCES.gravity
    n.vy += (0 - n.y) * FORCES.gravity

    const sp = Math.hypot(n.vx, n.vy)
    if (sp > FORCES.maxSpeed) {
      n.vx = (n.vx / sp) * FORCES.maxSpeed
      n.vy = (n.vy / sp) * FORCES.maxSpeed
    }
    n.vx *= FORCES.damping
    n.vy *= FORCES.damping
    n.x += n.vx * dt * 60
    n.y += n.vy * dt * 60
  }
}
