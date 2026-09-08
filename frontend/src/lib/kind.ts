// Shared kind vocabulary for the node index and detail sheet. Single source
// of truth for the icon + label + ordering used everywhere in the renderer.

import type { Component } from 'vue'
import {
  PhLightbulb,
  PhLightning,
  PhShieldCheck,
  PhSquaresFour,
  PhTreeStructure,
} from '@phosphor-icons/vue'
import type { NodeView } from '../types'

export type Kind = NodeView['kind']

/** Kind keys in canonical display order. */
export const KINDS: readonly Kind[] = ['card', 'genre', 'action', 'guard', 'idea']

export const KIND_ICON: Record<Kind, Component> = {
  card: PhSquaresFour,
  genre: PhTreeStructure,
  action: PhLightning,
  guard: PhShieldCheck,
  idea: PhLightbulb,
}

export const KIND_LABEL: Record<Kind, string> = {
  card: 'Card',
  genre: 'Genre',
  action: 'Action',
  guard: 'Guard',
  idea: 'Idea',
}

/** Sort rank: card → genre → action → guard → idea. */
export const KIND_RANK: Record<Kind, number> = {
  card: 0,
  genre: 1,
  action: 2,
  guard: 3,
  idea: 4,
}

/** Container kinds: structural groups that hold subgraphs and aggregate
 * descendant QP. Non-containers are leaf task kinds. */
export function isContainerKind(kind: string): boolean {
  return kind === 'card' || kind === 'genre'
}

export function kindOf(n: Pick<NodeView, 'kind'>): Kind {
  return n.kind
}
