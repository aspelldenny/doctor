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

// ---------------------------------------------------------------------------
// Test 6 (positive): shim_hook_connected — B+3 fail-closed shim (sos-kit P064).
// Hook is a thin shim that execs `claude-hooks block-unsafe-merge`; the sentinel
// grep and Verdict parse live in the binary, invisible to static grep of hook file.
// Agent still emits the sentinel and `Verdict:`/APPROVE (agent-side contracts).
// J1 + J6 must recognise the delegation → CONNECTED rc=0.
// ---------------------------------------------------------------------------
#[test]
fn shim_hook_connected() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // Agent: emits sentinel + Verdict/APPROVE + inline INV rubric.
    write(
        &base.join(".claude/agents/boundary-check.md"),
        "---\nname: boundary-check\n---\n\
         # Giám sát\n\
         INV-101 ownership. INV-102 env. INV-103 webhook. INV-104 idor. INV-105 credit.\n\
         Verdict: APPROVE | NEEDS_REVIEW (>=1 FLAG)\n\
         Emit:\n<!-- SECURITY_REVIEW_START -->\nVerdict: APPROVE\n<!-- SECURITY_REVIEW_END -->\n",
    );
    // Hook is a B+3 shim — delegates sentinel-grep + Verdict-parse to the binary.
    // Must still contain `gh pr merge` and `exit 2` so J5 stays WIRED.
    // The shim_re matches `claude-hooks block-unsafe-merge` (with whitespace between).
    write(
        &base.join("scripts/block-unsafe-merge.sh"),
        "#!/usr/bin/env bash\n\
         # B+3 fail-closed shim — delegates to claude-hooks binary (sos-kit P064).\n\
         # gh pr merge is gated via exit 2 from the binary.\n\
         exec claude-hooks block-unsafe-merge \"$@\"\n\
         # Fallback: if binary unavailable, fail-closed.\n\
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

    doctor()
        .args(["verify-setup", "--repo", base.to_str().unwrap()])
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("CONNECTED"))
        .stdout(predicate::str::contains("B+3 shim"));
}

// ---------------------------------------------------------------------------
// Test 7 (negative): shim_hook_but_agent_missing_verdict — shim hook present
// (J1 delegates OK) but agent does NOT emit a `Verdict:` line with APPROVE.
// J6 must remain BROKEN — shim-arm MUST NOT swallow agent-side failures.
// ---------------------------------------------------------------------------
#[test]
fn shim_hook_but_agent_missing_verdict() {
    let dir: TempDir = tempfile::tempdir().expect("tempdir");
    let base = dir.path();

    // Agent: emits sentinel but NO Verdict/APPROVE line.
    write(
        &base.join(".claude/agents/boundary-check.md"),
        "---\nname: boundary-check\n---\n\
         # Giám sát\n\
         INV-101 ownership. INV-102 env. INV-103 webhook. INV-104 idor. INV-105 credit.\n\
         Emit:\n<!-- SECURITY_REVIEW_START -->\n<!-- SECURITY_REVIEW_END -->\n",
    );
    // Same B+3 shim as test 6.
    write(
        &base.join("scripts/block-unsafe-merge.sh"),
        "#!/usr/bin/env bash\n\
         # B+3 fail-closed shim — delegates to claude-hooks binary (sos-kit P064).\n\
         exec claude-hooks block-unsafe-merge \"$@\"\n\
         # gh pr merge not allowed without APPROVE\n\
         exit 2\n",
    );
    write(
        &base.join("docs/security/INVARIANTS.md"),
        "### INV-101 — env\n### INV-102 — webhook\n",
    );
    write(
        &base.join(".claude/settings.json"),
        "{\n  \"hooks\": {\n    \"PreToolUse\": [\n      { \"matcher\": \"Bash\", \"hooks\": [ { \"type\": \"command\", \"command\": \"bash scripts/block-unsafe-merge.sh\" } ] }\n    ]\n  }\n}\n",
    );

    doctor()
        .args(["verify-setup", "--repo", base.to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("DORMANT"))
        .stderr(predicate::str::contains("J6 verdict-contract"));
}
