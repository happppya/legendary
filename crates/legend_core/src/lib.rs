//! # Legendary core engine
//!
//! Shared, high-performance Rust library powering both the `legend` CLI and the
//! Tauri desktop shell. Owns the ECS node model, YAML frontmatter parsing,
//! DAG traversal (blocked-state, cycle prevention, QP aggregation, landmark
//! inheritance) and realm indexing, per the design specification in
//! `notes/design-documents/`.

pub mod dag;
pub mod frontmatter;
pub mod id;
pub mod model;
pub mod realm;
pub mod registry;
pub mod scaffold;

pub use frontmatter::FrontmatterError;
pub use model::{Node, NodeFile, NodeKind, Priority, Status};
pub use realm::{NodeComputed, Realm, RealmError};
pub use registry::{LandmarkRegistry, Taxonomy};
