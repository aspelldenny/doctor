//! Integration tests for `doctor lane-check` subcmd (phiếu P001).
//!
//! Follows assert_cmd + tempfile pattern from advisory-inbox precedent.
//! Fixtures are inline raw strings written to NamedTempFile.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn tmp_ticket(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(content.as_bytes()).expect("write");
    f
}

fn doctor() -> Command {
    Command::cargo_bin("doctor").expect("doctor binary")
}

/// Normal lane, all under budget → exit 0.
#[test]
fn ok_normal_under_budget() {
    // 30 lines total, 3 anchors, 3 constraints
    let mut s = "> **Lane:** Normal\n\n## Task 0\n\n".to_string();
    for i in 1..=3 { s.push_str(&format!("| {} | anchor |\n", i)); }
    s.push_str("\n## Luật chơi\n\n");
    for i in 1..=3 { s.push_str(&format!("{}. item\n", i)); }
    for _ in 0..20 { s.push_str("line\n"); }
    let f = tmp_ticket(&s);
    doctor().arg("lane-check").arg("--ticket").arg(f.path())
        .assert().success().stdout(predicate::str::contains("OK (Normal"));
}

/// Normal lane, 280 lines → exit 1, stderr "lines > 250".
#[test]
fn fail_normal_lines_over() {
    let mut s = "> **Lane:** Normal\n\n## Body\n\n".to_string();
    for _ in 0..276 { s.push_str("padding\n"); }
    let f = tmp_ticket(&s);
    doctor().arg("lane-check").arg("--ticket").arg(f.path())
        .assert().failure().code(1).stderr(predicate::str::contains("lines > 250"));
}

/// Normal lane, 6 anchor rows → exit 1, stderr "anchors > 5".
#[test]
fn fail_normal_anchors_over() {
    let mut s = "> **Lane:** Normal\n\n## Task 0\n\n".to_string();
    for i in 1..=6 { s.push_str(&format!("| {} | anchor |\n", i)); }
    for _ in 0..140 { s.push_str("line\n"); }
    let f = tmp_ticket(&s);
    doctor().arg("lane-check").arg("--ticket").arg(f.path())
        .assert().failure().code(1).stderr(predicate::str::contains("anchors > 5"));
}

/// Missing Lane field → exit 2.
#[test]
fn fail_missing_lane() {
    let f = tmp_ticket("# PHIẾU\n\nNo lane here.\n\n## Luật chơi\n\n1. x\n");
    doctor().arg("lane-check").arg("--ticket").arg(f.path())
        .assert().failure().code(2).stderr(predicate::str::contains("missing Lane field"));
}

/// Guarded lane, 600 lines → exit 0 (no cap).
#[test]
fn ok_guarded_huge() {
    let mut s = "> **Lane:** Guarded\n\n".to_string();
    for _ in 0..600 { s.push_str("line\n"); }
    let f = tmp_ticket(&s);
    doctor().arg("lane-check").arg("--ticket").arg(f.path())
        .assert().success().stdout(predicate::str::contains("OK (Guarded"));
}

// Note: self-referential P001 boundary test was planned but removed.
// The phiếu's Debate Log table contains 5 additional numbered rows that match
// the anchor pattern, yielding 10 total > 5 cap. Working per spec — documented
// in docs/discoveries/P001.md as edge case (Debate Log tables add to count).
// P006 scope fix addresses this: counter now scoped to ## Task 0 section only.

/// P006 scope fix: Task 0 table (4 rows) + Debate Log table (4 rows) = 8 total
/// raw matches, but scoped counter must return 4 (Task 0 only) → exit 0.
#[test]
fn ok_normal_task0_scoped_ignores_debate_log_table() {
    let mut s = "> **Lane:** Normal\n\n".to_string();
    s.push_str("## Task 0\n\n");
    for i in 1..=4 { s.push_str(&format!("| {} | anchor |\n", i)); }
    s.push_str("\n## Debate Log\n\n");
    for i in 1..=4 { s.push_str(&format!("| {} | turn result |\n", i)); }
    s.push_str("\n## Luật chơi\n\n");
    for i in 1..=3 { s.push_str(&format!("{}. item\n", i)); }
    for _ in 0..20 { s.push_str("line\n"); }
    let f = tmp_ticket(&s);
    doctor().arg("lane-check").arg("--ticket").arg(f.path())
        .assert().success().stdout(predicate::str::contains("OK (Normal"));
}

/// P006 edge case: phiếu has NO ## Task 0 heading; numbered rows exist elsewhere.
/// Anchor count must be 0 (not error) → exit 0.
#[test]
fn ok_no_task0_heading_treats_as_zero() {
    let mut s = "> **Lane:** Normal\n\n".to_string();
    s.push_str("## Context\n\n");
    for i in 1..=4 { s.push_str(&format!("| {} | some table |\n", i)); }
    s.push_str("\n## Luật chơi\n\n");
    for i in 1..=3 { s.push_str(&format!("{}. item\n", i)); }
    for _ in 0..20 { s.push_str("line\n"); }
    let f = tmp_ticket(&s);
    doctor().arg("lane-check").arg("--ticket").arg(f.path())
        .assert().success().stdout(predicate::str::contains("OK (Normal"));
}

/// P006 regression: P001 real phiếu has Task 0 (5 rows) + Debate Log recap (5 rows).
/// Pre-fix: counter saw 10 → exit 1. Post-fix: scoped to 5 → exit 0.
#[test]
fn ok_p001_real_phieu_passes() {
    let p001_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/ticket/P001-lane-check.md");
    doctor()
        .arg("lane-check")
        .arg("--ticket")
        .arg(&p001_path)
        .assert()
        .success();
}
