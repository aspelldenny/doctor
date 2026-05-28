//! doctor — v2.2 mechanical gate enforcement.
//!
//! Doctrine source: ~/sos-kit/docs/WORKFLOW_V2.2.md
//! Retro trace: ~/sos-kit/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md
//!
//! MVP subcmd (each fixes a lỗi-đã-thấy with mã phiếu xác chết):
//!   lane-check    — §1 lane budget (P003 643 dòng precedent)
//!   validate-map  — §4 AGENT_MAP path/anchor (DISCOVERIES tarot drift precedent)
//!   rotate-check  — §6 dòng cap (DISCOVERIES tarot 265KB over-cap precedent)
//!   runtime-scan  — Sub-mech F (P305 token plaintext precedent)
//!
//! Cảnh báo: doctor là thước đo MECHANICAL SOUND, không phải cái não.
//! Lane-check đếm dòng. Validate-map check path. Rotate-check đếm dòng. Runtime-scan grep token.
//! Đừng để nó ôm logic judgment.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod cli;

#[derive(Parser)]
#[command(name = "doctor", version, about = "v2.2 mechanical gate enforcement")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// §1 lane budget — đếm dòng phiếu + anchor + constraint vs lane field.
    LaneCheck(cli::lane_check::Args),

    /// §4 AGENT_MAP path/anchor exist check (SOUND only).
    ValidateMap(cli::validate_map::Args),

    /// §6 dòng cap DISCOVERIES/CHANGELOG. Soft 1000 → warn. Hard 1500 → block.
    RotateCheck(cli::rotate_check::Args),

    /// Sub-mech F runtime token leak scan (.git/config, ~/.ssh, .env*, .mcp.json).
    RuntimeScan(cli::runtime_scan::Args),

    /// MCP serve mode — exposes subcmds as tools via JSON-RPC 2.0.
    Serve,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::LaneCheck(args) => cli::lane_check::run(args),
        Commands::ValidateMap(args) => cli::validate_map::run(args),
        Commands::RotateCheck(args) => cli::rotate_check::run(args),
        Commands::RuntimeScan(args) => cli::runtime_scan::run(args),
        Commands::Serve => {
            eprintln!("MCP serve mode — phiếu P005 (deferred)");
            std::process::exit(2);
        }
    }
}
