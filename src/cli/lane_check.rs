//! lane-check — §1 lane budget enforcement.
//!
//! Implementation deferred to phiếu P001.
//!
//! Spec từ WORKFLOW_V2.2.md §1:
//! - Normal lane: phiếu ≤ 250 dòng, ≤ 5 anchors, ≤ 5 hard constraints.
//! - Guarded lane: full quyền.
//! - Fast lane: ≤ 100 dòng, no architect.
//! - Vượt budget không có cờ override → exit 1.
//!
//! Mã phiếu xác chết: P003 advisory-inbox — 643 dòng cho 6 test, Normal lane labeled
//! nhưng vẫn chạy full DRAFT → CHALLENGE → RESPOND → SURGICAL → EXECUTE.

use anyhow::Result;
use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to phiếu markdown file.
    #[arg(long)]
    pub ticket: PathBuf,
}

pub fn run(_args: Args) -> Result<()> {
    todo!("phiếu P001 — lane budget check");
}
