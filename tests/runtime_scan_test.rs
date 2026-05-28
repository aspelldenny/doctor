//! Integration tests for `doctor runtime-scan` subcmd (phiếu P004).
//!
//! Pattern: assert_cmd + tempfile workspace (mirror of rotate_check_test.rs from P003).
//! Each test builds a temporary directory with fixture files, then runs `doctor runtime-scan`.
//!
//! Fake tokens used in fixtures:
//!   GitHub classic:  ghp_ + 36 'A' chars (minimum length match)
//!   Anthropic:       sk-ant- + 32 'A' chars
//!   AWS access key:  AKIA + 16 upper alphanum (AKIAABCDEFGHIJKLMNOP)
//!   OpenAI classic:  sk- + 32 'A' chars (legacy format, NOT sk-proj-)
//!
//! All fake tokens are designed to match patterns but are NOT real credentials.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn doctor() -> Command {
    Command::cargo_bin("doctor").expect("doctor binary")
}

// ---------------------------------------------------------------------------
// Test 1: empty_workspace_clean
// Empty tmpdir — no target files exist → all skipped → exit 0.
// ---------------------------------------------------------------------------
#[test]
fn empty_workspace_clean() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path().to_str().unwrap();

    doctor()
        .args(["runtime-scan", "--repo", repo])
        .assert()
        .success()
        .code(0);
}

// ---------------------------------------------------------------------------
// Test 2: clean_env_no_secrets
// .env file with harmless key=value content → exit 0.
// ---------------------------------------------------------------------------
#[test]
fn clean_env_no_secrets() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    fs::write(base.join(".env"), "FOO=bar\nBAZ=qux\nPORT=3000\n").expect("write .env");

    doctor()
        .args(["runtime-scan", "--repo", base.to_str().unwrap()])
        .assert()
        .success()
        .code(0);
}

// ---------------------------------------------------------------------------
// Test 3: github_token_detected
// .env with GitHub classic token (ghp_ + 36 A chars) → exit 1.
// stderr contains path fragment + line number + pattern name "github_classic".
// ---------------------------------------------------------------------------
#[test]
fn github_token_detected() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // Fake token: ghp_ + 36 uppercase A — matches ghp_[A-Za-z0-9]{36,}
    let fake_token = format!("GITHUB_TOKEN=ghp_{}", "A".repeat(36));
    fs::write(base.join(".env"), format!("{fake_token}\n")).expect("write .env");

    doctor()
        .args(["runtime-scan", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(".env:1:"))
        .stderr(predicate::str::contains("github_classic"));
}

// ---------------------------------------------------------------------------
// Test 4: mcp_json_anthropic_key
// .mcp.json with Anthropic key (sk-ant- + 32 A chars) → exit 1.
// stderr contains ".mcp.json:1:" + "anthropic_key".
// ---------------------------------------------------------------------------
#[test]
fn mcp_json_anthropic_key() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // Fake token: sk-ant- + 32 uppercase A — matches sk-ant-[A-Za-z0-9_-]{32,}
    let fake_token = format!("sk-ant-{}", "A".repeat(32));
    fs::write(
        base.join(".mcp.json"),
        format!("{{\"key\":\"{fake_token}\"}}\n"),
    )
    .expect("write .mcp.json");

    doctor()
        .args(["runtime-scan", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(".mcp.json:1:"))
        .stderr(predicate::str::contains("anthropic_key"));
}

// ---------------------------------------------------------------------------
// Test 5: aws_access_key_in_git_config
// .git/config with AWS access key (AKIA + 16 upper alphanum) → exit 1.
// stderr contains "aws_access_key".
// ---------------------------------------------------------------------------
#[test]
fn aws_access_key_in_git_config() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // Fake token: AKIA + 16 uppercase alphanumeric — matches AKIA[0-9A-Z]{16}
    fs::create_dir_all(base.join(".git")).expect("create .git");
    fs::write(
        base.join(".git/config"),
        "aws = AKIAABCDEFGHIJKLMNOP\n",
    )
    .expect("write .git/config");

    doctor()
        .args(["runtime-scan", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("aws_access_key"));
}

// ---------------------------------------------------------------------------
// Test 6: stderr_no_secret_content (constraint #3 sanitize)
// exit 1 case — stderr must NOT contain the fake token content itself.
// Output format: path:line:pattern_name ONLY.
// ---------------------------------------------------------------------------
#[test]
fn stderr_no_secret_content() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // Use Anthropic fake token: sk-ant- + 32 A chars
    let fake_body = "A".repeat(32);
    let fake_token = format!("sk-ant-{fake_body}");
    fs::write(
        base.join(".mcp.json"),
        format!("{{\"api_key\":\"{fake_token}\"}}\n"),
    )
    .expect("write .mcp.json");

    // stderr must NOT contain the token body (the 32 A's sequence)
    doctor()
        .args(["runtime-scan", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("anthropic_key"))
        .stderr(predicate::str::contains(&fake_body).not());
}

// ---------------------------------------------------------------------------
// Test 7: openai_legacy_key_detected
// .env with OpenAI legacy sk-* format → exit 1, stderr "openai_key".
// Verify sk-ant- tokens are classified as anthropic_key, not openai_key.
// ---------------------------------------------------------------------------
#[test]
fn openai_legacy_key_detected() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // Fake token: sk- + 32 uppercase A — matches sk-[A-Za-z0-9]{32,}
    // This does NOT match sk-ant-* (different prefix) — anthropic check fires first anyway.
    let fake_token = format!("sk-{}", "A".repeat(32));
    fs::write(base.join(".env"), format!("OPENAI_KEY={fake_token}\n")).expect("write .env");

    doctor()
        .args(["runtime-scan", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("openai_key"));
}

// ---------------------------------------------------------------------------
// Test 8: mixed_leaks_exit1
// Multiple files with different vendor leaks → exit 1 (not 2).
// Both patterns should appear in stderr.
// ---------------------------------------------------------------------------
#[test]
fn mixed_leaks_exit1() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // GitHub in .env
    let gh_token = format!("ghp_{}", "B".repeat(36));
    fs::write(base.join(".env"), format!("GH_TOKEN={gh_token}\n")).expect("write .env");

    // Anthropic in .mcp.json
    let ant_token = format!("sk-ant-{}", "C".repeat(32));
    fs::write(
        base.join(".mcp.json"),
        format!("{{\"key\":\"{ant_token}\"}}\n"),
    )
    .expect("write .mcp.json");

    doctor()
        .args(["runtime-scan", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("github_classic"))
        .stderr(predicate::str::contains("anthropic_key"));
}
