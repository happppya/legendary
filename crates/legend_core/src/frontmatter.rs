//! Frontmatter handling (spec §3, §4.2): parse `---`-fenced YAML into a
//! [`Node`], and render back in canonical key order while preserving the
//! unmanaged Markdown body verbatim.

use crate::model::{Node, NodeFile};
use serde_yaml::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrontmatterError {
    #[error("file does not begin with a `---` frontmatter fence")]
    MissingFence,
    #[error("unterminated `---` frontmatter fence")]
    UnterminatedFence,
    #[error("frontmatter YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("frontmatter key check failed: {0}")]
    KeyCheck(String),
}

/// Parsed halves of a node file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterParts {
    pub yaml: String,
    pub body: String,
}

/// Split text at the `---` frontmatter fences. Returns `None` when the file
/// does not start with a fence.
pub fn split(text: &str) -> Option<FrontmatterParts> {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.first().is_none_or(|l| l.trim_end() != "---") {
        return None;
    }
    let close = lines[1..]
        .iter()
        .position(|l| l.trim_end() == "---")
        .map(|i| i + 1)?;
    Some(FrontmatterParts {
        yaml: lines[1..close].join("\n"),
        body: lines[close + 1..].join("\n"),
    })
}

/// Parse just the component struct from node file text.
pub fn parse_node(text: &str) -> Result<Node, FrontmatterError> {
    let parts = split(text).ok_or(FrontmatterError::MissingFence)?;
    Ok(serde_yaml::from_str(&parts.yaml)?)
}

/// Parse a full node file (components + body).
pub fn parse_node_file(text: &str) -> Result<NodeFile, FrontmatterError> {
    let parts = split(text).ok_or(FrontmatterError::MissingFence)?;
    let node = serde_yaml::from_str(&parts.yaml)?;
    Ok(NodeFile {
        node,
        body: parts.body,
    })
}

/// Render a node file back to text with canonical frontmatter ordering.
///
/// The body (and therefore any unmanaged Markdown) is re-emitted untouched.
pub fn render_node_file(file: &NodeFile) -> Result<String, FrontmatterError> {
    let yaml = serde_yaml::to_string(&file.node)?;
    let mut out = String::from("---\n");
    out.push_str(&yaml);
    out.push_str("---\n");
    out.push_str(&file.body);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

/// Human-ish sanity check: parse, then verify YAML keys are exactly the
/// canonical component set (used by tests and `index sync` diagnostics).
pub fn check_canonical_keys(text: &str) -> Result<(), FrontmatterError> {
    let parts = split(text).ok_or(FrontmatterError::MissingFence)?;
    let value: Value = serde_yaml::from_str(&parts.yaml)?;
    let mapping = value
        .as_mapping()
        .ok_or_else(|| FrontmatterError::KeyCheck("frontmatter root must be a mapping".into()))?;
    let expected: &[&str] = &[
        "id",
        "kind",
        "title",
        "status",
        "priority",
        "disciplines",
        "epics",
        "quest_points",
        "landmark",
        "parent",
        "blocked_by",
        "created_at",
        "completed_at",
        "tags",
    ];
    for key in expected {
        if !mapping.contains_key(Value::String((*key).to_string())) {
            return Err(FrontmatterError::KeyCheck(format!(
                "missing component key `{key}`"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NodeKind;

    const SAMPLE: &str = "---\nid: ACT-3X7P\nkind: action\ntitle: Deduct Stamina on Dash Trigger\nstatus: active\npriority: high\ndisciplines:\n  - Programming/Locomotion\nepics:\n  - Combat_Engine/Locomotion\nquest_points: 3\nlandmark: null\nparent: CARD-K9F2\nblocked_by:\n  - ACT-8J3W\ncreated_at: 2026-09-04T18:10:00Z\ncompleted_at: null\ntags:\n  - mechanics\n---\n\n## Action Goal\nBody text with `code` and \"quotes\".\n";

    #[test]
    fn parses_components_and_body() {
        let file = parse_node_file(SAMPLE).unwrap();
        assert_eq!(file.node.id, "ACT-3X7P");
        assert_eq!(file.node.kind, NodeKind::Action);
        assert_eq!(file.node.blocked_by, vec!["ACT-8J3W".to_string()]);
        assert_eq!(file.node.parent.as_deref(), Some("CARD-K9F2"));
        assert!(file.body.starts_with("\n## Action Goal"));
        assert!(file.body.contains("Body text"));
    }

    #[test]
    fn roundtrip_preserves_wording() {
        let file = parse_node_file(SAMPLE).unwrap();
        let rendered = render_node_file(&file).unwrap();
        let reparsed = parse_node_file(&rendered).unwrap();
        assert_eq!(reparsed.node, file.node);
        assert_eq!(reparsed.body, file.body);
        assert!(rendered.contains("## Action Goal"));
        // canonical ordering: `id` line appears before `tags`
        let id_pos = rendered.find("\nid: ").unwrap();
        let tags_pos = rendered.find("\ntags:").unwrap();
        assert!(id_pos < tags_pos);
    }

    #[test]
    fn missing_fence_is_rejected() {
        assert!(matches!(
            parse_node_file("# no frontmatter here\nplain text"),
            Err(FrontmatterError::MissingFence)
        ));
    }

    #[test]
    fn canonical_keys_complete() {
        check_canonical_keys(SAMPLE).unwrap();
    }
}
