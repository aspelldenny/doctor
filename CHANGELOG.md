# Changelog

All notable changes to `doctor` are documented here.
Format: Keep a Changelog (https://keepachangelog.com/en/1.0.0/).

---

## [Unreleased]

### Added

- P001 — `doctor lane-check` subcmd: parse Lane field from phiếu markdown header, count total lines / anchor rows / numbered constraints, gate against Normal (≤250 lines, ≤5 anchors, ≤5 constraints) and Fast (≤100 lines) budgets. Guarded passes unconditionally. Exit codes: 0 OK, 1 budget exceeded, 2 missing Lane field.
