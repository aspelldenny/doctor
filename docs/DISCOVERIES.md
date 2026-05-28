# Discoveries Log — doctor

Phiếu discovery reports — newest entry on top.

Soft cap: 1000 dòng → warn (rotate to `docs/Archive/DISCOVERIES_ARCHIVE.md`).
Hard cap: 1500 dòng → block commit.

---

## 2026-05-28 — P005 MCP serve shipped

`doctor serve` MCP stdio JSON-RPC 2.0 server shipped via rmcp 1.7.0. 4 tools: lane_check, validate_map, rotate_check, runtime_scan. Refactored 4 subcmds to execute/run split (RunOutput struct in cli/mod.rs). DoctorServer unit struct, sync tool fns, #[tool_router] + #[tool_handler] two-step. Manual smoke: tools/list returns 4 tools confirmed. 29/29 tests pass (27 regression + 2 smoke). **MVP sprint COMPLETE — 5/5 phiếu shipped.** See `docs/discoveries/P005.md`.

---

## 2026-05-28 — P004 runtime-scan shipped

`doctor runtime-scan` implemented: 9 regex patterns across 5 vendor whitelist (GitHub classic/OAuth/user/server/fine-grained PAT, AWS access key, AWS secret key, Anthropic sk-ant-*, OpenAI legacy sk-*). Scan targets: `.git/config`, `.mcp.json`, `.env*` (repo root); `~/.ssh/config`/`.gitconfig`/`.netrc` opt-in via `--include-home`. Output sanitized (`path:line: pattern_name`, no secret content). Exit 0 clean, exit 1 leak. 8 integration tests pass. src/main.rs not touched (already wired). sk-proj- format deferred Python pilot vòng 2 per SOUL line 3. See `docs/discoveries/P004.md`.

---

## 2026-05-28 — P003 rotate-check shipped

`doctor rotate-check` implemented: `toml::from_str` parse of `.sos-stack.toml` `[rotate]` (files/soft/hard, per-field fallback to defaults 1000/1500). File missing → skip (no drift). Worst severity wins: exit 0 clean, 1 warn, 2 block. 7 integration tests pass. Toml parse error → exit 2. src/main.rs not touched (already wired). See `docs/discoveries/P003.md`.

---

## 2026-05-28 — P002 validate-map shipped

`doctor validate-map` implemented: serde-typed AgentMap parse + 5-category walker (edit/read_shallow/read_deep/research_gate/contract_test) + Path::exists + anchor-grep flexible match (markdown heading + Rust symbol, case-insensitive + hyphen↔space tolerant). Glob entries (`**`) skipped. 7 integration tests pass. Exit codes: 0 clean, 1 drift, 2 yaml error. See `docs/discoveries/P002.md`.

---

## 2026-05-28 — P001 lane-check shipped

`doctor lane-check` implemented: regex lane-parse + 3-metric counter (lines/anchors/constraints) + Normal/Fast/Guarded budget gate. 8 integration tests pass. Edge case: Debate Log numbered tables add to anchor count (PARTIAL oracle, documented). See `docs/discoveries/P001.md`.

---

## 2026-05-28 — Skeleton bootstrap

Doctor repo bootstrapped from `~/sos-kit/INSTALL.md` v2.2 install path.

Skeleton includes:
- 5 agent handbooks copied từ ~/sos-kit/agents/ (v2.2 doctrine — cụm A propagated)
- skills symlink → ~/sos-kit/skills/ (13 skill)
- scripts: architect-guard.sh, session-start-banner.sh (sos-kit) + block-unsafe-merge.sh, block-env-edit.sh (tarot)
- hooks/pre-commit (sos-kit)
- configs: .docs-gate.toml (v2.2 comment), .ship.toml (rust template)
- docs: PROJECT.md, SOUL.md (3 hard lines), BACKLOG.md (5 phiếu active), ARCHITECTURE.md, INVARIANTS.md
- src/: clap dispatch + 4 subcmd stubs với todo!() placeholder

Cargo.toml deps: clap 4 derive, serde + serde_yaml + serde_json, chrono, tokio rt, anyhow, thiserror 2, tempfile, regex, walkdir, rmcp 1.7.0 với schemars feature.

Build: `cargo build` exit 0 (no implementation, no tests).

Next: session mới ở ~/doctor sẽ build P001-P005 qua v2.2 workflow.
