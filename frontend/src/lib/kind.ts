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

export function kindOf(n: Pick<NodeView, 'kind'>): Kind {
  return n.kind
}
