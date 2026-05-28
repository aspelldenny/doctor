//! runtime-scan — Sub-mech F runtime state token leak.
//!
//! Implementation deferred to phiếu P004.
//!
//! Spec từ WORKFLOW_V2.2.md §7 (Sub-mech F row):
//! - Scan: .git/config, ~/.ssh/config, .env*, .mcp.json
//! - Patterns: ghp_*, gho_*, ghu_*, ghs_*, github_pat_*, AWS access key prefix, etc.
//! - Exit 0 if clean, exit 1 if leak found.
//!
//! Mã phiếu xác chết: P305 tarot — .git/config token plaintext local + VPS (instance #10).
//! INV-009 source-clean nhưng runtime DIRTY. Source code passing mechanical gate KHÔNG
//! guarantee runtime state clean.

use anyhow::Result;
use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to repo root (defaults to cwd).
    #[arg(long)]
    pub repo: Option<PathBuf>,

    /// Also scan home directory state (~/.ssh, ~/.gitconfig).
    #[arg(long)]
    pub include_home: bool,
}

pub fn run(_args: Args) -> Result<()> {
    todo!("phiếu P004 — runtime secret scan");
}
