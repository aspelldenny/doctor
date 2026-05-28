//! rotate-check — §6 dòng cap DISCOVERIES/CHANGELOG.
//!
//! Implementation deferred to phiếu P003.
//!
//! Spec từ WORKFLOW_V2.2.md §6:
//! - Soft cap 1000 dòng → warn.
//! - Hard cap 1500 dòng → block (force archive trước).
//! - Config trong .sos-stack.toml [rotate] section.
//!
//! IMPORTANT (round 8 Claude Web split): rotate-check làm phần SOUND only — đếm dòng + cảnh báo cap.
//! Phần PARTIAL (phân loại doctrine/operational entry, quyết cắt cái nào, format lạ) là việc
//! của rotate-archive Python tool ở pilot vòng 2. KHÔNG ôm logic judgment vào Rust binary.
//!
//! Mã phiếu xác chết: DISCOVERIES tarot 265KB = ~5300 dòng = 5x over cap.
//! Rule rotate là prose trong CLAUDE.md, KHÔNG enforce → drift.

use anyhow::Result;
use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to repo root (defaults to cwd).
    #[arg(long)]
    pub repo: Option<PathBuf>,
}

pub fn run(_args: Args) -> Result<()> {
    todo!("phiếu P003 — DISCOVERIES/CHANGELOG line count + cap warn/block");
}
