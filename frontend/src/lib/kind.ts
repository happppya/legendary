// Shared kind vocabulary for the node index and detail sheet. Single source
// of truth for the icon + label + ordering used everywhere in the renderer.

import type { Component } from 'vue'
import {
  PhLightbulb,
  PhLightning,
  PhShieldCheck,
  PhSquaresFour,
} from '@phosphor-icons/vue'
import type { NodeView } from '../types'

export type Kind = NodeView['kind']

/** Kind keys in canonical display order. */
export const KINDS: readonly Kind[] = ['card', 'action', 'guard', 'idea']

export const KIND_ICON: Record<Kind, Component> = {
  card: PhSquaresFour,
  action: PhLightning,
  guard: PhShieldCheck,
  idea: PhLightbulb,
}

export const KIND_LABEL: Record<Kind, string> = {
  card: 'Card',
  action: 'Action',
  guard: 'Guard',
  idea: 'Idea',
}

/** Sort rank: card → action → guard → idea. */
export const KIND_RANK: Record<Kind, number> = {
  card: 0,
  action: 1,
  guard: 2,
  idea: 3,
}

export function kindOf(n: Pick<NodeView, 'kind'>): Kind {
  return n.kind
}
