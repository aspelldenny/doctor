# Changelog

All notable changes to `doctor` are documented here.
Format: Keep a Changelog (https://keepachangelog.com/en/1.0.0/).

---

## [Unreleased]

### Added

- P001 — `doctor lane-check` subcmd: parse Lane field from phiếu markdown header, count total lines / anchor rows / numbered constraints, gate against Normal (≤250 lines, ≤5 anchors, ≤5 constraints) and Fast (≤100 lines) budgets. Guarded passes unconditionally. Exit codes: 0 OK, 1 budget exceeded, 2 missing Lane field.
- P002 — `doctor validate-map` subcmd: serde-typed parse of AGENT_MAP.yaml, walk 5 path categories (edit/read_shallow/read_deep/research_gate/contract_test) across all surfaces, check Path::exists + anchor-grep flexible match (markdown heading and Rust symbol, case-insensitive + hyphen↔space tolerant). Glob entries (`**`) skipped. Exit codes: 0 clean, 1 drift (PATH_MISSING / ANCHOR_MISSING on stderr), 2 yaml parse error.
- P003 — `doctor rotate-check` subcmd: line-count cap gate per §6. Loads `.sos-stack.toml` `[rotate]` table (files/soft/hard) with per-field fallback to defaults (files=["docs/DISCOVERIES.md","docs/CHANGELOG.md"], soft=1000, hard=1500). File missing → skip (no drift). Worst severity wins across all files. Exit codes: 0 all under soft, 1 ≥1 file warn (soft ≤ lines < hard), 2 ≥1 file block (lines ≥ hard) or toml parse error. SOUND only — no entry classification, no rotate suggestions.
