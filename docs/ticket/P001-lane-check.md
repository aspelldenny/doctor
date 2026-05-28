# PHIẾU P001: `doctor lane-check` subcmd

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 2 (≤3 source files, ≤200 LOC, anchor rõ, no schema/API/auth/dep change — clap+regex+anyhow đã có trong Cargo.toml)
> **Lane:** Normal (≤ 250 dòng phiếu, ≤ 5 anchor, ≤ 5 constraint)
> **Ảnh hưởng:** `src/cli/lane_check.rs`, `src/main.rs` (chỉ nếu chưa wire), `tests/lane_check_test.rs` (new)
> **Dependency:** None (skeleton đã ship commit 04a243b)

---

## Context

### Vấn đề hiện tại

Workflow v2.2 §1 chốt lane budget 3 mức (Normal ≤250 dòng / Guarded full / Fast ≤100) sau khi P003 advisory-inbox xác chết 643 dòng cho 6 test. Nhưng KHÔNG có dụng cụ mechanical đo phiếu — Sếp/orchestrator phải tự đếm bằng mắt → drift. Skeleton `src/cli/lane_check.rs` hiện chứa `todo!()`, chưa thực thi.

### Giải pháp

Implement `doctor lane-check --ticket <path>` như SOUND counter:
1. Read file phiếu (markdown).
2. Parse lane field từ header (regex `Lane:\s*(Normal|Guarded|Fast)`).
3. Đếm 3 metric mechanical:
   - **total lines** = số `\n` (đếm dòng file).
   - **anchor rows** = số row trong Task 0 table khớp pattern `^\|\s*\d+\s*\|` (row đánh số).
   - **constraint count** = số mục numbered list dưới heading `## Luật chơi` (pattern `^\d+\.\s`).
4. So với budget:
   - Normal: lines ≤ 250, anchors ≤ 5, constraints ≤ 5.
   - Guarded: skip (full quyền — return OK).
   - Fast: lines ≤ 100, anchors/constraints không gate.
5. Exit code per BACKLOG spec.

### Scope

- CHỈ sửa: `src/cli/lane_check.rs` (replace `todo!()`), `tests/lane_check_test.rs` (new integration test).
- ĐỘNG `src/main.rs` CHỈ KHI Worker grep xác nhận subcmd `LaneCheck` chưa wire vào clap dispatch.
- KHÔNG sửa: 3 subcmd khác (`validate_map.rs`, `rotate_check.rs`, `runtime_scan.rs`), `Cargo.toml` (deps đủ), `lib.rs`.

---

## Task 0 — Verification Anchors

> Architect không có grep tool — Worker grep thật trước khi code.

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `src/cli/lane_check.rs` tồn tại với `pub fn run(args: Args) -> Result<()>` chứa `todo!()` `[unverified — per ARCHITECTURE.md L29-41]` | `grep -n "todo!\|pub fn run\|pub struct Args" /Users/nguyenhuuanh/doctor/src/cli/lane_check.rs` | ✅ L19 struct Args, L25 `pub fn run(_args: Args)`, L26 `todo!()` |
| 2 | `Args` struct field tên `ticket: PathBuf` với `#[arg(long)]` `[unverified — per ARCHITECTURE.md L30]` | `grep -n "ticket\|PathBuf\|arg(long" /Users/nguyenhuuanh/doctor/src/cli/lane_check.rs` | ✅ L16 PathBuf import, L21 `#[arg(long)]`, L22 `pub ticket: PathBuf` |
| 3 | `src/main.rs` đã wire subcmd `LaneCheck` vào clap enum + dispatch tới `cli::lane_check::run` `[needs Worker verify]` | `grep -n "LaneCheck\|lane_check::run\|Commands::" /Users/nguyenhuuanh/doctor/src/main.rs` | ✅ L31 enum variant, L50 dispatch arm — Task 4 SKIP |
| 4 | `regex`, `anyhow`, `clap` 4 derive đã trong `[dependencies]` `[verified — Cargo.toml L15,22,24]` | `grep -E "^(regex\|anyhow\|clap)" /Users/nguyenhuuanh/doctor/Cargo.toml` | ✅ Cargo.toml L15,22,24 |
| 5 | `assert_cmd`, `predicates`, `tempfile` available cho integration test `[verified — Cargo.toml L23,30,31]` | `grep -E "^(assert_cmd\|predicates\|tempfile)" /Users/nguyenhuuanh/doctor/Cargo.toml` | ✅ Cargo.toml L23,30,31 |

**Anchor #3 result quyết định scope:**
- Nếu ✅ wired → Worker KHÔNG đụng `src/main.rs`, edit_allow drop dòng đó.
- Nếu ❌ chưa wire → Worker wire dispatch (1 match arm) trước khi implement run logic.

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**

| # | Result |
|---|--------|
| 1 | ✅ `src/cli/lane_check.rs` L19 `pub struct Args`, L25 `pub fn run(_args: Args) -> Result<()>`, L26 `todo!()` |
| 2 | ✅ L21 `#[arg(long)]`, L22 `pub ticket: PathBuf` — exact match |
| 3 | ✅ `src/main.rs` L31 `LaneCheck(cli::lane_check::Args)`, L50 dispatch arm — Task 4 is SKIP |
| 4 | ✅ Pre-verified by Architect — confirmed `clap`, `anyhow`, `regex` in Cargo.toml |
| 5 | ✅ Pre-verified by Architect — confirmed `assert_cmd`, `predicates`, `tempfile` in Cargo.toml |

**Objections:** None.

**Notes (Tầng 2, self-resolve in EXECUTE):**

- Anchor #1 slight drift: skeleton uses `_args: Args` (underscore prefix, unused param placeholder). EXECUTE mode renames to `args` when implementing — Tầng 2, no escalation needed.
- This phiếu file itself measures 219 lines / 5 anchors / 5 constraints — exactly at Normal budget ceiling. Manual test `cargo run -- lane-check --ticket docs/ticket/P001-lane-check.md` will exit 0 (≤ values pass). Worker should note this in Discovery as a boundary fixture.
- Lane regex `r"(?i)\*\*Lane:\*\*\s*(Normal|Guarded|Fast)"` correctly matches actual header format `> **Lane:** Normal (...)` — the `>` blockquote prefix before `**Lane:**` does not interfere since the pattern is unanchored.

**Status:** ✅ ACCEPTED V1 — no Architect response needed. Ready for Chủ nhà approval gate.

---

## Nhiệm vụ

### Task 1: Parse lane field từ phiếu header

**File:** `src/cli/lane_check.rs`

**Tìm:** `todo!()` trong body `pub fn run` (anchor #1).

**Thay bằng:** Logic đọc file qua `std::fs::read_to_string(&args.ticket)`, dùng `regex::Regex::new(r"(?i)\*\*Lane:\*\*\s*(Normal|Guarded|Fast)").unwrap()` (case-insensitive) để extract. Nếu không match → return `Err(anyhow!("ticket missing Lane field"))` và caller exit 2.

**Lưu ý:** Regex chấp `> **Lane:** Normal` (markdown blockquote header style theo TICKET_TEMPLATE.md L12) HOẶC `**Lane:** Normal`. Worker test cả 2 format trong fixture.

### Task 2: Đếm 3 metric (lines / anchors / constraints)

**File:** `src/cli/lane_check.rs`

**Tìm:** Sau khi Task 1 extract được lane name.

**Thay bằng:**
- `let total_lines = content.lines().count();`
- `let anchor_re = Regex::new(r"(?m)^\|\s*\d+\s*\|").unwrap(); let anchors = anchor_re.find_iter(&content).count();`
- Để đếm constraint dưới `## Luật chơi`: split content tại heading `## Luật chơi`, lấy slice từ đó tới next `^##\s` (hoặc EOF), đếm matches `(?m)^\d+\.\s`. Nếu không có heading → constraints = 0.

**Lưu ý:** Pattern anchor `^\|\s*\d+\s*\|` cố ý chỉ match table row có số (skip header `| # | Assumption |...`). KHÔNG đếm row trong table khác — Task 0 table là table duy nhất theo convention có format này, nhưng Worker note nếu fixture nào có table khác trùng pattern thì doc trong Discovery.

### Task 3: So budget → exit code

**File:** `src/cli/lane_check.rs`

**Tìm:** Sau khi Task 2 đã có 3 số.

**Thay bằng:** `match` trên lane string:
- `"Normal"` (case-insensitive): nếu `lines > 250 || anchors > 5 || constraints > 5` → `eprintln!` reason cụ thể (e.g., `Normal lane: 312 lines > 250 cap`) rồi `std::process::exit(1)`.
- `"Guarded"`: print `OK (Guarded — no cap)` → return Ok.
- `"Fast"`: nếu `lines > 100` → exit 1 với reason.
- Missing field handled in Task 1 → exit 2.

**Lưu ý:** `eprintln!` cho error (stderr), `println!` cho OK message (stdout). Worker assert cả 2 stream trong integration test. KHÔNG dùng `panic!` / `unwrap()` trong code path chính — dùng `anyhow::Result` + `?`.

### Task 4: Wire subcmd vào main.rs (CONDITIONAL — chỉ nếu anchor #3 ❌)

**File:** `src/main.rs`

**Tìm:** Clap `Commands` enum + match dispatch (Worker grep anchor #3).

**Thay bằng:** Nếu chưa wire, thêm variant `LaneCheck(cli::lane_check::Args)` và arm `Commands::LaneCheck(args) => cli::lane_check::run(args)`.

**Lưu ý:** Nếu anchor #3 ✅ (đã wire) → SKIP task này hoàn toàn, KHÔNG edit `src/main.rs`. Worker log decision vào Discovery report.

### Task 5: Integration test với assert_cmd

**File:** `tests/lane_check_test.rs` (new)

**Thay bằng:** ≥ 4 test case:
1. `ok_normal_under_budget` — fixture phiếu 100 dòng, lane Normal, 3 anchors, 3 constraints → exit 0.
2. `fail_normal_lines_over` — fixture 280 dòng Normal → exit 1, stderr contains `lines > 250`.
3. `fail_normal_anchors_over` — fixture 150 dòng Normal nhưng 6 anchor row → exit 1, stderr contains `anchors > 5`.
4. `fail_missing_lane` — fixture không có Lane field → exit 2.
5. (Optional) `ok_guarded_huge` — fixture 600 dòng Guarded → exit 0 (no cap).

**Lưu ý:** Dùng `tempfile::NamedTempFile` cho fixture, `assert_cmd::Command::cargo_bin("doctor")` để invoke. Pattern khớp advisory-inbox tests (precedent — `~/advisory-inbox/tests/`). Fixtures inline trong test file (raw string), KHÔNG cần file riêng.

---

## Edit scope (v2.2 §5)

```yaml
edit_allow:
  - src/cli/lane_check.rs
  - src/main.rs            # ONLY if anchor #3 ❌ — Worker skip nếu đã wire
  - tests/lane_check_test.rs

verify_read:
  - src/cli/lane_check.rs
  - src/main.rs
  - src/cli/mod.rs
  - Cargo.toml
  - docs/ARCHITECTURE.md
  - docs/BACKLOG.md
  - tests/lane_check_test.rs

contract_tests:
  - tests/lane_check_test.rs
```

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/cli/lane_check.rs` | Task 1-3: replace `todo!()` bằng parse/count/gate logic |
| `src/main.rs` | Task 4 (conditional): wire `LaneCheck` subcmd nếu chưa có |
| `tests/lane_check_test.rs` | Task 5: integration test 4-5 case |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/cli/mod.rs` | `pub mod lane_check;` đã expose — Worker chỉ grep confirm, KHÔNG edit |
| `src/cli/validate_map.rs` | Vẫn `todo!()`, `cargo build` không regression |
| `src/cli/rotate_check.rs` | Vẫn `todo!()`, không regression |
| `src/cli/runtime_scan.rs` | Vẫn `todo!()`, không regression |
| `Cargo.toml` | KHÔNG add dep mới — clap/regex/anyhow/assert_cmd/tempfile đủ |

---

## Luật chơi (Constraints)

1. **SOUND only** — đếm dòng/anchor/constraint là mechanical. KHÔNG judge phiếu "well-written" hay "over-engineered". KHÔNG warn về wording. (SOUL Hard line 1.)
2. **No `unwrap()` / `panic!`** trên code path chính — dùng `anyhow::Result<()>` + `?`. Exception: `Regex::new(literal_pattern).unwrap()` chấp nhận vì literal compile-time-safe (precedent advisory-inbox).
3. **Exit code khớp BACKLOG** — 0 OK, 1 budget exceeded (stderr reason), 2 missing lane field. Worker enforce qua `std::process::exit()` ở `main.rs` dispatch HOẶC return distinguishable `anyhow::Error` (Worker chọn pattern, document trong Discovery).
4. **Integration test dùng `assert_cmd` + `tempfile`** — KHÔNG unit test private functions (precedent advisory-inbox pattern, đã có dev-deps).
5. **Tổng LOC ≤ 200** cho 3 file (Tầng 2 cap). Nếu vượt → Worker STOP, escalate Sếp qua AskUserQuestion (có thể cần promote Tầng 1).

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean, no warnings new.
- [ ] `cargo test --test lane_check_test` — 4-5 case pass.
- [ ] `cargo test` (full suite) — không regression skeleton.

### Manual Testing
- [ ] Manual run: `cargo run -- lane-check --ticket docs/ticket/P001-lane-check.md` → exit 0 (chính phiếu này phải pass Normal budget).
- [ ] Manual run với fixture missing Lane → exit 2 với stderr message.
- [ ] Manual run với fixture 300-line Normal → exit 1 với stderr `lines > 250`.

### Regression
- [ ] `cargo build` cho 3 subcmd khác (`validate-map`, `rotate-check`, `runtime-scan`) — vẫn `todo!()` compile fine.
- [ ] `doctor --help` show 4 subcmd (lane-check, validate-map, rotate-check, runtime-scan).

### Docs Gate
- [ ] `CHANGELOG.md` — entry `P001: lane-check subcmd implemented (4 metrics, 3 exit codes)`.
- [ ] `docs/ARCHITECTURE.md` L29-41 — Worker verify code khớp contract, update comment nếu drift.

### Discovery Report
- [ ] Write to `docs/discoveries/P001.md`:
  - Anchor #1-3 result (đặc biệt #3 wire decision)
  - Edge cases: fixture format variation (blockquote vs plain header), Luật chơi heading variant nếu fixture dùng khác wording
  - Exit code pattern chosen (process::exit vs anyhow error) + lý do
  - 3-field SOUND oracle:
    ```
    Claim: lane-check accurately counts and gates per §1 budget
    Oracle: cargo test + manual run on this phiếu file
    Soundness: SOUND (đếm dòng/regex là exact, no judgment)
    ```
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
