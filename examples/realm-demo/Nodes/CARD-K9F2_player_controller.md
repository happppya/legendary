---
id: CARD-K9F2
kind: card                      # Entity type: card | action | guard | idea
title: Player Movement & Controller Core
status: active                  # User-set lifecycle: unstarted | active | vanquished
priority: high                  # Single-value component: critical | high | medium | low
disciplines:                    # Multi-value component (Validated against taxonomy.yaml)
  - Programming/Locomotion
  - Programming/Input
epics:                          # Multi-value component (Validated against taxonomy.yaml)
  - Combat_Engine/Locomotion
  - Core_Systems/Player
landmark: Landmark_01_Demo      # Validated against landmarks.yaml / Landmarks folder
parent: null
blocked_by: []
created_at: 2026-09-04T18:00:00Z
completed_at: null
tags:
  - player
  - locomotion
---

## System Context & Architectural Design
This card encompasses all locomotion mechanics, input buffer handling, and stamina consumption rules for the main player avatar.
