//! runtime-scan — Sub-mech F runtime state token leak.
//!
//! Scans local config files for hardcoded secrets matching 5 vendor patterns.
//! Exit 0 = clean. Exit 1 = ≥1 leak found.
//! No exit 2 — no config parse (scan is always valid, files may be absent).
//!
//! Target files (always):
//!   .git/config, .mcp.json, .env* (repo root, non-recursive)
//!
//! Target files (--include-home only):
//!   ~/.ssh/config, ~/.gitconfig, ~/.netrc
//!
//! Patterns (9 regex spanning 5 vendors):
//!   GitHub classic (ghp_, gho_, ghu_, ghs_), GitHub fine-grained PAT,
//!   AWS access key (AKIA prefix), AWS secret key (env var assignment),
//!   OpenAI legacy sk-* (sk-proj- deferred to Python pilot vòng 2 per SOUL line 3),
//!   Anthropic sk-ant-*
//!
//! Output: <path>:<line>: <pattern_name> — matched content NOT printed (sanitize).
//!
//! Doctrine: WORKFLOW_V2.2.md §7 Sub-mech F.
//! Mã phiếu xác chết: P305 tarot — .git/config token plaintext local + VPS (instance #10).

use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use regex::Regex;
use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};

use crate::cli::RunOutput;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to repo root (defaults to cwd).
    #[arg(long)]
    pub repo: Option<PathBuf>,

    /// Also scan home directory state (~/.ssh/config, ~/.gitconfig, ~/.netrc).
    /// Default false — scanning home is privacy-sensitive, must be explicit opt-in.
    #[arg(long)]
    pub include_home: bool,
}

// ---------------------------------------------------------------------------
// Vendor patterns — 5 vendors, 9 regex literals
// Name + regex source pairs. Compiled once in compile_patterns().
// ---------------------------------------------------------------------------

const PATTERNS: &[(&str, &str)] = &[
    ("github_classic",          r"ghp_[A-Za-z0-9]{36,}"),
    ("github_oauth",            r"gho_[A-Za-z0-9]{36,}"),
    ("github_user",             r"ghu_[A-Za-z0-9]{36,}"),
    ("github_server",           r"ghs_[A-Za-z0-9]{36,}"),
    ("github_pat_fine_grained", r"github_pat_[A-Za-z0-9_]{82,}"),
    ("aws_access_key",          r"AKIA[0-9A-Z]{16}"),
    // AWS secret key: env-var assignment form (case-insensitive key name)
    ("aws_secret_key",          r"(?i)aws_secret_access_key\s*=\s*[A-Za-z0-9/+=]{40}"),
    // Anthropic check BEFORE OpenAI — sk-ant- overlaps sk- prefix.
    // Checking anthropic_key first means a sk-ant- token reports as anthropic_key, not openai_key.
    ("anthropic_key",           r"sk-ant-[A-Za-z0-9_-]{32,}"),
    // OpenAI legacy classic format only. sk-proj-* deferred to Python pilot vòng 2 (SOUL line 3).
    // sk-[A-Za-z0-9]{32,} does NOT match sk-proj- because dash breaks [A-Za-z0-9] quantifier.
    ("openai_key",              r"sk-[A-Za-z0-9]{32,}"),
];

// ---------------------------------------------------------------------------
// Pattern compilation
// ---------------------------------------------------------------------------

/// Compile all PATTERNS entries into (name, Regex) pairs.
/// Pattern compile failure bubbles as anyhow error (should not happen — patterns are literals).
fn compile_patterns() -> Result<Vec<(&'static str, Regex)>> {
    PATTERNS
        .iter()
        .map(|(name, src)| {
            let re = Regex::new(src)
                .with_context(|| format!("compile regex for pattern '{name}': {src}"))?;
            Ok((*name, re))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Target file collection
// ---------------------------------------------------------------------------

/// Build list of files to scan.
/// File existence is checked later in the scan loop — collect unconditionally here.
fn collect_targets(repo_root: &Path, include_home: bool) -> Result<Vec<PathBuf>> {
    let mut targets: Vec<PathBuf> = Vec::new();

    // Always-scan repo files
    targets.push(repo_root.join(".git/config"));
    targets.push(repo_root.join(".mcp.json"));

    // .env* files at repo root (non-recursive)
    match read_dir(repo_root) {
        Ok(entries) => {
            for entry_result in entries {
                match entry_result {
                    Ok(entry) => {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with(".env") {
                            match entry.file_type() {
                                Ok(ft) if ft.is_file() => targets.push(entry.path()),
                                Ok(_) => {} // dir or symlink-to-dir — skip
                                Err(e) => {
                                    // Non-fatal: warn and skip this entry
                                    eprintln!("warning: cannot stat {:?}: {}", entry.path(), e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Non-fatal: warn and skip this entry
                        eprintln!("warning: read_dir entry error: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!("cannot read repo root {:?}: {}", repo_root, e));
        }
    }

    // Home directory files (opt-in only — privacy sensitive)
    if include_home {
        let home = std::env::var("HOME").context("HOME env required for --include-home")?;
        let home_path = PathBuf::from(&home);
        targets.push(home_path.join(".ssh/config"));
        targets.push(home_path.join(".gitconfig"));
        targets.push(home_path.join(".netrc"));
    }

    Ok(targets)
}

// ---------------------------------------------------------------------------
// execute() — returns RunOutput; IO errors bubble as Err
// ---------------------------------------------------------------------------

pub fn execute(args: Args) -> Result<RunOutput> {
    let mut out = RunOutput::default();

    let repo_root = args.repo.unwrap_or_else(|| PathBuf::from("."));
    let patterns = compile_patterns()?;
    let targets = collect_targets(&repo_root, args.include_home)?;

    let mut leaks: Vec<String> = Vec::new();

    for target in &targets {
        // File missing = OK skip, no drift (matches P003 file-missing-skip rule)
        if !target.exists() {
            continue;
        }

        let content = read_to_string(target)
            .with_context(|| format!("read {}", target.display()))?;

        for (idx, line) in content.lines().enumerate() {
            for (name, re) in &patterns {
                if re.is_match(line) {
                    // Output: path:line: pattern_name ONLY.
                    // Matched content is NOT printed — sanitize constraint #3.
                    leaks.push(format!("{}:{}: {}", target.display(), idx + 1, name));
                }
            }
        }
    }

    if leaks.is_empty() {
        return Ok(out); // exit 0 — clean
    }

    for leak in &leaks {
        out.stderr.push_str(leak);
        out.stderr.push('\n');
    }
    out.exit_code = 1;

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
