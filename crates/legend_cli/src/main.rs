//! `legend` — the agent-first CLI over a Legendary realm (spec §6.4).
//!
//! Every write/mutation command synchronously rewrites `.legend/index.json`
//! and `.legend/INDEX.md` before returning exit status 0.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use legend_core::dag;
use legend_core::id::{generate_unique, slugify_title};
use legend_core::model::{utc_now_rfc3339, Node, NodeFile, NodeKind, Priority, Status};
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

fn kind_prefix_body(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Card => "\n## System Context & Architectural Design\n",
        NodeKind::Action => "\n## Action Goal\n",
        NodeKind::Guard => "\n## Quality Check Criteria\n",
        NodeKind::Idea => "\n## Idea Concept & Mechanics Exploration\n",
    }
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

fn commit(realm: &mut Realm) -> Result<()> {
    realm.recompute();
    realm.sync_index()?;
    Ok(())
}

fn write_nodes(realm: &mut Realm, ids: &[String]) -> Result<()> {
    for id in ids {
        realm.write_node(id)?;
    }
    Ok(())
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
    if let Some(qp) = quest_points {
        if !legend_core::model::is_valid_quest_points(*qp) {
            bail!("quest_points must be Fibonacci: 1, 2, 3, 5, 8, 13, 21");
        }
    }
    if let Some(p) = parent {
        if !realm.nodes.contains_key(p) {
            bail!("unknown parent `{p}`");
        }
    }
    let now = utc_now_rfc3339();
    let node = Node {
        id: String::new(),
        kind,
        title: title.clone(),
        status: Status::Unstarted,
        priority,
        disciplines: vec![],
        epics: vec![],
        quest_points: *quest_points,
        landmark: None,
        parent: parent.clone(),
        blocked_by: vec![],
        created_at: Some(now.clone()),
        completed_at: None,
        tags: vec![],
    };
    let mut file = NodeFile {
        node,
        body: kind_prefix_body(kind).to_string(),
    };
    let id = generate_unique(kind, |candidate| realm.nodes.contains_key(candidate));
    file.node.id = id.to_string();
    let created = realm.create_node_file(file, &slugify_title(title))?;
    commit(realm)?;
    println!("created {} at {}", id, created.display());
    if *json {
        println!(
            "{}",
            serde_json::to_string_pretty(&node_json(realm, &id.to_string()).unwrap())?
        );
    }
    Ok(())
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
    let Some(node) = realm.nodes.get(id).map(|f| f.node.clone()) else {
        bail!("no such node `{id}`");
    };

    let mut changed = vec![id.clone()];
    if node.kind == NodeKind::Card && new_status == Status::Vanquished {
        let remaining = dag::count_unvanquished_leaves(&realm.node_map(), id);
        if remaining > 0 && !*cascade {
            bail!(
                "Cannot vanquish Card {id}. {remaining} child tasks remaining. Use --cascade to force complete."
            );
        }
        let targets: Vec<String> = if *cascade {
            dag::cascade_ids(&realm.node_map(), id)
        } else {
            vec![]
        };
        for t in &targets {
            let f = realm.nodes.get_mut(t).unwrap();
            f.node.status = Status::Vanquished;
            f.node.completed_at = Some(utc_now_rfc3339());
        }
        changed.extend(targets);
        let f = realm.nodes.get_mut(id).unwrap();
        f.node.status = Status::Vanquished;
        f.node.completed_at = Some(utc_now_rfc3339());
    } else {
        let f = realm.nodes.get_mut(id).unwrap();
        f.node.status = new_status;
        if new_status == Status::Vanquished {
            f.node.completed_at = Some(utc_now_rfc3339());
        } else {
            f.node.completed_at = None;
        }
    }
    changed.sort();
    changed.dedup();
    write_nodes(realm, &changed)?;
    commit(realm)?;
    println!("updated {id} → {status_str}");
    if *json {
        println!(
            "{}",
            serde_json::to_string_pretty(&node_json(realm, id).unwrap())?
        );
    }
    Ok(())
}

fn rename(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Rename { id, title, json } = &cli.command else {
        unreachable!()
    };
    let Some(new_title) = title else {
        bail!("missing required `--title` argument");
    };
    if !realm.nodes.contains_key(id) {
        bail!("no such node `{id}`");
    }
    let outcome = realm.rename_node(id, new_title)?;
    commit(realm)?;
    if outcome.title_changed || outcome.file_renamed {
        let mut parts: Vec<&str> = Vec::new();
        if outcome.title_changed {
            parts.push("title");
        }
        if outcome.file_renamed {
            parts.push("file slug");
        }
        println!("renamed {id} ({}): {}", parts.join(" + "), outcome.new_name);
    } else {
        println!("no change for {id} (title and slug already match)");
    }
    if *json {
        println!(
            "{}",
            serde_json::to_string_pretty(&node_json(realm, id).unwrap())?
        );
    }
    Ok(())
}

fn wrap(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Wrap { id, title, json } = &cli.command else {
        unreachable!()
    };
    let Some(target) = realm.nodes.get(id).map(|f| f.node.clone()) else {
        bail!("no such node `{id}`");
    };
    if target.kind == NodeKind::Card {
        bail!("`{id}` is already a Card; wrap only applies to Action, Guard or Idea nodes");
    }
    let card_title = title
        .clone()
        .unwrap_or_else(|| format!("{} (Group)", target.title));
    let now = utc_now_rfc3339();
    let card = Node {
        id: String::new(),
        kind: NodeKind::Card,
        title: card_title.clone(),
        status: Status::Unstarted,
        priority: target.priority,
        disciplines: target.disciplines.clone(),
        epics: target.epics.clone(),
        quest_points: None,
        landmark: target.landmark.clone(),
        parent: target.parent.clone(),
        blocked_by: vec![],
        created_at: Some(now),
        completed_at: None,
        tags: vec![],
    };
    let mut card_file = NodeFile {
        node: card,
        body: kind_prefix_body(NodeKind::Card).to_string(),
    };
    let new_id = generate_unique(NodeKind::Card, |candidate| {
        realm.nodes.contains_key(candidate)
    });
    card_file.node.id = new_id.to_string();
    let path = realm.create_node_file(card_file, &slugify_title(&card_title))?;
    // Reparent target under the new card.
    {
        let f = realm.nodes.get_mut(id).unwrap();
        f.node.parent = Some(new_id.to_string());
    }
    write_nodes(realm, std::slice::from_ref(id))?;
    commit(realm)?;
    println!(
        "wrapped {} under new card {} → {}",
        id,
        new_id,
        path.display()
    );
    if *json {
        println!(
            "{}",
            serde_json::to_string_pretty(&node_json(realm, &new_id.to_string()).unwrap())?
        );
    }
    Ok(())
}

fn reparent(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Reparent { id, parent, json } = &cli.command else {
        unreachable!()
    };
    if !realm.nodes.contains_key(id) {
        bail!("no such node `{id}`");
    }
    if let Some(p) = parent {
        if p == id {
            bail!("a node cannot be its own parent");
        }
        if !realm.nodes.contains_key(p) {
            bail!("unknown parent `{p}`");
        }
        let extra = vec![(id.clone(), p.clone())];
        if dag::would_create_cycle(&realm.node_map(), &extra) {
            bail!("CycleDetectedError: reparenting {id} under {p} would close a cycle");
        }
    }
    let f = realm.nodes.get_mut(id).unwrap();
    f.node.parent = parent.clone();
    write_nodes(realm, std::slice::from_ref(id))?;
    commit(realm)?;
    println!(
        "reparented {id} → {}",
        parent.as_deref().unwrap_or("root (parent: null)")
    );
    if *json {
        println!(
            "{}",
            serde_json::to_string_pretty(&node_json(realm, id).unwrap())?
        );
    }
    Ok(())
}

fn delete(realm: &mut Realm, cli: &Cli) -> Result<()> {
    let Command::Delete { id, recursive } = &cli.command else {
        unreachable!()
    };
    if !realm.nodes.contains_key(id) {
        bail!("no such node `{id}`");
    }
    let descendants = dag::descendants_ids(&realm.node_map(), id);
    if !descendants.is_empty() && !*recursive {
        bail!(
            "node {id} has {} descendant(s). Use --recursive to delete the whole subgraph.",
            descendants.len()
        );
    }
    let mut doomed = vec![id.clone()];
    doomed.extend(descendants);
    let affected = realm.delete_ids(&doomed);
    write_nodes(realm, &affected)?;
    commit(realm)?;
    println!(
        "deleted {} ({} node(s)); stripped {} dangling blocked_by reference(s)",
        id,
        doomed.len(),
        affected.len()
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
    realm.recompute();
    realm.sync_index()?;
    println!(
        "indexed {} node(s) → {}",
        realm.nodes.len(),
        realm.legend_dir.join("index.json").display()
    );
    let sorted = realm.sorted_ids();
    let errors: Vec<String> = sorted
        .iter()
        .filter(|i| realm.computed[*i].validation_error.is_some())
        .cloned()
        .collect();
    if errors.is_empty() {
        println!("validation: OK");
        return Ok(true);
    }
    println!("validation: {} node(s) with issues", errors.len());
    for id in errors {
        println!(
            "  {id}: {}",
            realm.computed[&id]
                .validation_error
                .as_deref()
                .unwrap_or("")
        );
        for suggestion in realm.suggestions_for(&id) {
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
