---
id: ACT-3X7P
kind: action                    # Entity type: card | action | guard | idea
title: Deduct Stamina on Dash Trigger
status: active                  # Note: Dynamically evaluated as blocked if prerequisites unvanquished
priority: high
disciplines:
  - Programming/Locomotion
  - Programming/Systems
epics:
  - Combat_Engine/Locomotion
quest_points: 3                 # Fibonacci: 1, 2, 3, 5, 8, 13, 21
landmark: null                  # Inherits Landmark_01_Demo from CARD-K9F2
parent: CARD-K9F2
blocked_by:
  - ACT-8J3W
created_at: 2026-09-04T18:10:00Z
completed_at: null
tags:
  - mechanics
---

## Action Goal
Deduct 25 stamina points from `PlayerStaminaPool` immediately when the dash input is accepted.
