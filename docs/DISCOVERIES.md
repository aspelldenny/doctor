# Discoveries Log — doctor

Phiếu discovery reports — newest entry on top.

Soft cap: 1000 dòng → warn (rotate to `docs/Archive/DISCOVERIES_ARCHIVE.md`).
Hard cap: 1500 dòng → block commit.

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
