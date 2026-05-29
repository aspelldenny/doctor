//! verify-setup — static wiring-integrity check for the Giám sát (boundary-check) role.
//!
//! Q-D5 (WORKFLOW_V2.3 retro §2/§6). Answers "is the role WIRED, not just PRESENT?"
//! verify TỒN-TẠI ≠ verify HOẠT-ĐỘNG: a repo can have every boundary-check file on disk
//! yet have the merge-gate chain dead at the root (doc-rotate: Giám sát 0/7 phiếu).
//!
//! Checklist DERIVED FROM tarot wiring (the only repo where Giám sát runs end-to-end,
//! proven by a live canary) — NOT invented in a vacuum. Each joint traces to "tarot has this":
//!   J1 sentinel-contract — emit marker (agent/command) == grep marker (merge hook), byte-exact
//!   J2 rubric-source     — agent handbook enumerates INV rubric (OR command injects INVARIANTS)
//!   J4 invariants-file   — docs/security/INVARIANTS.md exists (the cited source-of-record)
//!   J5 merge-gate        — block-unsafe-merge.sh exists + registered in settings.json + gates `gh pr merge`
//!   J6 verdict-contract  — agent emits `^Verdict:`/APPROVE == merge hook greps `^Verdict:`/APPROVE
//!
//! NHÃN CỨNG: kiểm NỐI, KHÔNG kiểm TỐT. STATIC only — read file + regex + string-compare.
//! NO spawn, NO judgment of INVARIANTS *content* (that is the runtime canary's job, Q-D7).
//!
//! Soundness pins (adversarial-critic-verified against tarot, the running reference — a check
//! that flags the proven-running repo DORMANT is BLIND, not strict):
//!   - J2 accepts inline-agent-rubric OR command-inject (tarot inlines; do NOT mandate inject).
//!   - J5 "wired" = settings.json PreToolUse registration edge (tarot wires via Claude Code, NOT a git hook).
//!   - Deferred to a hardening pass (would false-flag tarot or need arch-aware extraction):
//!       J7 surface-pattern agreement (tarot blocking-command vs advisory-command differ),
//!       J9 spawn-name == agent name (tarot carries a stale `@agent-security-reviewer` ref).
//!
//! Exit codes:
//!   0 — CONNECTED (all joints wired) OR role not configured (no apparatus — nothing to verify)
//!   1 — DORMANT (≥1 joint broken/absent)
//!   2 — repo path unreadable (bubbles as Err)

use anyhow::{Result, anyhow};
use clap::Args as ClapArgs;
use regex::Regex;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::cli::RunOutput;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to repo root (defaults to cwd).
    #[arg(long)]
    pub repo: Option<PathBuf>,
}

#[derive(PartialEq)]
enum Status {
    Wired,
    Broken,
    Absent,
}

struct Joint {
    id: &'static str,
    detail: String,
    status: Status,
}

/// Read the first existing candidate path; return (path, content). None if none exist/readable.
fn read_first(repo: &Path, candidates: &[&str]) -> Option<(PathBuf, String)> {
    for c in candidates {
        let p = repo.join(c);
        if p.exists() {
            if let Ok(content) = read_to_string(&p) {
                return Some((p, content));
            }
        }
    }
    None
}

/// Extract the security-review START sentinel token AS WRITTEN (case + separator preserved).
/// Matches both `<!-- SECURITY_REVIEW_START -->` and `<!-- security-review-start -->`;
/// the capture group keeps the original spelling so callers can byte-compare emit vs grep.
fn extract_sentinel(content: &str) -> Option<String> {
    let re = Regex::new(r"(?i)<!--\s*(security[_-]review[_-]start)\s*-->").unwrap();
    re.captures(content).map(|c| c[1].to_string())
}

pub fn execute(args: Args) -> Result<RunOutput> {
    let mut out = RunOutput::default();
    let repo = args.repo.unwrap_or_else(|| PathBuf::from("."));

    if !repo.exists() {
        return Err(anyhow!("repo path does not exist: {:?}", repo));
    }

    let agent = read_first(
        &repo,
        &[".claude/agents/boundary-check.md", "agents/boundary-check.md"],
    );
    let command = read_first(
        &repo,
        &[
            ".claude/commands/security-review.md",
            "commands/security-review.md",
        ],
    );
    let hook_path = repo.join("scripts/block-unsafe-merge.sh");
    let hook = if hook_path.exists() {
        read_to_string(&hook_path).ok()
    } else {
        None
    };
    let invariants_path = repo.join("docs/security/INVARIANTS.md");
    let settings = read_first(
        &repo,
        &[".claude/settings.json", ".claude/settings.local.json"],
    );

    // No apparatus at all → the boundary-check role simply isn't configured here.
    // A repo that doesn't use the role is NOT dormant — exit 0, nothing to verify.
    if agent.is_none() && command.is_none() && hook.is_none() {
        out.stdout.push_str(
            "boundary-check role: NOT CONFIGURED (no agent / command / merge-hook) — nothing to verify\n",
        );
        return Ok(out);
    }

    let mut joints: Vec<Joint> = Vec::new();

    // ---- J1 sentinel-contract: emit (agent/command) == grep (hook), byte-exact ----
    {
        let emit = agent
            .as_ref()
            .and_then(|(_, c)| extract_sentinel(c))
            .or_else(|| command.as_ref().and_then(|(_, c)| extract_sentinel(c)));
        let grep = hook.as_ref().and_then(|c| extract_sentinel(c));
        let (status, detail) = match (emit, grep) {
            (Some(e), Some(g)) if e == g => (Status::Wired, format!("emit == grep '{e}'")),
            (Some(e), Some(g)) => (
                Status::Broken,
                format!("emit '{e}' != grep '{g}' (case/separator mismatch)"),
            ),
            (None, _) => (
                Status::Absent,
                "no security-review sentinel emitted in agent/command".to_string(),
            ),
            (_, None) => (
                Status::Absent,
                "no security-review sentinel grep in merge hook".to_string(),
            ),
        };
        joints.push(Joint {
            id: "J1 sentinel-contract",
            detail,
            status,
        });
    }

    // ---- J2 rubric-source: agent inlines INV rubric OR command injects INVARIANTS ----
    {
        let inv_re = Regex::new(r"(?im)\bINV-(?:LOCAL|\d)").unwrap();
        let agent_has = agent
            .as_ref()
            .map(|(_, c)| inv_re.find_iter(c).count() >= 3)
            .unwrap_or(false);
        let cmd_injects = command
            .as_ref()
            .map(|(_, c)| {
                c.contains("INVARIANTS.md")
                    && (c.contains("Read ") || c.contains("cat ") || c.to_lowercase().contains("inject"))
            })
            .unwrap_or(false);
        let (status, detail) = if agent_has {
            (
                Status::Wired,
                "agent handbook enumerates INV rubric inline".to_string(),
            )
        } else if cmd_injects {
            (
                Status::Wired,
                "command injects INVARIANTS into spawn prompt".to_string(),
            )
        } else {
            (
                Status::Absent,
                "no INV rubric inline in agent and no INVARIANTS inject in command".to_string(),
            )
        };
        joints.push(Joint {
            id: "J2 rubric-source",
            detail,
            status,
        });
    }

    // ---- J4 invariants-file: docs/security/INVARIANTS.md exists ----
    {
        let (status, detail) = if invariants_path.exists() {
            (
                Status::Wired,
                "docs/security/INVARIANTS.md present".to_string(),
            )
        } else {
            (
                Status::Absent,
                "docs/security/INVARIANTS.md missing (agent cites it as source-of-record)".to_string(),
            )
        };
        joints.push(Joint {
            id: "J4 invariants-file",
            detail,
            status,
        });
    }

    // ---- J5 merge-gate: hook exists + registered in settings.json + gates `gh pr merge` ----
    {
        let hook_present = hook.is_some();
        let registered = settings
            .as_ref()
            .map(|(_, c)| c.contains("block-unsafe-merge.sh") && c.contains("PreToolUse"))
            .unwrap_or(false);
        let gates_merge = hook
            .as_ref()
            .map(|c| c.contains("gh pr merge") && c.contains("exit 2"))
            .unwrap_or(false);
        let (status, detail) = if hook_present && registered && gates_merge {
            (
                Status::Wired,
                "block-unsafe-merge.sh present + registered in settings.json + gates `gh pr merge`"
                    .to_string(),
            )
        } else if !hook_present {
            (
                Status::Absent,
                "scripts/block-unsafe-merge.sh missing".to_string(),
            )
        } else if !registered {
            (
                Status::Broken,
                "block-unsafe-merge.sh present but NOT registered in .claude/settings.json PreToolUse"
                    .to_string(),
            )
        } else {
            (
                Status::Broken,
                "block-unsafe-merge.sh present but does not gate `gh pr merge` (no exit-2 block)"
                    .to_string(),
            )
        };
        joints.push(Joint {
            id: "J5 merge-gate",
            detail,
            status,
        });
    }

    // ---- J6 verdict-contract: agent emits `^Verdict:`/APPROVE == hook greps it ----
    {
        let verdict_re = Regex::new(r"(?im)^\s*Verdict:").unwrap();
        let emit_ok = agent
            .as_ref()
            .map(|(_, c)| verdict_re.is_match(c) && c.contains("APPROVE"))
            .unwrap_or(false);
        let grep_ok = hook
            .as_ref()
            .map(|c| c.contains("Verdict:") && c.contains("APPROVE"))
            .unwrap_or(false);
        let (status, detail) = if emit_ok && grep_ok {
            (
                Status::Wired,
                "agent emits `Verdict:`/APPROVE == hook parses `^Verdict:`/APPROVE".to_string(),
            )
        } else if !emit_ok {
            (
                Status::Broken,
                "agent does not emit a `Verdict:` line with APPROVE (hook can never pass)".to_string(),
            )
        } else {
            (
                Status::Broken,
                "merge hook does not parse a `Verdict:`/APPROVE line".to_string(),
            )
        };
        joints.push(Joint {
            id: "J6 verdict-contract",
            detail,
            status,
        });
    }

    // ---- aggregate: CONNECTED iff every joint wired ----
    let broken: Vec<&Joint> = joints.iter().filter(|j| j.status != Status::Wired).collect();

    for j in &joints {
        let mark = match j.status {
            Status::Wired => "WIRED ",
            Status::Broken => "BROKEN",
            Status::Absent => "ABSENT",
        };
        let line = format!("  [{}] {} — {}\n", mark, j.id, j.detail);
        if j.status == Status::Wired {
            out.stdout.push_str(&line);
        } else {
            out.stderr.push_str(&line);
        }
    }

    if broken.is_empty() {
        out.stdout
            .push_str("boundary-check: CONNECTED — all wiring joints intact\n");
    } else {
        let reasons: Vec<String> = broken.iter().map(|j| j.detail.clone()).collect();
        out.stderr.push_str(&format!(
            "boundary-check: DORMANT — {}\n",
            reasons.join("; ")
        ));
        out.exit_code = 1;
    }

    Ok(out)
}

/// Thin CLI wrapper — calls execute(), prints output, maps exit_code → process exit.
pub fn run(args: Args) -> Result<()> {
    let out = execute(args)?;
    if !out.stdout.is_empty() {
        print!("{}", out.stdout);
    }
    if !out.stderr.is_empty() {
        eprint!("{}", out.stderr);
    }
    if out.exit_code != 0 {
        std::process::exit(out.exit_code as i32);
    }
    Ok(())
}
