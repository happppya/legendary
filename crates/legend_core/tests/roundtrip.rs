//! Round-trip guarantees (ROADMAP Milestone 1): parse -> render -> parse must
//! be lossless for the components (`Node`) and the unmanaged Markdown body.
//!
//! Covers three corpora:
//! 1. every node file in the checked-in `examples/realm-demo` realm,
//! 2. canonical node samples mirroring design doc `03` schemas,
//! 3. a seeded pseudo-random fuzz over arbitrary component strings, so
//!    quoting edge cases in YAML round-trip cleanly.

use legend_core::frontmatter::{parse_node_file, render_node_file};
use legend_core::model::{Node, NodeFile, NodeKind, Priority, Status};
use std::fs;
use std::path::{Path, PathBuf};

fn assert_lossless(original: &NodeFile) {
    let rendered = render_node_file(original).expect("render");
    let reparsed = parse_node_file(&rendered).expect("re-parse of rendered output");
    assert_eq!(reparsed.node, original.node, "components must round-trip");
    assert_eq!(
        reparsed.body, original.body,
        "markdown body must round-trip"
    );
    // Rendering is deterministic: a second pass yields identical bytes.
    let again = render_node_file(&reparsed).expect("re-render");
    assert_eq!(again, rendered, "rendered output must be stable");
}

fn canonical_file(text: &str) -> NodeFile {
    parse_node_file(text).expect("canonical sample parses")
}

/// 1. Every node file in the sample realm.
#[test]
fn sample_realm_nodes_roundtrip_losslessly() {
    let nodes_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/realm-demo")
        .join("Nodes");
    let mut files: Vec<PathBuf> = fs::read_dir(&nodes_dir)
        .expect("examples/realm-demo/Nodes")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "sample realm must have node files");
    for path in files {
        let text = fs::read_to_string(&path).expect("read node file");
        let file =
            parse_node_file(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        assert_lossless(&file);
    }
}

/// 2. Canonical schemas from design doc 03 (all four node kinds).
#[test]
fn canonical_doc_samples_roundtrip_losslessly() {
    let card = canonical_file(
        "---
id: CARD-K9F2
kind: card
title: Player Movement & Controller Core
status: active
priority: high
disciplines:
  - Programming/Locomotion
  - Programming/Input
epics:
  - Combat_Engine/Locomotion
  - Core_Systems/Player
quest_points: null
landmark: Landmark_01_Demo
parent: null
blocked_by: []
created_at: 2026-09-04T18:00:00Z
completed_at: null
tags:
  - player
  - locomotion
---

## System Context & Architectural Design
This card encompasses all locomotion mechanics, input buffer handling, and
stamina consumption rules for the main player avatar.
",
    );
    let action = canonical_file(
        "---
id: ACT-3X7P
kind: action
title: Deduct Stamina on Dash Trigger
status: active
priority: high
disciplines:
  - Programming/Locomotion
  - Programming/Systems
epics:
  - Combat_Engine/Locomotion
quest_points: 3
landmark: null
parent: CARD-K9F2
blocked_by:
  - ACT-8J3W
created_at: 2026-09-04T18:10:00Z
completed_at: null
tags:
  - mechanics
---

## Action Goal
Deduct 25 stamina points from PlayerStaminaPool immediately when the dash
input is accepted.
",
    );
    let guard = canonical_file(
        "---
id: GRD-7M2Q
kind: guard
title: \"Code Review: Dash Mechanics & Memory Leaks\"
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
",
    );
    let idea = canonical_file(
        "---
id: IDEA-5T2P
kind: idea
title: Grappling Hook Physics & Momentum Retention
status: unstarted
priority: low
disciplines:
  - Programming/Locomotion
  - Art/Concept
epics:
  - Combat_Engine/Locomotion
quest_points: 5
landmark: null
parent: CARD-K9F2
blocked_by: []
created_at: 2026-09-05T09:00:00Z
completed_at: null
tags:
  - concept
  - pitch
---

## Idea Concept & Mechanics Exploration
Investigate physics-based cable simulation for momentum-preserving grappling
locomotion in air combat.
",
    );
    for sample in [card, action, guard, idea] {
        assert_lossless(&sample);
    }
}

/// Deterministic xorshift32 PRNG so failures are reproducible.
struct Rng(u32);

impl Rng {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    fn below(&mut self, n: u32) -> u32 {
        self.next_u32() % n
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u32) as usize]
    }

    /// A random string over characters that stress YAML quoting/escaping.
    fn text(&mut self, max_len: u32) -> String {
        const CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 '#:\"&*[]{}!?/.,()_-+%@$`~|><=;";
        let len = 1 + self.below(max_len);
        (0..len)
            .map(|_| *self.pick(CHARS.as_bytes()) as char)
            .collect::<String>()
    }
}

/// 3. Seeded fuzz: arbitrary component strings must survive the round trip.
#[test]
fn randomized_nodes_roundtrip_losslessly() {
    let mut rng = Rng(0x5EED_CAFE);
    let kinds = [
        NodeKind::Card,
        NodeKind::Action,
        NodeKind::Guard,
        NodeKind::Idea,
    ];
    let statuses = [Status::Unstarted, Status::Active, Status::Vanquished];
    let priorities = [
        Priority::Critical,
        Priority::High,
        Priority::Medium,
        Priority::Low,
    ];
    let prefixes = ["CARD", "ACT", "GRD", "IDEA"];

    for i in 0..400 {
        let mut node = Node {
            id: format!("{}-{}", rng.pick(&prefixes), i % 9000),
            kind: *rng.pick(&kinds),
            title: rng.text(24),
            status: *rng.pick(&statuses),
            priority: *rng.pick(&priorities),
            disciplines: (0..rng.below(3)).map(|_| rng.text(16)).collect(),
            epics: (0..rng.below(3)).map(|_| rng.text(16)).collect(),
            quest_points: if rng.below(2) == 0 {
                Some(rng.below(22))
            } else {
                None
            },
            landmark: if rng.below(3) == 0 {
                Some(rng.text(14))
            } else {
                None
            },
            parent: if rng.below(3) == 0 {
                Some(format!("CARD-X{}X", rng.below(100)))
            } else {
                None
            },
            blocked_by: (0..rng.below(3))
                .map(|_| format!("ACT-Y{}Y", rng.below(100)))
                .collect(),
            created_at: if rng.below(2) == 0 {
                Some("2026-09-04T18:00:00Z".to_string())
            } else {
                None
            },
            completed_at: None,
            tags: (0..rng.below(4)).map(|_| rng.text(12)).collect(),
        };
        if node.kind == NodeKind::Card {
            node.quest_points = None; // cards aggregate, never estimate directly
        }
        let file = NodeFile {
            node,
            body: format!(
                "\n## Random Body {}\n\ntext with `code` and quoted \"words\" in it.\n- a\n- b\n",
                i
            ),
        };
        assert_lossless(&file);
    }
}
