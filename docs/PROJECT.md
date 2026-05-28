# doctor — v2.2 mechanical gate enforcement

## Mission

doctor là **dụng cụ** (NOT thí nghiệm) của Workflow v2.2 — 1 Rust binary cài qua `cargo install`, cung cấp 4 subcmd enforce mechanical sound gates cho mọi repo dùng sos-kit v2.2.

**Doctrine source:** `~/sos-kit/docs/WORKFLOW_V2.2.md` (single-source-of-truth).
**Retro trace:** `~/sos-kit/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md` (7-round forge, CLOSED 2026-05-28).

## Khác biệt với thí nghiệm

| Loại | Mục đích | Ngôn ngữ | Repo |
|------|----------|----------|------|
| **doctor** (this repo) | DỤNG CỤ — portable, cài 1 lần dùng nhiều repo | Rust (oracle SOUND) | aspelldenny/doctor |
| **rotate-archive** (pilot vòng 2, sau) | THÍ NGHIỆM — partial-oracle, test v2.2 chống mù | Python | TBD |

Hai cái chồng nhau ở việc rotate DISCOVERIES/CHANGELOG. Split tier:
- doctor `rotate-check` Rust = SOUND phần (đếm dòng, cảnh báo cap).
- rotate-archive Python = PARTIAL phần (phân loại doctrine/operational, quyết cắt, format lạ).

## 4 MVP subcmd (mỗi cái có mã phiếu xác chết)

| Subcmd | Doctrine | Mã phiếu / lỗi thật |
|--------|----------|---------------------|
| `doctor lane-check` | §1 lane budget (đếm dòng + anchor + constraint vs lane field) | P003 advisory-inbox 643 dòng cho 6 test |
| `doctor validate-map` | §4 AGENT_MAP path/anchor exist (SOUND only — KHÔNG check contract_test compile) | DISCOVERIES tarot 265KB drift, rule-không-enforce |
| `doctor rotate-check` | §6 dòng cap DISCOVERIES/CHANGELOG (soft 1000 warn, hard 1500 block) | DISCOVERIES tarot 265KB over-cap 5x |
| `doctor runtime-scan` | Sub-mech F token leak (`.git/config`, `~/.ssh`, `.env*`, `.mcp.json`) | P305 tarot — token plaintext local + VPS (instance #10) |

5 deferred subcmd ship khi sensor nổ thật / batch sau:
- `doctor phieu-next` — defer (M6 team race chưa nổ, em solo)
- `doctor migrate-check` — Sub-mech C (defer khi migration phiếu fire)
- `doctor doctrine-check` — Sub-mech D commit msg home: field
- `doctor hook-wall-time` — §10 N4 sensor (ship khi tiering chính đáng)
- `doctor lane-override-rate` — §1 metric Tier 2 (ship sau khi có data 50 PR)
- `doctor verify-setup` — per-repo bootstrap verification

## Criterion build (round 7 Claude Web doctrine)

> *"Mỗi subcmd phải trả lời được: lỗi nào đã xảy ra thật mà mày chặn?
> Trả lời được bằng MỘT MÃ PHIẾU thì build. Trả lời bằng 'phòng khi...' thì là sensor, để watchlist, đừng cho vào MVP."*

## Workflow

Apply v2.2 doctrine lên chính build doctor:
1. Phiếu P001 → P004 build từng subcmd.
2. Architect DRAFT → Worker CHALLENGE → APPROVAL gate → Worker EXECUTE per phiếu.
3. Test + commit per phiếu.
4. Cuối: `cargo install --path .` → binary cài.
5. `~/.mcp.json` entry để Claude session future invoke via MCP tool.

## Stack

- Language: Rust 2024 edition, rust-version 1.85+
- Build: cargo
- Tests: assert_cmd + predicates (CLI integration), tokio-test (MCP)
- MCP: rmcp 1.7.0 (matches advisory-inbox precedent)
- Distribution: `cargo install --path .` (no crates.io publish yet)

## Status

- [x] Skeleton ship (2026-05-28)
- [ ] P001 lane-check
- [ ] P002 validate-map
- [ ] P003 rotate-check
- [ ] P004 runtime-scan
- [ ] P005 MCP serve mode
- [ ] cargo install + .mcp.json wire
