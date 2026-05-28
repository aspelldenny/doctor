# CHANGELOG — doctor

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
