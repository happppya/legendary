//! Dependency & node resolution (spec §4.3–4.6): computed runtime blocked
//! state, strict cycle prevention & defensive detection, multi-level Quest
//! Point aggregation, locked-QP percentages, landmark inheritance, and the
//! forced-cascade vanquish rule.

use crate::model::{Node, NodeKind};
use std::collections::{HashMap, HashSet, VecDeque};

const MAX_DEPTH: usize = 1000;

/// Child node ids of `parent`, sorted for determinism.
pub fn children_sorted(nodes: &HashMap<String, Node>, parent: &str) -> Vec<String> {
    let mut ids: Vec<String> = nodes
        .values()
        .filter(|n| n.parent.as_deref() == Some(parent))
        .map(|n| n.id.clone())
        .collect();
    ids.sort();
    ids
}

/// Breadth-first descendant ids (excluding `root` itself).
pub fn descendants_ids(nodes: &HashMap<String, Node>, root: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.to_string());
    while let Some(id) = queue.pop_front() {
        for child in children_sorted(nodes, &id) {
            out.push(child.clone());
            queue.push_back(child);
        }
    }
    out
}

/// Whether an entity has any unvanquished prerequisite (spec §4.3): blocked
/// iff any node in `blocked_by` is not vanquished. `blocked` is computed at
/// runtime and never persisted.
pub fn is_blocked(nodes: &HashMap<String, Node>, id: &str) -> bool {
    let Some(node) = nodes.get(id) else {
        return false;
    };
    node.blocked_by
        .iter()
        .any(|dep| nodes.get(dep).map(|d| !d.is_vanquished()).unwrap_or(false))
}

/// Effective runtime status of a node:
///
/// - leaf kinds: `vanquished` if vanquished, else `blocked` when a
///   prerequisite is unvanquished, else their stored status;
/// - Cards: `blocked` if any direct active child is blocked, `vanquished`
///   when all children are vanquished (or the card itself is), else their
///   stored status.
///
/// Depth-capped so defensive indexing can never stack-overflow on manual
/// cycles (spec §4.3).
pub fn effective_status(nodes: &HashMap<String, Node>, id: &str) -> String {
    fn rec(nodes: &HashMap<String, Node>, id: &str, depth: usize) -> String {
        if depth > MAX_DEPTH {
            return "unknown".to_string();
        }
        let Some(node) = nodes.get(id) else {
            return "unknown".to_string();
        };
        if node.kind == NodeKind::Card {
            let kids = children_sorted(nodes, id);
            if kids.is_empty() {
                return node.status.to_string();
            }
            let mut any_blocked = false;
            let mut all_vanquished = true;
            for kid in &kids {
                let ks = rec(nodes, kid, depth + 1);
                if ks == "blocked" {
                    any_blocked = true;
                }
                if ks != "vanquished" {
                    all_vanquished = false;
                }
            }
            if any_blocked {
                "blocked".to_string()
            } else if all_vanquished {
                "vanquished".to_string()
            } else {
                node.status.to_string()
            }
        } else if node.is_vanquished() {
            "vanquished".to_string()
        } else if is_blocked(nodes, id) {
            "blocked".to_string()
        } else {
            node.status.to_string()
        }
    }
    rec(nodes, id, 0)
}

/// Effective landmark via recursive parent lookup (spec §4.6): an explicit
/// non-null `landmark` wins; otherwise inherit from the parent chain; else
/// `None`.
pub fn effective_landmark(nodes: &HashMap<String, Node>, id: &str) -> Option<String> {
    let mut current = id.to_string();
    let mut hops = 0;
    while hops <= nodes.len() {
        let node = nodes.get(&current)?;
        if let Some(lm) = &node.landmark {
            return Some(lm.clone());
        }
        match &node.parent {
            Some(p) => current = p.clone(),
            None => return None,
        }
        hops += 1;
    }
    None
}

/// Multi-level card point aggregation (spec §4.4): recursive post-order sum
/// over leaf Action/Guard/Idea nodes in the descendant tree. Intermediate
/// container nodes are never double-counted.
pub fn aggregate_qp(nodes: &HashMap<String, Node>, id: &str) -> u32 {
    fn rec(nodes: &HashMap<String, Node>, id: &str, depth: usize) -> u32 {
        if depth > MAX_DEPTH {
            return 0;
        }
        let Some(node) = nodes.get(id) else {
            return 0;
        };
        if node.kind.is_leaf_kind() {
            return node.quest_points();
        }
        children_sorted(nodes, id)
            .iter()
            .map(|c| rec(nodes, c, depth + 1))
            .sum()
    }
    rec(nodes, id, 0)
}

/// Percentage of descendant QP currently locked behind blocked prerequisites
/// (spec §4.3 formula). `None` when the container holds no points.
pub fn locked_qp_percent(nodes: &HashMap<String, Node>, id: &str) -> Option<f64> {
    let total = aggregate_qp(nodes, id);
    if total == 0 {
        return None;
    }
    let locked: u32 = leaf_tasks(nodes, id)
        .iter()
        .filter(|leaf| is_blocked(nodes, leaf))
        .map(|leaf| nodes.get(leaf).map_or(0, Node::quest_points))
        .sum();
    Some(locked as f64 * 100.0 / total as f64)
}

/// Leaf Action/Guard/Idea descendants of `root` (childless task nodes).
pub fn leaf_tasks(nodes: &HashMap<String, Node>, root: &str) -> Vec<String> {
    let mut leaves: Vec<String> = descendants_ids(nodes, root)
        .into_iter()
        .filter(|id| {
            let is_task = nodes.get(id).is_some_and(|n| n.kind.is_leaf_kind());
            let has_children = !children_sorted(nodes, id).is_empty();
            is_task && !has_children
        })
        .collect();
    leaves.sort();
    leaves
}

/// Cardinality of unvanquished leaf tasks under `root` (drives the
/// "N child tasks remaining" rule).
pub fn count_unvanquished_leaves(nodes: &HashMap<String, Node>, root: &str) -> usize {
    leaf_tasks(nodes, root)
        .iter()
        .filter(|id| nodes.get(*id).is_none_or(|n| !n.is_vanquished()))
        .count()
}

/// Would adding the candidate upstream edges create a dependency or
/// parent-child container cycle? (Spec §4.3: DFS validation on mutation.)
pub fn would_create_cycle(nodes: &HashMap<String, Node>, extra: &[(String, String)]) -> bool {
    fn dfs(
        edge_fn: &impl Fn(&str) -> Vec<(String, String)>,
        id: &str,
        color: &mut HashMap<String, u8>,
    ) -> bool {
        match color.get(id) {
            Some(1) => return true, // gray: back edge
            Some(2) => return false,
            _ => {}
        }
        color.insert(id.to_string(), 1);
        for (from, to) in edge_fn(id) {
            debug_assert_eq!(from, id);
            if dfs(edge_fn, &to, color) {
                return true;
            }
        }
        color.insert(id.to_string(), 2);
        false
    }
    let edge_fn = |id: &str| -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        if let Some(n) = nodes.get(id) {
            if let Some(p) = n.parent.as_deref() {
                out.push((id.to_string(), p.to_string()));
            }
            for dep in &n.blocked_by {
                out.push((id.to_string(), dep.clone()));
            }
        }
        out.extend(
            extra
                .iter()
                .filter(|(f, _)| f == id)
                .map(|(_, t)| (id.to_string(), t.clone())),
        );
        out
    };
    let ids: Vec<String> = nodes.keys().cloned().collect();
    for id in &ids {
        let mut color: HashMap<String, u8> = HashMap::new();
        if dfs(&edge_fn, id, &mut color) {
            return true;
        }
    }
    false
}

/// Detect existing cycles in a (possibly manually edited) graph and return
/// the ids participating in them, so `index sync` can flag affected nodes
/// with `validation_error: "Cycle detected"` and safely isolate them.
pub fn cyclic_ids(nodes: &HashMap<String, Node>) -> Vec<String> {
    // Kahn: repeatedly drop nodes with no unresolved upstream neighbor.
    let mut remaining: HashSet<String> = nodes.keys().cloned().collect();
    loop {
        let resolved: Vec<String> = remaining
            .iter()
            .filter(|id| {
                let Some(n) = nodes.get(*id) else {
                    return true;
                };
                let parents_ok = n.parent.as_ref().is_none_or(|p| !remaining.contains(p));
                let deps_ok = n.blocked_by.iter().all(|d| !remaining.contains(d));
                parents_ok && deps_ok
            })
            .cloned()
            .collect();
        if resolved.is_empty() {
            break;
        }
        for r in resolved {
            remaining.remove(&r);
        }
    }
    if remaining.is_empty() {
        return Vec::new();
    }
    // Walk the induced subgraph to find ids on actual closed paths.
    let in_set = |id: &str| remaining.contains(id);
    let mut members: HashSet<String> = HashSet::new();
    for start in remaining.iter() {
        if members.contains(start) {
            continue;
        }
        // DFS with explicit stack; record path; on back-edge mark the cycle.
        let mut stack: Vec<String> = vec![start.clone()];
        let mut path_set: HashSet<String> = HashSet::from([start.clone()]);
        let mut idx: HashMap<String, usize> = HashMap::new();
        idx.insert(start.clone(), 0);
        while let Some(top) = stack.last() {
            let node = nodes.get(top).unwrap();
            let mut nexts: Vec<String> = Vec::new();
            if let Some(p) = node.parent.as_deref() {
                if in_set(p) {
                    nexts.push(p.to_string());
                }
            }
            for dep in &node.blocked_by {
                if in_set(dep) {
                    nexts.push(dep.clone());
                }
            }
            nexts.sort();
            nexts.dedup();
            let mut advanced = false;
            if let Some(&cursor) = idx.get(top) {
                if let Some(next) = nexts.get(cursor) {
                    let next = next.clone();
                    let entry = idx.entry(top.clone()).or_insert(0);
                    *entry += 1;
                    if path_set.contains(&next) {
                        // back edge: mark nodes from `next` up the stack.
                        let pos = stack.iter().position(|s| s == &next).unwrap();
                        for n in &stack[pos..] {
                            members.insert(n.clone());
                        }
                        advanced = true;
                    } else {
                        stack.push(next.clone());
                        path_set.insert(next.clone());
                        idx.insert(next.clone(), 0);
                        advanced = true;
                    }
                }
            }
            if !advanced {
                let popped = stack.pop().unwrap();
                path_set.remove(&popped);
            }
        }
    }
    let mut out: Vec<String> = members.into_iter().collect();
    out.sort();
    out
}

/// Whether any cycle exists in the current graph.
pub fn has_cycle(nodes: &HashMap<String, Node>) -> bool {
    !cyclic_ids(nodes).is_empty()
}

/// Nodes to force-complete when a Card is vanquished with `--cascade`
/// (spec §4.3): every descendant leaf Action/Guard/Idea node.
pub fn cascade_ids(nodes: &HashMap<String, Node>, card: &str) -> Vec<String> {
    leaf_tasks(nodes, card)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeFile, Priority, Status};
    use std::collections::HashMap;

    fn task(
        id: &str,
        kind: NodeKind,
        parent: Option<&str>,
        deps: &[&str],
        qp: u32,
        status: Status,
    ) -> Node {
        Node {
            id: id.to_string(),
            kind,
            title: id.to_string(),
            status,
            priority: Priority::Medium,
            disciplines: vec![],
            epics: vec![],
            quest_points: Some(qp),
            landmark: None,
            parent: parent.map(|s| s.to_string()),
            blocked_by: deps.iter().map(|s| s.to_string()).collect(),
            created_at: None,
            completed_at: None,
            tags: vec![],
        }
    }

    fn card(id: &str, landmark: Option<&str>) -> Node {
        let mut n = task(id, NodeKind::Card, None, &[], 0, Status::Active);
        n.landmark = landmark.map(|s| s.to_string());
        n
    }

    fn graph(nodes: Vec<Node>) -> HashMap<String, Node> {
        nodes.into_iter().map(|n| (n.id.clone(), n)).collect()
    }

    #[test]
    fn blocked_state_chain() {
        let g = graph(vec![
            task("CARD-1", NodeKind::Card, None, &[], 0, Status::Active),
            task(
                "ACT-1",
                NodeKind::Action,
                Some("CARD-1"),
                &[],
                3,
                Status::Active,
            ),
            task(
                "ACT-2",
                NodeKind::Action,
                Some("CARD-1"),
                &["ACT-1"],
                5,
                Status::Active,
            ),
            task(
                "GRD-1",
                NodeKind::Guard,
                Some("CARD-1"),
                &["ACT-2"],
                1,
                Status::Unstarted,
            ),
            task(
                "IDEA-1",
                NodeKind::Idea,
                Some("CARD-1"),
                &[],
                2,
                Status::Unstarted,
            ),
        ]);
        assert!(!is_blocked(&g, "ACT-1"));
        assert!(is_blocked(&g, "ACT-2"), "prerequisite ACT-1 is active");
        assert!(is_blocked(&g, "GRD-1"));
        assert_eq!(effective_status(&g, "ACT-1"), "active");
        assert_eq!(effective_status(&g, "ACT-2"), "blocked");
        assert_eq!(effective_status(&g, "GRD-1"), "blocked");
        assert_eq!(effective_status(&g, "CARD-1"), "blocked");
    }

    #[test]
    fn vanquished_prereq_unblocks() {
        let mut g = graph(vec![
            task("CARD-1", NodeKind::Card, None, &[], 0, Status::Active),
            task(
                "ACT-1",
                NodeKind::Action,
                Some("CARD-1"),
                &[],
                3,
                Status::Vanquished,
            ),
            task(
                "ACT-2",
                NodeKind::Action,
                Some("CARD-1"),
                &["ACT-1"],
                5,
                Status::Active,
            ),
        ]);
        assert!(!is_blocked(&g, "ACT-2"));
        assert_eq!(effective_status(&g, "ACT-2"), "active");
        // Card now vanquished when all leaves are
        g.get_mut("ACT-2").unwrap().status = Status::Vanquished;
        assert_eq!(effective_status(&g, "CARD-1"), "vanquished");
    }

    #[test]
    fn qp_aggregation_and_locking() {
        let g = graph(vec![
            task("CARD-1", NodeKind::Card, None, &[], 0, Status::Active),
            task(
                "CARD-2",
                NodeKind::Card,
                Some("CARD-1"),
                &[],
                0,
                Status::Active,
            ),
            task(
                "ACT-1",
                NodeKind::Action,
                Some("CARD-2"),
                &[],
                3,
                Status::Active,
            ),
            task(
                "ACT-2",
                NodeKind::Action,
                Some("CARD-1"),
                &["ACT-1"],
                5,
                Status::Active,
            ),
            task(
                "IDEA-1",
                NodeKind::Idea,
                Some("CARD-1"),
                &[],
                2,
                Status::Unstarted,
            ),
        ]);
        assert_eq!(
            aggregate_qp(&g, "CARD-1"),
            10,
            "nested card not double-counted"
        );
        let pct = locked_qp_percent(&g, "CARD-1").unwrap();
        // only blocked leaf is ACT-2 (5 QP, prerequisite ACT-1 active): 5/10
        assert!((pct - 50.0).abs() < 0.001);
    }

    #[test]
    fn landmark_inheritance_and_override() {
        let g = graph(vec![
            card("CARD-1", Some("Landmark_01_Demo")),
            task(
                "ACT-1",
                NodeKind::Action,
                Some("CARD-1"),
                &[],
                1,
                Status::Active,
            ),
        ]);
        assert_eq!(
            effective_landmark(&g, "ACT-1").as_deref(),
            Some("Landmark_01_Demo")
        );

        let mut g2 = graph(vec![
            card("CARD-1", Some("Landmark_01_Demo")),
            task(
                "ACT-1",
                NodeKind::Action,
                Some("CARD-1"),
                &[],
                1,
                Status::Active,
            ),
        ]);
        g2.get_mut("ACT-1").unwrap().landmark = Some("Landmark_02_Slice".to_string());
        assert_eq!(
            effective_landmark(&g2, "ACT-1").as_deref(),
            Some("Landmark_02_Slice")
        );
    }

    #[test]
    fn cycle_detection_and_mutation_guard() {
        let g = graph(vec![
            task(
                "ACT-1",
                NodeKind::Action,
                None,
                &["ACT-2"],
                1,
                Status::Active,
            ),
            task(
                "ACT-2",
                NodeKind::Action,
                None,
                &["ACT-1"],
                1,
                Status::Active,
            ),
        ]);
        assert!(has_cycle(&g));
        let cyc = cyclic_ids(&g);
        assert_eq!(cyc.len(), 2, "both ids participate in the loop");

        let mut g2 = graph(vec![
            task("ACT-1", NodeKind::Action, None, &[], 1, Status::Active),
            task(
                "ACT-2",
                NodeKind::Action,
                None,
                &["ACT-1"],
                1,
                Status::Active,
            ),
        ]);
        assert!(!has_cycle(&g2));
        // ACT-1 → ACT-2 dependency would close a loop
        assert!(would_create_cycle(&g2, &[("ACT-1".into(), "ACT-2".into())]));
        // parent-child container loop
        g2.get_mut("ACT-1").unwrap().parent = Some("ACT-2".to_string());
        assert!(has_cycle(&g2));
    }

    #[test]
    fn cascade_targets_and_card_vanquish_rule() {
        let g = graph(vec![
            task("CARD-1", NodeKind::Card, None, &[], 0, Status::Active),
            task(
                "CARD-2",
                NodeKind::Card,
                Some("CARD-1"),
                &[],
                0,
                Status::Active,
            ),
            task(
                "ACT-1",
                NodeKind::Action,
                Some("CARD-2"),
                &[],
                3,
                Status::Active,
            ),
            task(
                "GRD-1",
                NodeKind::Guard,
                Some("CARD-1"),
                &["ACT-1"],
                1,
                Status::Unstarted,
            ),
        ]);
        assert_eq!(count_unvanquished_leaves(&g, "CARD-1"), 2);
        let targets = cascade_ids(&g, "CARD-1");
        assert_eq!(targets, vec!["ACT-1".to_string(), "GRD-1".to_string()]);
    }

    #[test]
    fn realm_file_helper_placeholder() {
        // NodeFile is produced by the frontmatter layer; just keep the import
        // surface exercised.
        let _ = std::mem::size_of::<NodeFile>();
    }
}
