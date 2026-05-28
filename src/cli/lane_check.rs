//! lane-check — §1 lane budget enforcement.
//!
//! Reads a phiếu markdown file, extracts the Lane field, counts 3 metrics
//! (total lines, Task-0 anchor rows, numbered constraints under "## Luật chơi"),
//! and exits per budget:
//!   0 — within budget (or Guarded)
//!   1 — budget exceeded (stderr reason)
//!   2 — Lane field missing from phiếu header

use anyhow::{anyhow, Result};
use clap::Args as ClapArgs;
use regex::Regex;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to phiếu markdown file.
    #[arg(long)]
    pub ticket: PathBuf,
}

pub fn run(args: Args) -> Result<()> {
    let content = std::fs::read_to_string(&args.ticket)
        .map_err(|e| anyhow!("cannot read ticket {:?}: {}", args.ticket, e))?;

    // --- Task 1: extract lane field ---
    // Match both `> **Lane:** Normal` (blockquote header) and `**Lane:** Normal` (plain).
    let lane_re = Regex::new(r"(?i)\*\*Lane:\*\*\s*(Normal|Guarded|Fast)").unwrap();
    let lane = match lane_re.captures(&content) {
        Some(caps) => caps[1].to_string(),
        None => {
            eprintln!("error: ticket missing Lane field (expected **Lane:** Normal|Guarded|Fast)");
            std::process::exit(2);
        }
    };

    // Guarded: no cap — return immediately.
    if lane.eq_ignore_ascii_case("Guarded") {
        println!("OK (Guarded — no cap)");
        return Ok(());
    }

    // --- Task 2: count 3 metrics ---

    // 1. Total lines
    let total_lines = content.lines().count();

    // 2. Anchor rows — Task 0 table rows starting with `| <digit(s)> |`
    let anchor_re = Regex::new(r"(?m)^\|\s*\d+\s*\|").unwrap();
    let anchors = anchor_re.find_iter(&content).count();

    // 3. Numbered constraints under "## Luật chơi" heading
    let constraints = count_constraints(&content);

    // --- Task 3: gate per lane budget ---
    if lane.eq_ignore_ascii_case("Normal") {
        let mut violations: Vec<String> = Vec::new();
        if total_lines > 250 {
            violations.push(format!("Normal lane: {} lines > 250 cap", total_lines));
        }
        if anchors > 5 {
            violations.push(format!("Normal lane: {} anchors > 5 cap", anchors));
        }
        if constraints > 5 {
            violations.push(format!("Normal lane: {} constraints > 5 cap", constraints));
        }
        if !violations.is_empty() {
            for v in &violations {
                eprintln!("{}", v);
            }
            std::process::exit(1);
        }
        println!(
            "OK (Normal — {} lines, {} anchors, {} constraints)",
            total_lines, anchors, constraints
        );
    } else if lane.eq_ignore_ascii_case("Fast") {
        if total_lines > 100 {
            eprintln!("Fast lane: {} lines > 100 cap", total_lines);
            std::process::exit(1);
        }
        println!("OK (Fast — {} lines)", total_lines);
    }

    Ok(())
}

/// Count numbered-list items (`^\d+\. `) under the "## Luật chơi" heading,
/// stopping at the next `##`-level heading or end-of-file.
fn count_constraints(content: &str) -> usize {
    let heading_re = Regex::new(r"(?mi)^##\s+Luật chơi").unwrap();
    let next_heading_re = Regex::new(r"(?m)^##\s").unwrap();
    let item_re = Regex::new(r"(?m)^\d+\.\s").unwrap();

    let start = match heading_re.find(content) {
        Some(m) => m.end(),
        None => return 0,
    };

    let slice = &content[start..];
    let end = match next_heading_re.find(slice) {
        Some(m) => m.start(),
        None => slice.len(),
    };

    item_re.find_iter(&slice[..end]).count()
}
