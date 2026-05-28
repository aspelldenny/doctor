//! validate-map — §4 AGENT_MAP path/anchor check.
//!
//! Implementation deferred to phiếu P002.
//!
//! Spec từ WORKFLOW_V2.2.md §4:
//! - Check 1: mọi path trong edit/read_shallow/read_deep/research_gate/contract_test exists (test -e).
//! - Check 2: mọi anchor #section còn trong file đích.
//! - SOUND only — KHÔNG check contract_test compile/parse (round 5 Claude Web #3 fix).
//!
//! Mã phiếu xác chết: DISCOVERIES tarot 265KB rule-không-enforce-thì-drift.
//! Map drift còn tệ hơn không map vì nó chủ động nói architect "leaf, đọc nông"
//! trong khi load-bearing, và partial-oracle không vớt.

use anyhow::Result;
use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to AGENT_MAP.yaml file.
    #[arg(long)]
    pub map: PathBuf,
}

pub fn run(_args: Args) -> Result<()> {
    todo!("phiếu P002 — AGENT_MAP path/anchor validate");
}
