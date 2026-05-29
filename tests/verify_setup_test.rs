//! Integration tests for `doctor verify-setup` subcmd (Q-D5, WORKFLOW_V2.3).
//!
//! Pattern: assert_cmd + tempfile workspace (mirror of rotate_check_test.rs).
//! Each test builds a temp repo of the boundary-check apparatus and asserts the
//! CONNECTED/DORMANT verdict. These fixtures are a microcosm of the real
//! discrimination test (tarot = connected; doc-rotate = sentinel-mismatch + no INVARIANTS).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn doctor() -> Command {
    Command::cargo_bin("doctor").expect("doctor binary")
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, content).expect("write file");
}

/// Build a fully-wired (CONNECTED) boundary-check apparatus under `base`.
/// `sentinel` is the emit marker spelling used by the agent + command (the hook always
/// greps UPPERCASE `SECURITY_REVIEW_START`, mirroring the real merge hook).
fn write_connected(base: &Path, emit_sentinel: &str) {
    // Agent handbook: inline INV rubric + verdict line + emit sentinel.
    write(
        &base.join(".claude/agents/boundary-check.md"),
        &format!(
            "---\nname: boundary-check\n---\n\
             # Giám sát\n\
             INV-101 ownership. INV-102 env. INV-103 webhook. INV-104 idor. INV-105 credit.\n\
             Verdict: APPROVE | NEEDS_REVIEW (>=1 FLAG)\n\
             Emit:\n<!-- {emit} -->\nVerdict: APPROVE\n<!-- {end} -->\n",
            emit = emit_sentinel,
            end = emit_sentinel.replace("start", "end").replace("START", "END"),
        ),
    );
    // Command: names INV source.
    write(
        &base.join(".claude/commands/security-review.md"),
        "# /security-review\nSpawn boundary-check on INV-101 -> INV-107.\nSource: docs/security/INVARIANTS.md\n",
    );
    // Merge hook: greps UPPERCASE sentinel + Verdict APPROVE, gates gh pr merge, exit 2.
    write(
        &base.join("scripts/block-unsafe-merge.sh"),
        "#!/usr/bin/env bash\n\
         # PreToolUse hook — block gh pr merge <N> until APPROVE.\n\
         grep -q '<!-- SECURITY_REVIEW_START -->' <<< \"$COMMENTS\"\n\
         VERDICT_LINE=$(grep -E '^Verdict:' <<< \"$COMMENTS\")\n\
         echo \"$VERDICT_LINE\" | grep -q 'APPROVE'\n\
         if echo \"$CMD\" | grep -qE 'gh pr merge'; then :; fi\n\
         exit 2\n",
    );
    // INVARIANTS source-of-record.
    write(
        &base.join("docs/security/INVARIANTS.md"),
        "### INV-101 — env\n### INV-102 — webhook\n### INV-103 — idor\n",
    );
    // settings.json registers the hook under PreToolUse(Bash).
    write(
        &base.join(".claude/settings.json"),
        "{\n  \"hooks\": {\n    \"PreToolUse\": [\n      { \"matcher\": \"Bash\", \"hooks\": [ { \"type\": \"command\", \"command\": \"bash scripts/block-unsafe-merge.sh\" } ] }\n    ]\n  }\n}\n",
    );
}

// ---------------------------------------------------------------------------
// Test 1: connected_all_joints — full apparatus, matching sentinels → exit 0.
// Mirrors tarot.
// ---------------------------------------------------------------------------
#[test]
fn connected_all_joints() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();
    write_connected(base, "SECURITY_REVIEW_START");

    doctor()
        .args(["verify-setup", "--repo", base.to_str().unwrap()])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("CONNECTED"));
}

// ---------------------------------------------------------------------------
// Test 2: sentinel_mismatch_dormant — agent emits lowercase-hyphen, hook greps
// UPPERCASE → J1 BROKEN → exit 1. Mirrors doc-rotate fracture #1.
// ---------------------------------------------------------------------------
#[test]
fn sentinel_mismatch_dormant() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();
    write_connected(base, "security-review-start");

    doctor()
        .args(["verify-setup", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("DORMANT"))
        .stderr(predicate::str::contains("J1 sentinel-contract"))
        .stderr(predicate::str::contains("case/separator"));
}

// ---------------------------------------------------------------------------
// Test 3: missing_invariants_dormant — fully wired but no docs/security/INVARIANTS.md
// → J4 ABSENT → exit 1. Mirrors doc-rotate fracture #4.
// ---------------------------------------------------------------------------
#[test]
fn missing_invariants_dormant() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();
    write_connected(base, "SECURITY_REVIEW_START");
    fs::remove_file(base.join("docs/security/INVARIANTS.md")).expect("rm invariants");

    doctor()
        .args(["verify-setup", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("DORMANT"))
        .stderr(predicate::str::contains("J4 invariants-file"));
}

// ---------------------------------------------------------------------------
// Test 4: hook_present_but_unregistered_dormant — hook on disk but not in
// settings.json → J5 BROKEN (registration edge missing) → exit 1.
// This is the "dead script" dormancy the critic's J8 guards against.
// ---------------------------------------------------------------------------
#[test]
fn hook_present_but_unregistered_dormant() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();
    write_connected(base, "SECURITY_REVIEW_START");
    // Wipe the registration edge — script still on disk.
    write(&base.join(".claude/settings.json"), "{ \"hooks\": {} }\n");

    doctor()
        .args(["verify-setup", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("J5 merge-gate"))
        .stderr(predicate::str::contains("NOT registered"));
}

// ---------------------------------------------------------------------------
// Test 5: not_configured_clean — empty repo, no apparatus → exit 0 (not dormant,
// the role just isn't used here). Soundness: don't scream DORMANT at repos that
// don't run the Giám sát role.
// ---------------------------------------------------------------------------
#[test]
fn not_configured_clean() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    doctor()
        .args(["verify-setup", "--repo", base.to_str().unwrap()])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("NOT CONFIGURED"));
}
