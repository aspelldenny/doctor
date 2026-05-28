//! Integration tests for `doctor rotate-check` subcmd (phiếu P003).
//!
//! Pattern: assert_cmd + tempfile workspace (mirror of validate_map_test.rs from P002).
//! Each test builds a temporary directory, writes fixture files, and runs doctor.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn doctor() -> Command {
    Command::cargo_bin("doctor").expect("doctor binary")
}

/// Helper: write N lines of "x" content to a file path (including parent dirs).
fn write_lines(path: &std::path::Path, n: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, "x\n".repeat(n)).expect("write lines");
}

// ---------------------------------------------------------------------------
// Test 1: no_config_no_files_clean
// Empty workspace — no docs/, no .sos-stack.toml.
// Default files list: DISCOVERIES.md + CHANGELOG.md, both missing → skip → exit 0.
// ---------------------------------------------------------------------------
#[test]
fn no_config_no_files_clean() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path().to_str().unwrap();

    doctor()
        .args(["rotate-check", "--repo", repo])
        .assert()
        .success()
        .code(0);
}

// ---------------------------------------------------------------------------
// Test 2: default_clean_under_soft
// DISCOVERIES.md with 100 lines (well under soft cap 1000) → exit 0.
// CHANGELOG.md absent → skip.
// ---------------------------------------------------------------------------
#[test]
fn default_clean_under_soft() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    write_lines(&base.join("docs/DISCOVERIES.md"), 100);

    doctor()
        .args(["rotate-check", "--repo", base.to_str().unwrap()])
        .assert()
        .success()
        .code(0);
}

// ---------------------------------------------------------------------------
// Test 3: default_warn_soft_cap
// DISCOVERIES.md with exactly 1000 lines → soft ≤ lines < hard → exit 1.
// stderr should contain "WARN" and "soft cap 1000".
// ---------------------------------------------------------------------------
#[test]
fn default_warn_soft_cap() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    write_lines(&base.join("docs/DISCOVERIES.md"), 1000);

    doctor()
        .args(["rotate-check", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("WARN"))
        .stderr(predicate::str::contains("soft cap 1000"));
}

// ---------------------------------------------------------------------------
// Test 4: default_block_hard_cap
// DISCOVERIES.md with 1500 lines → lines ≥ hard → exit 2.
// stderr should contain "BLOCK" and "hard cap 1500".
// ---------------------------------------------------------------------------
#[test]
fn default_block_hard_cap() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    write_lines(&base.join("docs/DISCOVERIES.md"), 1500);

    doctor()
        .args(["rotate-check", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("BLOCK"))
        .stderr(predicate::str::contains("hard cap 1500"));
}

// ---------------------------------------------------------------------------
// Test 5: mixed_worst_wins
// DISCOVERIES.md at 1000 lines (warn) + CHANGELOG.md at 1500 lines (block).
// Worst severity wins → exit 2.
// ---------------------------------------------------------------------------
#[test]
fn mixed_worst_wins() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    write_lines(&base.join("docs/DISCOVERIES.md"), 1000);
    write_lines(&base.join("docs/CHANGELOG.md"), 1500);

    doctor()
        .args(["rotate-check", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("BLOCK"))
        .stderr(predicate::str::contains("WARN"));
}

// ---------------------------------------------------------------------------
// Test 6: custom_config_override
// .sos-stack.toml sets [rotate] soft=10 hard=20. File with 15 lines → exit 1.
// ---------------------------------------------------------------------------
#[test]
fn custom_config_override() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    fs::write(
        base.join(".sos-stack.toml"),
        "[rotate]\nsoft = 10\nhard = 20\n",
    )
    .expect("write toml");

    // Use default file list: docs/DISCOVERIES.md
    write_lines(&base.join("docs/DISCOVERIES.md"), 15);

    doctor()
        .args(["rotate-check", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("WARN"))
        .stderr(predicate::str::contains("soft cap 10"));
}

// ---------------------------------------------------------------------------
// Test 7: invalid_toml_parse_error
// .sos-stack.toml with unparseable content → toml parse error → exit 2.
// ---------------------------------------------------------------------------
#[test]
fn invalid_toml_parse_error() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    fs::write(base.join(".sos-stack.toml"), "not valid toml [[[\n").expect("write toml");

    doctor()
        .args(["rotate-check", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(2);
}
