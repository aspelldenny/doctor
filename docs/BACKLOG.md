# BACKLOG — doctor

## Active sprint — post-MVP fix (1 phiếu)

- [ ] **P006** — `doctor lane-check` anchor counter scope fix
  - Bug: anchor counter đếm TẤT CẢ table rows kiểu `| <N> |` trên toàn phiếu, lẫn Debate Log Turn table (5 row tự template) → false positive khi phiếu thực sự ≤ 5 Task 0 anchor.
  - Dogfood evidence: `doctor lane-check --ticket docs/ticket/P001-lane-check.md` → exit 1 "Normal lane: 10 anchors > 5 cap" trong khi P001 có đúng 5 Task 0 anchor.
  - Doctrine: WORKFLOW_V2.2.md §1 — anchor count = Task 0 row only.
  - Fix: scope counter giữa heading `## Task 0` (hoặc fuzzy `Task 0 — Verification Anchors`) → heading `##` tiếp theo. Chỉ đếm table rows trong scope.
  - Edit_allow: `src/cli/lane_check.rs`, `tests/lane_check_test.rs`.
  - Regression: 5 test lane_check + 24 test khác phải pass. Thêm 1 test case fixture phiếu có Debate Log table to assert chỉ đếm Task 0.
  - Lane: Normal (Tầng 2 — 2 file, ≤ 100 LOC).
  - Mã phiếu xác chết: P001 self-eat dogfood gate fired 2026-05-28.

## MVP build — ✅ SHIPPED 2026-05-28 (5 phiếu)

- [x] **P001** — `doctor lane-check` subcmd
  - Doctrine: WORKFLOW_V2.2.md §1 lane budget
  - Input: `--ticket <path>` markdown file
  - Logic: đếm dòng phiếu + anchor table rows + constraint count. So với lane field (Normal ≤ 250 / Guarded full / Fast ≤ 100).
  - Output: exit 0 = OK, exit 1 = budget exceeded + reason, exit 2 = ticket missing lane field
  - Mã phiếu xác chết: P003 advisory-inbox 643 dòng cho 6 test.

- [x] **P002** — `doctor validate-map` subcmd
  - Doctrine: WORKFLOW_V2.2.md §4 AGENT_MAP path/anchor
  - Input: `--map <path>` AGENT_MAP.yaml file
  - Logic: serde_yaml parse + walk all `edit/read_shallow/read_deep/research_gate/contract_test` paths. Check `Path::exists()`. Parse anchor `#section` strings, grep target file cho `^#+\s+<section>`.
  - SOUND only — KHÔNG check contract_test compile/parse (round 5 Claude Web #3 fix).
  - Output: exit 0 = clean, exit 1 = drift (path/anchor missing) với list, exit 2 = yaml parse error.
  - Mã phiếu xác chết: DISCOVERIES tarot 265KB drift.

- [x] **P003** — `doctor rotate-check` subcmd
  - Doctrine: WORKFLOW_V2.2.md §6 dòng cap
  - Input: `--repo <path>` (default cwd)
  - Logic: đếm dòng `docs/DISCOVERIES.md`, `docs/CHANGELOG.md`. So với cap config `.sos-stack.toml [rotate]` (default soft=1000, hard=1500).
  - SOUND only — KHÔNG phân loại entry hay quyết cắt cái nào (đó là rotate-archive Python pilot vòng 2).
  - Output: exit 0 = under cap, exit 1 = soft cap warn (info), exit 2 = hard cap block.
  - Mã phiếu xác chết: DISCOVERIES tarot 265KB = ~5300 dòng = 5x over-cap.

- [x] **P004** — `doctor runtime-scan` subcmd
  - Doctrine: WORKFLOW_V2.2.md §7 Sub-mech F
  - Input: `--repo <path>`, `--include-home` flag (default false)
  - Logic: grep regex patterns trong `.git/config`, `.env*`, `.mcp.json`. Optional: `~/.ssh/config`, `~/.gitconfig` (chỉ khi `--include-home`).
  - Patterns: `ghp_*`, `gho_*`, `ghu_*`, `ghs_*`, `github_pat_*`, AWS access key `AKIA[0-9A-Z]{16}`, OpenAI `sk-*`, etc.
  - Output: exit 0 = clean, exit 1 = leak found với file:line.
  - Mã phiếu xác chết: P305 tarot — token plaintext local + VPS (Sub-mech F instance #10).

- [x] **P005** — MCP serve mode
  - Doctrine: pattern khớp advisory-inbox P010-P011 (rmcp 1.7.0)
  - Input: `doctor serve` (stdin/stdout JSON-RPC 2.0)
  - Logic: expose 4 subcmd as MCP tools. Caller (Claude session) invoke via `mcp__doctor__lane_check`, etc.
  - Output: long-running process. Errors via JSON-RPC error response.
  - Mã phiếu xác chết: advisory-inbox MCP precedent (em đã build 3 ngày trước).

## Out of MVP (sensor / defer)

- `phieu-next` — counter increment (defer M6 team race chưa nổ, em solo)
- `migrate-check` — Sub-mech C (defer khi migration phiếu fire)
- `doctrine-check` — Sub-mech D commit msg home: (defer)
- `hook-wall-time` — §10 N4 sensor (defer)
- `lane-override-rate` — §1 metric Tier 2 (defer, cần 50 PR data)
- `verify-setup` — per-repo bootstrap verification (defer)

## Done

### MVP sprint (2026-05-28) — 5/5 phiếu, 29/29 tests

- **P001** — `doctor lane-check` — commit `a4fddd4`, 5 tests
- **P002** — `doctor validate-map` — commit `9bde634`, 7 tests
- **P003** — `doctor rotate-check` — commit `7c1f9ef`, 7 tests (+ `toml = "0.8"` dep)
- **P004** — `doctor runtime-scan` — commit `c34048a`, 8 tests
- **P005** — MCP serve mode — commit `076598c`, 2 smoke tests (refactor 4 subcmd → `pub fn run()`, new `src/mcp/` module, rmcp 1.7.0 stdio)

State machine v2.2: 5 phiếu, 2 RESPOND turns (P004 sk-proj comment fix, P005 sync fn + return type design), 0 escalation. Sếp delegated full APPROVAL to orchestrator — em duyệt 5/5.
