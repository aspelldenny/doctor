//! validate-map — §4 AGENT_MAP path/anchor check.
//!
//! Parses an AGENT_MAP.yaml file (serde typed), walks 5 path categories across
//! all surfaces, and reports PATH_MISSING / ANCHOR_MISSING drift.
//!
//! Exit codes:
//!   0 — clean (no drift)
//!   1 — drift found (list on stderr)
//!   2 — yaml parse error or I/O error
//!
//! Glob entries (`**`) are skipped — path-exists check is undefined for globs.
//! Anchor flexible match: case-insensitive + hyphen↔space tolerant.
//! Covers markdown headings (`^#+\s+`) and Rust symbols (`fn/struct/enum/trait/impl`).
//!
//! Doctrine: WORKFLOW_V2.2.md §4.
//! Mã phiếu xác chết: DISCOVERIES tarot 265KB — anchor pointed at sections no longer present.

use anyhow::{Result, anyhow};
use clap::Args as ClapArgs;
use regex::Regex;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::cli::RunOutput;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to AGENT_MAP.yaml file.
    #[arg(long)]
    pub map: PathBuf,
}

// --------------------------------------------------------------------------
// Typed structs matching AGENT_MAP schema (v1)
// --------------------------------------------------------------------------

#[derive(serde::Deserialize, Default)]
struct AgentMap {
    #[allow(dead_code)]
    version: Option<u32>,
    #[serde(default)]
    surfaces: BTreeMap<String, Surface>,
    #[serde(default)]
    #[allow(dead_code)]
    never_default_read: Vec<String>,
}

#[derive(serde::Deserialize, Default)]
struct Surface {
    #[serde(default)]
    #[allow(dead_code)]
    load_bearing: bool,
    #[serde(default)]
    edit: Vec<String>,
    #[serde(default)]
    read_shallow: Vec<String>,
    #[serde(default)]
    read_deep: Vec<String>,
    #[serde(default)]
    research_gate: Vec<String>,
    #[serde(default)]
    contract_test: Vec<String>,
    // blast, oracle_soundness intentionally omitted — doctor does not check semantics
}

// --------------------------------------------------------------------------
// execute() — returns RunOutput; IO/parse error bubbles as Err
// --------------------------------------------------------------------------

pub fn execute(args: Args) -> Result<RunOutput> {
    let mut out = RunOutput::default();

    let raw = std::fs::read_to_string(&args.map)
        .map_err(|e| anyhow!("cannot read map {:?}: {}", args.map, e))?;

    let map: AgentMap = match serde_yaml::from_str(&raw) {
        Ok(m) => m,
        Err(e) => {
            out.stderr
                .push_str(&format!("error: yaml parse failed: {}\n", e));
            out.exit_code = 2;
            return Ok(out);
        }
    };

    let mut drifts: Vec<String> = Vec::new();

    // Walk all surfaces
    for (surface_name, surface) in &map.surfaces {
        for (category, entries) in surface_categories(surface) {
            for entry in entries {
                check_entry(surface_name, category, entry, &mut drifts);
            }
        }
    }

    // never_default_read is intentionally not path-checked (scope is explicit per
    // phiếu Debate Log SC2 — entries are either globs or files that may not exist
    // in the current repo context).

    if drifts.is_empty() {
        out.stdout.push_str("OK — no drift\n");
    } else {
        drifts.sort();
        for d in &drifts {
            out.stderr.push_str(d);
            out.stderr.push('\n');
        }
        out.exit_code = 1;
    }

    Ok(out)
}

/// Thin CLI wrapper — calls execute(), prints output, maps exit_code → process exit.
pub fn run(args: Args) -> Result<()> {
    let out = execute(args)?;
    if !out.stdout.is_empty() {
        print!("{}", out.stdout);
    }
    if !out.stderr.is_empty() {
        eprint!("{}", out.stderr);
    }
    if out.exit_code != 0 {
        std::process::exit(out.exit_code as i32);
    }
    Ok(())
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

/// Return the 5 path categories as (name, entries) pairs.
fn surface_categories(s: &Surface) -> [(&'static str, &Vec<String>); 5] {
    [
        ("edit", &s.edit),
        ("read_shallow", &s.read_shallow),
        ("read_deep", &s.read_deep),
        ("research_gate", &s.research_gate),
        ("contract_test", &s.contract_test),
    ]
}

/// Check one AGENT_MAP entry (may be `path` or `path#anchor`).
/// Glob entries (`**`) are skipped.
fn check_entry(surface: &str, category: &str, entry: &str, drifts: &mut Vec<String>) {
    // Skip glob entries — path-exists is undefined for glob patterns
    if entry.contains("**") {
        return;
    }

    // Split on first `#` to separate path from anchor
    let (raw_path, anchor_opt) = match entry.find('#') {
        Some(pos) => (&entry[..pos], Some(&entry[pos + 1..])),
        None => (entry, None),
    };

    let path = Path::new(raw_path);

    if !path.exists() {
        drifts.push(format!(
            "{}.{}: {} — PATH_MISSING",
            surface, category, entry
        ));
        return; // No point checking anchor if file is missing
    }

    if let Some(anchor) = anchor_opt {
        if !anchor.is_empty() {
            match anchor_exists(path, anchor) {
                Ok(true) => {}
                Ok(false) => {
                    drifts.push(format!(
                        "{}.{}: {} — ANCHOR_MISSING",
                        surface, category, entry
                    ));
                }
                Err(e) => {
                    drifts.push(format!(
                        "{}.{}: {} — READ_ERROR({})",
                        surface, category, entry, e
                    ));
                }
            }
        }
    }
}

/// Return true if `anchor` exists in the target file.
///
/// Match strategy (flexible):
///   1. Markdown heading: `^#+\s+<text>` — normalize both sides
///      (lowercase, hyphen→space, collapse whitespace).
///   2. Rust symbol: `^(pub )?fn/struct/enum/trait/impl <name>` —
///      compare lowercase (Rust idents use underscores, not hyphens).
///
/// Anchor normalization: lowercase + replace `-` with space + collapse whitespace.
fn anchor_exists(target: &Path, anchor: &str) -> Result<bool> {
    let content = std::fs::read_to_string(target)
        .map_err(|e| anyhow!("cannot read {:?}: {}", target, e))?;

    let normalized_anchor = normalize_anchor(anchor);

    // 1. Markdown heading match
    let heading_re = Regex::new(r"(?m)^#+\s+(.*)$").unwrap();
    for cap in heading_re.captures_iter(&content) {
        let heading_text = normalize_anchor(&cap[1]);
        if heading_text == normalized_anchor {
            return Ok(true);
        }
    }

    // 2. Rust symbol match (fn, struct, enum, trait, impl)
    //    Anchor may use hyphens or spaces as separators — map to lowercase for comparison.
    let symbol_re =
        Regex::new(r"(?m)^(?:pub\s+)?(?:fn|struct|enum|trait|impl)\s+(\w+)").unwrap();
    let anchor_lower = anchor.to_lowercase();
    for cap in symbol_re.captures_iter(&content) {
        if cap[1].to_lowercase() == anchor_lower {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Normalize an anchor or heading text for flexible comparison:
/// lowercase, replace `-` with space, collapse consecutive whitespace.
fn normalize_anchor(s: &str) -> String {
    let lower = s.to_lowercase();
    let spaced = lower.replace('-', " ");
    // Collapse multiple spaces into one and trim
    spaced
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
