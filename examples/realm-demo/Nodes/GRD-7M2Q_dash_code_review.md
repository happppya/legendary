---
id: GRD-7M2Q
kind: guard                     # Entity type: card | action | guard | idea
title: "Code Review: Dash Mechanics & Memory Leaks"
status: unstarted
priority: medium
disciplines:
  - Programming/Optimization
  - QualityAssurance
epics:
  - Combat_Engine/Locomotion
quest_points: 1
landmark: null                  # Inherits Landmark_01_Demo from CARD-K9F2
parent: CARD-K9F2
blocked_by:
  - ACT-3X7P
created_at: 2026-09-04T18:15:00Z
completed_at: null
tags:
  - qa
  - code_review
---

## Quality Check Criteria
- Verify state machine transition cleanup.
- Ensure no memory allocations in update loop.
