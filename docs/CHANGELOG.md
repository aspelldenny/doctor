# CHANGELOG — doctor

---

## P005 — MCP serve mode (2026-05-28)

`doctor serve` MCP stdio JSON-RPC 2.0 server implemented via rmcp 1.7.0. Exposes 4 tools: `lane_check`, `validate_map`, `rotate_check`, `runtime_scan`. Refactored all 4 CLI subcmds to `pub fn execute() -> Result<RunOutput>` + thin `run()` wrapper (zero regression — 27/27 existing tests pass). `src/mcp/mod.rs`: `DoctorServer` struct with `#[tool_router]` + `#[tool_handler]` two-step pattern; sync tool fns; `serve()` async entry point using `tokio::runtime::Builder::new_current_thread()` in main.rs Serve arm. 2 smoke tests: `tools/list` 4-tool verify + `lane_check` tools/call happy path. `cargo install --path .` → `~/.cargo/bin/doctor`. MCP wire: `{"mcpServers":{"doctor":{"command":"/Users/nguyenhuuanh/.cargo/bin/doctor","args":["serve"]}}}` in `~/.mcp.json`. **MVP sprint COMPLETE — 5/5 phiếu shipped (P001-P005), 4 CLI + 1 MCP, 29 tests pass.**

---

## P004 — runtime-scan subcmd (2026-05-28)

`doctor runtime-scan` implemented: Sub-mech F token leak detection (5 vendor whitelist, 9 regex patterns). Scans `.git/config`, `.mcp.json`, `.env*` (repo root) always; `~/.ssh/config`, `~/.gitconfig`, `~/.netrc` with `--include-home` opt-in. Output sanitized to `path:line: pattern_name` (matched content not printed). Exit 0 clean, exit 1 leak found. 8 integration tests pass.

---

## P003 — rotate-check subcmd (2026-05-28)

`doctor rotate-check` implemented: toml::from_str parse of `.sos-stack.toml` `[rotate]` (files/soft/hard, per-field fallback to defaults soft=1000/hard=1500). File missing → skip. Worst severity wins: exit 0 clean, 1 warn, 2 block. 7 integration tests pass.

---

## P002 — validate-map subcmd (2026-05-28)

`doctor validate-map` implemented: serde-typed AgentMap parse + 5-category walker + Path::exists + anchor-grep (markdown heading + Rust symbol, case-insensitive + hyphen↔space tolerant). Glob entries skipped. 7 integration tests pass. Exit 0 clean, 1 drift, 2 yaml error.

---

## P001 — lane-check subcmd (2026-05-28)

`doctor lane-check` implemented: regex lane-parse + 3-metric counter (lines/anchors/constraints) + Normal/Fast/Guarded budget gate. 5 integration tests pass. Edge case: Debate Log numbered tables add to anchor count (PARTIAL oracle, documented).

---

## Skeleton bootstrap (2026-05-28)

Doctor repo bootstrapped — 4 subcmd stubs, Cargo.toml deps, build exit 0.
