//! Node ID system (spec §3.3): randomized Base36 IDs shaped as
//! `[KIND_PREFIX]-[BASE36_HASH]`, e.g. `CARD-K9F2`, `ACT-3X7P`, `GRD-7M2Q`,
//! `IDEA-5T2P`. IDs are the immutable primary key; collision resistance is
//! handled by re-rolling or extending the hash length.

use crate::model::NodeKind;
use std::fmt;

const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const BASE: u32 = 36;

/// Default randomized hash length (4 Base36 chars → 36⁴ = 1,679,616 combos).
pub const DEFAULT_ID_LEN: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeId {
    pub kind: NodeKind,
    pub hash: String,
}

impl NodeId {
    /// Parse an ID string such as `ACT-3X7P`. Hash may exceed 4 chars (a
    /// collision-extension re-roll like `ACT-3X7PA`) but never be shorter.
    pub fn parse(s: &str) -> Result<Self, String> {
        let (prefix, hash) = s
            .split_once('-')
            .ok_or_else(|| format!("invalid node id `{s}`: expected `KIND-HASH`"))?;
        let kind = NodeKind::from_prefix(prefix)
            .ok_or_else(|| format!("invalid node id `{s}`: unknown prefix `{prefix}`"))?;
        let hash = hash.to_ascii_uppercase();
        if hash.len() < DEFAULT_ID_LEN {
            return Err(format!(
                "invalid node id `{s}`: hash must be at least {DEFAULT_ID_LEN} Base36 chars"
            ));
        }
        if !hash.bytes().all(|b| CHARSET.contains(&b)) {
            return Err(format!(
                "invalid node id `{s}`: hash must be Base36 (0-9, A-Z)"
            ));
        }
        Ok(NodeId { kind, hash })
    }

    /// Render the canonical `KIND-HASH` string.
    pub fn as_str(&self) -> String {
        format!("{}-{}", self.kind.prefix(), self.hash)
    }
}

fn random_base36_char() -> char {
    CHARSET[fastrand::u32(..BASE) as usize] as char
}

/// File slug for a title: lowercased alphanumerics separated by single
/// underscores (`Grappling Hook / Momentum` → `grappling_hook_momentum`).
/// Used for the `[ID]_[slug].md` naming scheme (spec §3.4).
pub fn slugify_title(title: &str) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("node");
    }
    out
}

/// Generate a fresh ID for `kind`, asking `taken` before accepting a hash so
/// the length is extended on collision (re-roll strategy, spec §3.3).
pub fn generate_unique(kind: NodeKind, taken: impl Fn(&str) -> bool) -> NodeId {
    let mut len = DEFAULT_ID_LEN;
    loop {
        let hash: String = (0..len).map(|_| random_base36_char()).collect();
        let id = NodeId { kind, hash };
        if !taken(&id.as_str()) {
            return id;
        }
        len += 1;
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spec_examples() {
        for s in ["CARD-K9F2", "ACT-3X7P", "GRD-7M2Q", "IDEA-5T2P"] {
            let id = NodeId::parse(s).unwrap();
            assert_eq!(id.as_str(), s);
        }
    }

    #[test]
    fn rejects_malformed() {
        assert!(NodeId::parse("CARD").is_err());
        assert!(NodeId::parse("CARD-").is_err());
        assert!(NodeId::parse("CARD-K9").is_err());
        assert!(NodeId::parse("NOPE-K9F2").is_err());
        assert!(NodeId::parse("ACT-K9F2!").is_err());
        // collision extension (5 chars) is legal
        assert!(NodeId::parse("ACT-3X7PA").is_ok());
    }

    #[test]
    fn generated_ids_have_shape() {
        for _ in 0..200 {
            let id = generate_unique(NodeKind::Action, |_| false);
            assert_eq!(id.kind, NodeKind::Action);
            assert_eq!(id.hash.len(), DEFAULT_ID_LEN);
            assert!(id.hash.bytes().all(|b| CHARSET.contains(&b)));
            let reparsed = NodeId::parse(&id.as_str()).unwrap();
            assert_eq!(reparsed, id);
        }
    }

    #[test]
    fn slugify_matches_examples() {
        assert_eq!(slugify_title("Dash Stamina Cost"), "dash_stamina_cost");
        assert_eq!(
            slugify_title("Grappling Hook / Momentum"),
            "grappling_hook_momentum"
        );
        assert_eq!(slugify_title("UI & HUD (2026)"), "ui_hud_2026");
        assert_eq!(slugify_title("----"), "node");
        assert_eq!(
            slugify_title("  Title--with  spaces  "),
            "title_with_spaces"
        );
    }

    #[test]
    fn collision_triggers_extension() {
        // pretend the 4-char space is exhausted for this kind: all 4-char
        // attempts collide, so generation extends to 5 chars.
        // treat every 4-char hash as taken so generation must extend to 5
        let taken = |candidate: &str| {
            candidate
                .rsplit_once('-')
                .is_some_and(|(_, h)| h.len() <= DEFAULT_ID_LEN)
        };
        let id = generate_unique(NodeKind::Card, taken);
        assert!(id.hash.len() > DEFAULT_ID_LEN);
    }
}
