---
id: GRD-7M2Q
kind: guard
title: 'Code Review: Dash Mechanics & Memory Leaks'
status: unstarted
priority: medium
disciplines:
- Programming/Optimization
- QualityAssurance
epics:
- Combat_Engine/Locomotion
quest_points: 1
landmark: null
parent: CARD-K9F2
blocked_by: []
created_at: 2026-09-04T18:15:00Z
completed_at: null
tags:
- qa
- code_review
---

## Quality Check Criteria
- Verify state machine transition cleanup.
- Ensure no memory allocations in update loop.
