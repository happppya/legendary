//! `legend` — the agent-first CLI over a Legendary realm (spec §6.4).
//!
//! Every write/mutation command synchronously rewrites `.legend/index.json`
//! and `.legend/INDEX.md` before returning exit status 0. All mutation
//! semantics live in `legend_core::ops` so the CLI and the desktop shell
//! share one implementation and cannot drift.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use legend_core::model::{NodeKind, Priority, Status};
use legend_core::ops::{self, CreateArgs, MutationOutcome, OpsError, UpdateArgs};
use legend_core::scaffold::scaffold;
use legend_core::{NodeComputed, Realm};
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "legend",
    version,
    about = "Legendary — local-first Markdown DAG task & lore engine"
)]
struct Cli {
    /// Path to the realm directory (contains `.legend/`, `Nodes/`, ...).
    #[arg(long, global = true, default_value = ".")]
    realm: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new realm directory (flat layout + agent readme)
    Init {
        /// Directory to create the realm in (must be new or empty)
        dir: PathBuf,
    },
    /// Query nodes by discipline, epic, kind, status or landmark
    Query {
        #[arg(long)]
        discipline: Option<String>,
        #[arg(long)]
        epic: Option<String>,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        landmark: Option<String>,
        #[arg(long)]
        status: Option<String>,
        /// Only tasks that are currently unblocked (not blocked, not vanquished)
        #[arg(long)]
        unblocked: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show full node metadata, effective landmark, runtime status and body
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Print ancestor/descendant tree of a node
    Tree { id: String },
    /// Create a new node
    Create {
        kind: String,
        title: String,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        quest_points: Option<u32>,
        #[arg(long)]
        json: bool,
    },
    /// Update a node's status (vanquish with optional --cascade for Cards)
    Update {
        id: String,
        #[arg(long)]
        status: Option<String>,
        /// Force-complete all child tasks when vanquishing a Card
        #[arg(long)]
        cascade: bool,
        #[arg(long)]
        json: bool,
    },
    /// Rename a node: update its title and re-derive the file slug
    Rename {
        id: String,
        /// New title (required); the file slug is re-derived from it
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Wrap a node in a new Card group (Promote to Card)
    Wrap {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Re-parent a node or sub-tree (cycle-checked)
    Reparent {
        id: String,
        /// New parent id; omit to promote to root (`parent: null`)
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Delete a node or subgraph (strips dangling blocked_by references)
    Delete {
        id: String,
        /// Recursively delete the whole subgraph
        #[arg(long)]
        recursive: bool,
    },
    /// Regenerate `.legend/index.json` + `.legend/INDEX.md`
    Index {
        #[command(subcommand)]
        action: IndexAction,
    },
}

#[derive(Subcommand)]
enum IndexAction {
    /// Rebuild indexes and run cycle validation after manual external edits
    Sync,
}

fn open_realm(cli: &Cli) -> Result<Realm> {
    Realm::open(&cli.realm)
        .with_context(|| format!("failed to open realm `{}`", cli.realm.display()))
}

fn node_json(realm: &Realm, id: &str) -> Option<Value> {
    let file = realm.file(id)?;
    let n = &file.node;
    let comp: &NodeComputed = realm.computed.get(id)?;
    Some(json!({
        "id": n.id,
        "kind": n.kind.to_string(),
        "title": n.title,
        "status": n.status.to_string(),
        "effective_status": comp.effective_status,
        "blocked": comp.blocked,
        "priority": n.priority.to_string(),
        "disciplines": n.disciplines,
        "epics": n.epics,
        "quest_points": n.quest_points,
        "landmark": comp.effective_landmark,
        "parent": n.parent,
        "blocked_by": n.blocked_by,
        "total_qp": comp.total_qp,
        "locked_qp_percent": comp.locked_qp_pct,
        "completed_at": n.completed_at,
        "created_at": n.created_at,
        "validation_error": comp.validation_error,
        "path": format!("Nodes/{}", realm.file_names.get(id).map_or(id, |v| v.as_str())),
    }))
}

fn print_node_line(realm: &Realm, id: &str) {
    let n = &realm.nodes[id].node;
    let comp = &realm.computed[id];
    let lm = comp.effective_landmark.as_deref().unwrap_or("-");
    let qp = comp.total_qp;
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        n.id, n.kind, comp.effective_status, lm, qp, n.title
    );
}

fn query(realm: &Realm, cli: &Cli) -> Result<()> {
    let Command::Query {
        discipline,
        epic,
        kind,
        landmark,
        status,
        unblocked,
        json,
    } = &cli.command
    else {
        unreachable!()
    };

    let kind_filter = match kind {
        Some(k) => Some(k.parse::<NodeKind>().map_err(anyhow::Error::msg)?),
        None => None,
    };
    // `--status` filters on the *computed runtime* status (spec §6.4: e.g.
    // `--status blocked` lists nodes held up by prerequisites). `blocked` is
    // never stored, so it is accepted here but rejected by `update`.
    let status_filter = match status {
        Some(s) => {
            if !matches!(
                s.as_str(),
                "unstarted" | "active" | "blocked" | "vanquished"
            ) {
                bail!(
                    "unknown status filter `{s}` (expected unstarted, active, blocked or vanquished)"
                );
            }
            Some(s.clone())
        }
        None => None,
    };

    let matches = |id: &str| -> bool {
        let n = &realm.nodes[id].node;
        let comp = &realm.computed[id];
        if let Some(k) = kind_filter {
            if n.kind != k {
                return false;
            }
        }
        if let Some(s) = &status_filter {
            if comp.effective_status != *s {
                return false;
            }
        }
        if *unblocked && (comp.blocked || n.is_vanquished()) {
            return false;
        }
        if let Some(lm) = landmark {
            if comp.effective_landmark.as_deref() != Some(lm) {
                return false;
            }
        }
        if let Some(d) = discipline {
            if !n
                .disciplines
                .iter()
                .any(|p| p == d || p.starts_with(&format!("{d}/")))
            {
                return false;
            }
        }
        if let Some(e) = epic {
            if !n
                .epics
                .iter()
                .any(|p| p == e || p.starts_with(&format!("{e}/")))
            {
                return false;
            }
        }
        true
    };

    let ids: Vec<String> = realm
        .sorted_ids()
        .into_iter()
        .filter(|i| matches(i))
        .collect();
    if *json {
        let arr: Vec<Value> = ids.iter().filter_map(|i| node_json(realm, i)).collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
    } else {
        for id in &ids {
            print_node_line(realm, id);
        }
    }
    Ok(())
}

fn show(realm: &Realm, cli: &Cli) -> Result<()> {
    let Command::Show { id, json } = &cli.command else {
        unreachable!()
    };
    if !realm.nodes.contains_key(id) {
        bail!("no such node `{id}`");
    }
    if *json {
        println!(
            "{}",
            serde_json::to_string_pretty(&node_json(realm, id).unwrap())?
        );
        return Ok(());
    }
    let file = &realm.nodes[id];
    let n = &file.node;
    let comp = &realm.computed[id];
    println!("id:              {}", n.id);
    println!("kind:            {}", n.kind.label());
    println!("title:           {}", n.title);
    println!(
        "status:          {} (effective: {})",
        n.status, comp.effective_status
    );
    println!("priority:        {}", n.priority);
    println!("disciplines:     {}", n.disciplines.join(", "));
    println!("epics:           {}", n.epics.join(", "));
    println!("quest_points:    {}", comp.total_qp);
    println!(
        "landmark:        {}",
        comp.effective_landmark.as_deref().unwrap_or("-")
    );
    println!("parent:          {}", n.parent.as_deref().unwrap_or("-"));
    println!("blocked_by:      {}", n.blocked_by.join(", "));
    println!(
        "created_at:      {}",
        n.created_at.as_deref().unwrap_or("-")
    );
    println!(
        "completed_at:    {}",
        n.completed_at.as_deref().unwrap_or("-")
    );
    println!("tags:            {}", n.tags.join(", "));
    if let Some(ve) = &comp.validation_error {
        println!("validation_error: {ve}");
    }
    let body = file.body.trim_matches('\n');
    if !body.is_empty() {
        println!("--- body ---");
        println!("{}", file.body.trim());
    }
    Ok(())
}

fn tree(realm: &Realm, cli: &Cli) -> Result<()> {
    let Command::Tree { id } = &cli.command else {
        unreachable!()
    };
    if !realm.nodes.contains_key(id) {
        bail!("no such node `{id}`");
    }
    // Ancestors first (root-most at the top).
    let mut chain: Vec<String> = Vec::new();
    let mut cur: Option<String> = Some(id.clone());
    let mut hops = 0;
    while let Some(c) = cur {
        if hops > realm.nodes.len() {
            break;
        }
        chain.push(c.clone());
        cur = realm.node(&c).and_then(|n| n.parent.clone());
        hops += 1;
    }
    chain.reverse();
    if chain.len() > 1 {
        println!("ancestors:");
        for a in &chain[..chain.len() - 1] {
            print_node_line(realm, a);
        }
        println!();
    }
    println!("tree under {id}:");
    let mut stack: Vec<(String, usize)> = vec![(id.clone(), 0)];
    while let Some((node_id, depth)) = stack.pop() {
        let n = &realm.nodes[&node_id].node;
        let comp = &realm.computed[&node_id];
        let indent = "  ".repeat(depth);
        println!(
            "{indent}{} [{}] {}{}",
            match n.kind {
                NodeKind::Card => "▪",
                NodeKind::Genre => "◫",
                NodeKind::Guard => "◇",
                NodeKind::Idea => "○",
                NodeKind::Action => "•",
            },
            n.id,
            n.title,
            if comp.blocked { " (blocked)" } else { "" }
        );
        let mut kids = realm.children(&node_id);
        kids.reverse();
        for k in kids {
            stack.push((k, depth + 1));
        }
    }
    Ok(())
}

/// Report an [`MutationOutcome`]: the one-line summary plus, with `--json`,
/// the affected node payloads. Used by every mutation command.
fn report(realm: &Realm, out: &MutationOutcome, json: bool, focus_id: &str) -> Result<()> {
    println!("{}", out.message);
    if json {
        let mut ids: Vec<&String> = out.created.iter().collect::<Vec<_>>();
        ids.extend(out.changed.iter());
        let focus = ids.iter().find(|i| i.as_str() == focus_id).copied();
        let show = focus.map(|s| s.as_str()).unwrap_or(focus_id);
        if realm.nodes.contains_key(show) {
            println!(
                "{}",
                serde_json::to_string_pretty(&node_json(realm, show).unwrap())?
            );
        }
    }
    Ok(())
}

/// Map a shared [`OpsError`] onto process exit semantics (message on stderr,
/// exit status 1 for agent pipelines).
fn fail(err: OpsError) -> anyhow::Error {
    anyhow::Error::msg(err.to_string())
}

fn create(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Create {
        kind,
        title,
        parent,
        priority,
        quest_points,
        json,
    } = &cli.command
    else {
        unreachable!()
    };
    let kind = kind.parse::<NodeKind>().map_err(anyhow::Error::msg)?;
    let priority = match priority {
        Some(p) => p.parse::<Priority>().map_err(anyhow::Error::msg)?,
        None => Priority::Medium,
    };
    let out = ops::create(
        realm,
        CreateArgs {
            kind,
            title: title.clone(),
            parent: parent.clone(),
            priority,
            quest_points: *quest_points,
        },
    )
    .map_err(fail)?;
    let id = out.created[0].clone();
    report(realm, &out, *json, &id)
}

fn init(cli: &Cli) -> Result<()> {
    let Command::Init { dir } = &cli.command else {
        unreachable!()
    };
    scaffold(dir)?;
    println!("initialized realm at {}", dir.display());
    println!("  AGENTS.md               agent instructions for this realm");
    println!("  .legend/taxonomy.yaml   canonical disciplines + epics");
    println!("  .legend/landmarks.yaml  landmark registry (auto-extraction on)");
    println!("  Landmarks/              drop Landmark_Name.md files here");
    println!("  Nodes/                  flat entity database");
    println!("next: cd into the realm and run `legend index sync` to (re)build indexes");
    let mut realm = Realm::open(dir)
        .with_context(|| format!("failed to open the fresh realm at `{}`", dir.display()))?;
    let clean = index_sync(&mut realm)?;
    if !clean {
        std::process::exit(1);
    }
    Ok(())
}

fn update(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Update {
        id,
        status,
        cascade,
        json,
    } = &cli.command
    else {
        unreachable!()
    };
    let Some(status_str) = status else {
        bail!("missing required `--status` argument");
    };
    if status_str == "blocked" {
        bail!(
            "`blocked` is a computed runtime state and cannot be set; resolve or vanquish its prerequisites instead"
        );
    }
    let new_status = status_str.parse::<Status>().map_err(anyhow::Error::msg)?;
    let out = ops::update(
        realm,
        UpdateArgs {
            id: id.clone(),
            status: new_status,
            cascade: *cascade,
        },
    )
    .map_err(fail)?;
    report(realm, &out, *json, id)
}

fn rename(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Rename { id, title, json } = &cli.command else {
        unreachable!()
    };
    let Some(new_title) = title else {
        bail!("missing required `--title` argument");
    };
    let out = ops::rename(realm, id, new_title).map_err(fail)?;
    report(realm, &out, *json, id)
}

fn wrap(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Wrap { id, title, json } = &cli.command else {
        unreachable!()
    };
    let out = ops::wrap(realm, id, title.clone()).map_err(fail)?;
    let new_id = out.created[0].clone();
    report(realm, &out, *json, &new_id)
}

fn reparent(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Reparent { id, parent, json } = &cli.command else {
        unreachable!()
    };
    let out = ops::reparent(realm, id, parent.clone()).map_err(fail)?;
    report(realm, &out, *json, id)
}

fn delete(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Delete { id, recursive } = &cli.command else {
        unreachable!()
    };
    let out = ops::delete(realm, id, *recursive).map_err(fail)?;
    println!(
        "deleted {} ({} node(s)); stripped {} dangling blocked_by reference(s)",
        id,
        out.deleted.len(),
        out.references_stripped.len()
    );
    Ok(())
}

/// Recompute and rewrite both index artifacts. Returns `Ok(true)` when the
/// realm validates cleanly and `Ok(false)` when validation errors remain
/// (so the caller can exit with status 1 after surfacing suggestions).
fn index_sync(realm: &mut Realm) -> Result<bool> {
    for w in &realm.warnings {
        eprintln!("warning: {w}");
    }
    let issues = ops::index_sync(realm)?;
    println!(
        "indexed {} node(s) → {}",
        realm.nodes.len(),
        realm.legend_dir.join("index.json").display()
    );
    if issues.is_empty() {
        println!("validation: OK");
        return Ok(true);
    }
    println!("validation: {} node(s) with issues", issues.len());
    for (id, suggestions) in &issues {
        println!(
            "  {id}: {}",
            realm.computed[id].validation_error.as_deref().unwrap_or("")
        );
        for suggestion in suggestions {
            println!("    fix: {suggestion}");
        }
    }
    Ok(false)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Command::Init { .. } => init(&cli),
        Command::Rename { .. } => {
            let mut realm = open_realm(&cli)?;
            rename(&mut realm, &cli)
        }
        Command::Query { .. } => {
            let realm = open_realm(&cli)?;
            query(&realm, &cli)
        }
        Command::Show { .. } => {
            let realm = open_realm(&cli)?;
            show(&realm, &cli)
        }
        Command::Tree { .. } => {
            let realm = open_realm(&cli)?;
            tree(&realm, &cli)
        }
        Command::Create { .. } => {
            let mut realm = open_realm(&cli)?;
            create(&mut realm, &cli)
        }
        Command::Update { .. } => {
            let mut realm = open_realm(&cli)?;
            update(&mut realm, &cli)
        }
        Command::Wrap { .. } => {
            let mut realm = open_realm(&cli)?;
            wrap(&mut realm, &cli)
        }
        Command::Reparent { .. } => {
            let mut realm = open_realm(&cli)?;
            reparent(&mut realm, &cli)
        }
        Command::Delete { .. } => {
            let mut realm = open_realm(&cli)?;
            delete(&mut realm, &cli)
        }
        Command::Index {
            action: IndexAction::Sync,
        } => {
            let mut realm = open_realm(&cli)?;
            let clean = index_sync(&mut realm)?;
            if !clean {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
