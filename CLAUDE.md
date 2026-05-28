# CLAUDE.md — doctor

> Read this BEFORE editing anything in this repo.
> Doctrine source: `~/sos-kit/docs/WORKFLOW_V2.2.md` (single-source-of-truth).
> Retro trace: `~/sos-kit/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md` (7-round forge, CLOSED 2026-05-28).

## What this repo is

doctor = **dụng cụ** Rust binary cung cấp 4 SOUND mechanical gates cho Workflow v2.2:

| Subcmd | Doctrine | Mã phiếu xác chết |
|--------|----------|-------------------|
| `doctor lane-check` | §1 lane budget | P003 advisory-inbox |
| `doctor validate-map` | §4 AGENT_MAP path/anchor | DISCOVERIES tarot drift |
| `doctor rotate-check` | §6 dòng cap | DISCOVERIES tarot 265KB over-cap |
| `doctor runtime-scan` | Sub-mech F token leak | P305 tarot |

## What this repo is NOT

- **Not a judgment engine.** Doctor đếm/grep/check path. Judgment ở boundary-check subagent (advisory mode).
- **Not a partial-oracle test.** Test value sẽ ở Python pilot vòng 2 (rotate-archive). Doctor là dụng cụ, không phải thí nghiệm.
- **Not a CI/CD pipeline.** Doctor là CLI invoked by pre-commit hooks + orchestrator + slash commands. Pipeline orchestration ở caller side.

## Doctrine — 3 luật cứng (v2.2 §0.1)

1. **Mỗi fix gắn cờ** `[gate]` / `[hook]` / `[guidance]` — KHÔNG prose để agent nhớ.
2. **Một bệnh, một cơ chế** rẻ nhất bắt 80%. Cấm 3 tầng cho 1 bệnh.
3. **Mechanical mới gate**, judgment giữ guidance. Đừng ép judgment thành hook giả → phán bừa.

## 3 hard lines doctor (xem `docs/SOUL.md`)

1. MECHANICAL SOUND ONLY — không ôm judgment.
2. RUST PORTABLE, CARGO INSTALL ONCE — 1 binary, mọi repo.
3. KHÔNG NUỐT PILOT THÍ NGHIỆM — khi gặp PARTIAL, defer Python pilot vòng 2.

**Câu hỏi vàng em phải tự hỏi suốt:** *"Làm Rust vì nó giải vấn đề test, hay vì em thích Rust?"*

## Workflow v2.2 áp dụng cho build doctor

Sprint 5 phiếu (P001-P005). Mỗi phiếu:

1. Orchestrator (Quản đốc, main session) đọc BACKLOG → pick phiếu.
2. Spawn `architect` (DRAFT mode) → viết phiếu V1 với Task 0 anchors.
3. Spawn `worker` (CHALLENGE mode) → verify Task 0 + grep code thật → Debate Log Turn 1.
4. (Nếu có objection) Spawn `architect` (RESPOND mode) → phiếu V2.
5. Loop tới consensus HOẶC max 3 turns → FORCE_ESCALATION.
6. **APPROVAL gate** — orchestrator `AskUserQuestion` show phiếu cuối, Sếp duyệt.
7. Spawn `worker` (EXECUTE mode) → Task 0 → code → tests → Discovery → commit.
8. Sếp nghiệm thu.

## Lane budget (v2.2 §1)

| Lane | Budget | Use for |
|------|--------|---------|
| Normal | ≤ 250 dòng phiếu, ≤ 5 anchor, ≤ 5 constraint | Default cho P001-P004 subcmd implementation |
| Guarded | Full quyền | Schema change, security boundary, cross-cutting |
| Fast | ≤ 100 dòng, KHÔNG architect | Lặt vặt, oracle SOUND only |

Override: Chủ nhà approval explicit reason. Metric weekly (>20% rolling 50 PR → workflow-tune ticket).

## Oracle-first routing (v2.2 §2)

Worker CHALLENGE hỏi 2 câu trước mọi objection:
1. **Claim:** what objection actually asks?
2. **Oracle + soundness:** what tool phán đúng CLAIM, SOUND or PARTIAL?

Routing:
- `[mechanical + SOUND]` → Worker self-close (CHALLENGE: log fix plan; EXECUTE: apply fix).
- `[shape + SOUND]` → Worker compile/probe.
- `[shape + PARTIAL]` → Worker chạy oracle như sàng, contract-test verify final.
- `[design / security]` → Architect respond.

3-field BẮT BUỘC trong Discovery khi self-close:
```
Claim: <what objection asks>
Oracle: <tool/command>
Soundness: SOUND | PARTIAL | NONE for this claim
```

## Edit-scope vs verify-scope (v2.2 §5)

Phiếu BẮT BUỘC khai báo:
```yaml
edit_allow:           # [gate] mechanical, grep diff
  - src/cli/lane_check.rs
verify_read:          # [guidance] worker self-report
  - src/cli/lane_check.rs
  - docs/ARCHITECTURE.md
  - tests/lane_check_test.rs
contract_tests:       # [gate] pre-commit/pre-merge
  - tests/lane_check_integration_test.rs
```

Worker EXECUTE pre-commit: `git diff --name-only` vs `edit_allow` glob. Outside allow → STOP, AskUserQuestion.

## Boundary-check rubric injection (v2.2 §8)

TRƯỚC khi spawn boundary-check subagent (qua `/security-review` slash):
1. Read `docs/security/INVARIANTS.md`
2. Extract `INV-LOCAL-*` block
3. Paste verbatim vào spawn prompt

Boundary-check em (subagent) KHÔNG tự grep INVARIANTS — canary 2 finding (2026-05-28).

## Anti-patterns (orchestrator MUST NOT)

1. **Code yourself.** Spawn `worker` (EXECUTE). Main session = orchestrator.
2. **Skip APPROVAL gate.** Only mandatory user gate. KHÔNG fake-gate mid-state-machine.
3. **Skip CHALLENGE phase** for Tầng 1 phiếu (architectural, schema, API, security boundary).
4. **Read source for "context."** That's Worker's surface. Use AGENT_MAP if available; sequential-thinking for high-level.
5. **Make doctor implement judgment.** Doctor là SOUND mechanical only. Judgment → boundary-check advisory mode HOẶC Python pilot vòng 2.

## Common tasks

### Build next subcmd
1. Sếp pick P00X từ BACKLOG.
2. Orchestrator state machine: DRAFT → CHALLENGE → APPROVAL → EXECUTE.
3. Apply edit_allow gate per worker.md instruction.
4. Run `cargo build && cargo test` post-EXECUTE.
5. Discovery report → `docs/discoveries/P00X.md` + index entry.

### Test scaffold compile (no implementation)
```bash
cargo build
# exit 0 = scaffold OK (todo!() panics at runtime but compiles fine)
```

### Install binary (post-P005 MCP ship)
```bash
cargo install --path .
# binary cài vào ~/.cargo/bin/doctor
which doctor && doctor --version
```

### Wire MCP entry
```json
// ~/.mcp.json (user-level)
{
  "mcpServers": {
    "doctor": {
      "command": "/Users/nguyenhuuanh/.cargo/bin/doctor",
      "args": ["serve"]
    }
  }
}
```

## Related repos

- `~/sos-kit` — doctrine source (`docs/WORKFLOW_V2.2.md`)
- `~/advisory-inbox` — precedent Rust binary (CLI + MCP, 7 subcmd, 69 tests)
- `~/tarot` — primary user of doctor (will adopt after P004 runtime-scan ships)

## Language

- Internal communication (Sếp ↔ Claude): Vietnamese
- Code comments: English
- Commit messages: English
- Public README: English
- Phiếu body: Vietnamese (Sếp + em internal)
