//! ECS entity model: nodes are Markdown files whose frontmatter keys are
//! composable components. See design doc `03-data-contract-and-node-schema.md`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Node entity types. `Card` is the structural group node; `Action`, `Guard`
/// and `Idea` are the executable/quality-gate/speculative leaf kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Card,
    Action,
    Guard,
    Idea,
}

impl NodeKind {
    /// The uppercase file/ID prefix used by the Base36 hash strategy.
    pub fn prefix(self) -> &'static str {
        match self {
            NodeKind::Card => "CARD",
            NodeKind::Action => "ACT",
            NodeKind::Guard => "GRD",
            NodeKind::Idea => "IDEA",
        }
    }

    pub fn from_prefix(s: &str) -> Option<Self> {
        match s {
            "CARD" => Some(NodeKind::Card),
            "ACT" => Some(NodeKind::Action),
            "GRD" => Some(NodeKind::Guard),
            "IDEA" => Some(NodeKind::Idea),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            NodeKind::Card => "Card",
            NodeKind::Action => "Action",
            NodeKind::Guard => "Guard",
            NodeKind::Idea => "Idea",
        }
    }

    /// True for the non-container leaf kinds (Action/Guard/Idea).
    pub fn is_leaf_kind(self) -> bool {
        !matches!(self, NodeKind::Card)
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NodeKind::Card => "card",
            NodeKind::Action => "action",
            NodeKind::Guard => "guard",
            NodeKind::Idea => "idea",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for NodeKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "card" => Ok(NodeKind::Card),
            "action" => Ok(NodeKind::Action),
            "guard" => Ok(NodeKind::Guard),
            "idea" => Ok(NodeKind::Idea),
            other => Err(format!(
                "unknown node kind `{other}` (expected card, action, guard or idea)"
            )),
        }
    }
}

/// User-assigned lifecycle state stored in frontmatter.
///
/// `blocked` is intentionally *not* a stored state — it is computed at runtime
/// during DAG evaluation (see [`crate::dag`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Unstarted,
    Active,
    Vanquished,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Unstarted => "unstarted",
            Status::Active => "active",
            Status::Vanquished => "vanquished",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for Status {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "unstarted" => Ok(Status::Unstarted),
            "active" => Ok(Status::Active),
            "vanquished" => Ok(Status::Vanquished),
            other => Err(format!(
                "unknown status `{other}` (expected unstarted, active or vanquished)"
            )),
        }
    }
}

/// Execution importance; strictly single-value per entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "critical" => Ok(Priority::Critical),
            "high" => Ok(Priority::High),
            "medium" => Ok(Priority::Medium),
            "low" => Ok(Priority::Low),
            other => Err(format!(
                "unknown priority `{other}` (expected critical, high, medium or low)"
            )),
        }
    }
}

/// Fibonacci estimation points (spec §4.4).
pub const FIBONACCI_POINTS: [u32; 7] = [1, 2, 3, 5, 8, 13, 21];

/// Quest Point values are restricted to the Fibonacci set.
pub fn is_valid_quest_points(n: u32) -> bool {
    FIBONACCI_POINTS.contains(&n)
}

/// A single graph entity: composable frontmatter components.
///
/// Field order below is the canonical YAML key order used on read/write, which
/// keeps Git diffs deterministic across the whole graph (spec §4.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    #[serde(default)]
    pub disciplines: Vec<String>,
    #[serde(default)]
    pub epics: Vec<String>,
    #[serde(default)]
    pub quest_points: Option<u32>,
    #[serde(default)]
    pub landmark: Option<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub blocked_by: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Node {
    /// Quest Points of this node (0 for nodes without an estimate).
    pub fn quest_points(&self) -> u32 {
        self.quest_points.unwrap_or(0)
    }

    pub fn is_vanquished(&self) -> bool {
        self.status == Status::Vanquished
    }
}

/// A node file on disk: parsed frontmatter components plus the unmanaged
/// Markdown body, which is preserved verbatim on writes (spec §4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeFile {
    pub node: Node,
    pub body: String,
}

/// RFC3339 UTC timestamp (`2026-09-04T18:00:00Z`) for `created_at` /
/// `completed_at`.
pub fn utc_now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_parsing_roundtrip() {
        for k in [
            NodeKind::Card,
            NodeKind::Action,
            NodeKind::Guard,
            NodeKind::Idea,
        ] {
            assert_eq!(k.to_string().parse::<NodeKind>().unwrap(), k);
            assert_eq!(NodeKind::from_prefix(k.prefix()), Some(k));
        }
        assert!("epic".parse::<NodeKind>().is_err());
    }

    #[test]
    fn status_and_priority_parsing() {
        assert_eq!("vanquished".parse::<Status>().unwrap(), Status::Vanquished);
        assert_eq!("critical".parse::<Priority>().unwrap(), Priority::Critical);
        assert!(
            "blocked".parse::<Status>().is_err(),
            "blocked is computed, never stored"
        );
    }

    #[test]
    fn fibonacci_points_only() {
        assert!(is_valid_quest_points(3));
        assert!(is_valid_quest_points(21));
        assert!(!is_valid_quest_points(4));
        assert!(!is_valid_quest_points(0));
    }
}
