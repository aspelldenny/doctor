//! rotate-check — §6 dòng cap DISCOVERIES/CHANGELOG.
//!
//! Loads optional `.sos-stack.toml` [rotate] section for config (files/soft/hard).
//! Falls back to defaults: files=["docs/DISCOVERIES.md","docs/CHANGELOG.md"],
//! soft=1000, hard=1500.
//!
//! File-missing rule: skip (no drift). File exists → count lines → compare cap.
//! Worst severity wins across all files.
//!
//! Exit codes:
//!   0 — all files under soft cap (or no files exist)
//!   1 — ≥1 file soft ≤ lines < hard (WARN)
//!   2 — ≥1 file lines ≥ hard (BLOCK), or .sos-stack.toml parse error
//!
//! SOUND only — no entry classification, no rotate suggestions.
//! Rotate-archive judgment delegated to Python pilot vòng 2.
//!
//! Doctrine: WORKFLOW_V2.2.md §6.
//! Mã phiếu xác chết: DISCOVERIES tarot 265KB = ~5300 dòng = 5× over hard cap.

use anyhow::{Result, anyhow};
use clap::Args as ClapArgs;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::cli::RunOutput;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to repo root (defaults to cwd).
    #[arg(long)]
    pub repo: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Config structs — parsed from .sos-stack.toml [rotate]
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SosStack {
    #[serde(default)]
    rotate: Option<RotateConfig>,
}

#[derive(Debug, Deserialize)]
struct RotateConfig {
    #[serde(default)]
    files: Option<Vec<String>>,
    #[serde(default)]
    soft: Option<usize>,
    #[serde(default)]
    hard: Option<usize>,
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

const DEFAULT_FILES: &[&str] = &["docs/DISCOVERIES.md", "docs/CHANGELOG.md"];
const DEFAULT_SOFT: usize = 1000;
const DEFAULT_HARD: usize = 1500;

// ---------------------------------------------------------------------------
// execute() — returns RunOutput; IO errors bubble as Err
// ---------------------------------------------------------------------------

pub fn execute(args: Args) -> Result<RunOutput> {
    let mut out = RunOutput::default();

    let repo_root = args.repo.unwrap_or_else(|| PathBuf::from("."));
    let (files, soft, hard) = resolve_config(&repo_root, &mut out)?;

    // resolve_config sets exit_code=2 on toml parse error and returns Ok(out) for
    // the early-return path; if exit_code is already set, return immediately.
    if out.exit_code != 0 {
        return Ok(out);
    }

    let mut worst: u8 = 0;
    let mut report: Vec<String> = Vec::new();

    for rel in &files {
        let full = repo_root.join(rel);
        if !full.exists() {
            // File missing = OK skip, no drift (file not yet created is valid)
            continue;
        }

        let content = read_to_string(&full)
            .map_err(|e| anyhow!("cannot read {:?}: {}", full, e))?;
        let lines = content.lines().count();

        if lines >= hard {
            worst = worst.max(2);
            report.push(format!(
                "BLOCK {}: {} lines (hard cap {})",
                rel, lines, hard
            ));
        } else if lines >= soft {
            worst = worst.max(1);
            report.push(format!(
                "WARN  {}: {} lines (soft cap {})",
                rel, lines, soft
            ));
        }
    }

    for line in &report {
        out.stderr.push_str(line);
        out.stderr.push('\n');
    }

    out.exit_code = worst;
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

// ---------------------------------------------------------------------------
// Config resolution
// ---------------------------------------------------------------------------

/// Load `.sos-stack.toml` from repo root if present, parse [rotate] table.
/// Missing file or missing [rotate] section → all defaults.
/// Per-field fallback (not all-or-nothing).
/// On toml parse error: sets out.exit_code=2 + out.stderr message, returns Ok(early).
fn resolve_config(
    repo_root: &Path,
    out: &mut RunOutput,
) -> Result<(Vec<String>, usize, usize)> {
    let toml_path = repo_root.join(".sos-stack.toml");

    if !toml_path.exists() {
        return Ok(default_tuple());
    }

    let content = read_to_string(&toml_path)
        .map_err(|e| anyhow!("cannot read .sos-stack.toml: {}", e))?;

    let stack: SosStack = match toml::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            out.stderr.push_str(&format!(
                "error: toml parse error in .sos-stack.toml: {}\n",
                e
            ));
            out.exit_code = 2;
            return Ok(default_tuple());
        }
    };

    let rotate = match stack.rotate {
        None => return Ok(default_tuple()),
        Some(r) => r,
    };

    let files = rotate
        .files
        .unwrap_or_else(|| DEFAULT_FILES.iter().map(|s| s.to_string()).collect());
    let soft = rotate.soft.unwrap_or(DEFAULT_SOFT);
    let hard = rotate.hard.unwrap_or(DEFAULT_HARD);

    Ok((files, soft, hard))
}

fn default_tuple() -> (Vec<String>, usize, usize) {
    (
        DEFAULT_FILES.iter().map(|s| s.to_string()).collect(),
        DEFAULT_SOFT,
        DEFAULT_HARD,
    )
}
