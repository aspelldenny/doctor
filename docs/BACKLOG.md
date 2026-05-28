# BACKLOG — doctor

## Active sprint — MVP build (5 phiếu)

- [ ] **P001** — `doctor lane-check` subcmd
  - Doctrine: WORKFLOW_V2.2.md §1 lane budget
  - Input: `--ticket <path>` markdown file
  - Logic: đếm dòng phiếu + anchor table rows + constraint count. So với lane field (Normal ≤ 250 / Guarded full / Fast ≤ 100).
  - Output: exit 0 = OK, exit 1 = budget exceeded + reason, exit 2 = ticket missing lane field
  - Mã phiếu xác chết: P003 advisory-inbox 643 dòng cho 6 test.

- [ ] **P002** — `doctor validate-map` subcmd
  - Doctrine: WORKFLOW_V2.2.md §4 AGENT_MAP path/anchor
  - Input: `--map <path>` AGENT_MAP.yaml file
  - Logic: serde_yaml parse + walk all `edit/read_shallow/read_deep/research_gate/contract_test` paths. Check `Path::exists()`. Parse anchor `#section` strings, grep target file cho `^#+\s+<section>`.
  - SOUND only — KHÔNG check contract_test compile/parse (round 5 Claude Web #3 fix).
  - Output: exit 0 = clean, exit 1 = drift (path/anchor missing) với list, exit 2 = yaml parse error.
  - Mã phiếu xác chết: DISCOVERIES tarot 265KB drift.

- [ ] **P003** — `doctor rotate-check` subcmd
  - Doctrine: WORKFLOW_V2.2.md §6 dòng cap
  - Input: `--repo <path>` (default cwd)
  - Logic: đếm dòng `docs/DISCOVERIES.md`, `docs/CHANGELOG.md`. So với cap config `.sos-stack.toml [rotate]` (default soft=1000, hard=1500).
  - SOUND only — KHÔNG phân loại entry hay quyết cắt cái nào (đó là rotate-archive Python pilot vòng 2).
  - Output: exit 0 = under cap, exit 1 = soft cap warn (info), exit 2 = hard cap block.
  - Mã phiếu xác chết: DISCOVERIES tarot 265KB = ~5300 dòng = 5x over-cap.

- [ ] **P004** — `doctor runtime-scan` subcmd
  - Doctrine: WORKFLOW_V2.2.md §7 Sub-mech F
  - Input: `--repo <path>`, `--include-home` flag (default false)
  - Logic: grep regex patterns trong `.git/config`, `.env*`, `.mcp.json`. Optional: `~/.ssh/config`, `~/.gitconfig` (chỉ khi `--include-home`).
  - Patterns: `ghp_*`, `gho_*`, `ghu_*`, `ghs_*`, `github_pat_*`, AWS access key `AKIA[0-9A-Z]{16}`, OpenAI `sk-*`, etc.
  - Output: exit 0 = clean, exit 1 = leak found với file:line.
  - Mã phiếu xác chết: P305 tarot — token plaintext local + VPS (Sub-mech F instance #10).

- [ ] **P005** — MCP serve mode
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

(empty — skeleton vừa ship)
