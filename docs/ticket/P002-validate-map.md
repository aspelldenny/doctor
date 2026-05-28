# PHIẾU P002: `doctor validate-map` subcmd

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 2 (≤3 source files, ≤200 LOC, anchor rõ, no schema/API/auth/dep change — serde_yaml/serde/anyhow/regex đã có Cargo.toml)
> **Lane:** Normal (≤ 250 dòng phiếu, ≤ 5 anchor, ≤ 5 constraint)
> **Ảnh hưởng:** `src/cli/validate_map.rs`, `src/main.rs` (chỉ nếu chưa wire), `tests/validate_map_test.rs` (new)
> **Dependency:** P001 ship (commit a4fddd4) — pattern Worker đã set: anyhow Result, clap derive, no unwrap, assert_cmd+tempfile

---

## Context

### Vấn đề hiện tại

Workflow v2.2 §4 chốt AGENT_MAP.yaml cho architect DỪNG đọc khi đã thấy hết blast field. Map drift = thuốc độc chậm (DISCOVERIES tarot 265KB precedent: anchor pointed at sections không còn). Skeleton `src/cli/validate_map.rs` hiện `todo!()`.

### Giải pháp

Implement `doctor validate-map --map <path>` SOUND path/anchor checker:

1. Parse yaml qua `serde_yaml::from_str` vào struct typed (KHÔNG `Value` walk).
2. Walk mọi surface qua 5 path category: `edit`, `read_shallow`, `read_deep`, `research_gate`, `contract_test`.
3. Mỗi entry dạng `path` HOẶC `path#anchor`: split `#`, `Path::exists()` → `PATH_MISSING` nếu miss; có anchor → grep target file regex `^#+\s+<anchor>` (markdown heading) HOẶC `^(pub\s+)?(fn|struct|enum|trait|impl)\s+<anchor>` (Rust symbol) → `ANCHOR_MISSING` nếu miss.
4. Anchor match flexible: case-insensitive + hyphen↔space tolerant (`My-Section` ↔ `## My Section`).
5. Exit: 0 clean, 1 drift (list ra stderr), 2 yaml parse error.

### Scope

- CHỈ sửa: `src/cli/validate_map.rs` (replace `todo!()`), `tests/validate_map_test.rs` (new).
- ĐỘNG `src/main.rs` CHỈ KHI Worker grep xác nhận subcmd `ValidateMap` chưa wire (anchor #2).
- KHÔNG sửa: 3 subcmd khác, `Cargo.toml`, `lib.rs`, `src/cli/mod.rs`.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `src/cli/validate_map.rs` skeleton có `pub struct Args { map: PathBuf }` + `pub fn run(args: Args) -> Result<()>` chứa `todo!()` `[unverified — per ARCHITECTURE.md L45-58]` | `grep -nE "todo!\|pub fn run\|pub struct Args\|map.*PathBuf" /Users/nguyenhuuanh/doctor/src/cli/validate_map.rs` | ✅ Read confirmed: L19-23 `pub struct Args { pub map: PathBuf }`, L25-27 `pub fn run` with `todo!()` |
| 2 | `src/main.rs` đã wire `ValidateMap(cli::validate_map::Args)` enum variant + dispatch arm `cli::validate_map::run` `[needs Worker verify — P001 anchor #3 đã ✅ cho LaneCheck, expect tương tự]` | `grep -nE "ValidateMap\|validate_map::run" /Users/nguyenhuuanh/doctor/src/main.rs` | ✅ L34 variant + L51 dispatch arm confirmed |
| 3 | `serde_yaml 0.9`, `serde 1 derive`, `regex 1`, `anyhow 1` available `[verified — Cargo.toml L15-24]` | `grep -E "^(serde_yaml\|serde \|regex\|anyhow)" /Users/nguyenhuuanh/doctor/Cargo.toml` | ✅ L18 serde_yaml 0.9, L16 serde 1 derive, L24 regex 1, L21 anyhow 1 |
| 4 | `assert_cmd`, `predicates`, `tempfile` available `[verified — Cargo.toml L23,31,32]` | `grep -E "^(assert_cmd\|predicates\|tempfile)" /Users/nguyenhuuanh/doctor/Cargo.toml` | ✅ L23 tempfile 3 (deps), L31 assert_cmd 2 (dev-deps), L32 predicates 3 (dev-deps) |
| 5 | AGENT_MAP schema: `version: 1`, `surfaces: { <name>: { load_bearing, edit[], read_shallow[]?, read_deep[]?, research_gate[]?, contract_test[]?, blast, oracle_soundness? } }`, `never_default_read[]?`; entries có thể là `path/**` glob HOẶC `path#anchor` HOẶC plain `path` `[verified — docs/AGENT_MAP.yaml.example L25-125]` | Read `docs/AGENT_MAP.yaml.example` | ✅ Schema confirmed — 5 path categories per surface, glob `**` allowed |

**Anchor #1+#2 quyết định scope:** #2 ✅ → Worker KHÔNG đụng `src/main.rs`, drop khỏi edit_allow. #2 ❌ → Worker wire dispatch trước implement (mirror P001 outcome).

**Glob handling:** Entries `src/parser/**` → skip path-exists check (glob ≠ file path đơn). Anchor không apply cho glob. Worker decide EXECUTE, default skip, doc Discovery.

---

## Debate Log

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge (phiếu V1)

**Worker accepted V1 — no challenges.** Ready for Chủ nhà approval gate.

**Anchor verification:**
- [#1] ✅ `src/cli/validate_map.rs` has `pub struct Args { pub map: PathBuf }` + `pub fn run(_args: Args) -> Result<()>` with `todo!()` at L25-27. Grep confirmed via Read (grep -nE returned empty because field names differ slightly: `map` not `map.*PathBuf` pattern — file has `pub map: PathBuf` not `map.*PathBuf` literal, Read confirmed actual content).
- [#2] ✅ `src/main.rs:34` `ValidateMap(cli::validate_map::Args)` variant present; `src/main.rs:51` dispatch arm `cli::validate_map::run(args)` present. **Task 4 → SKIP, do NOT edit `src/main.rs`.**
- [#3] ✅ Cargo.toml: serde 1 (derive), serde_yaml 0.9, anyhow 1, regex 1.
- [#4] ✅ Cargo.toml: tempfile 3 (deps), assert_cmd 2 (dev-deps), predicates 3 (dev-deps).
- [#5] ✅ AGENT_MAP.yaml.example confirmed: version/surfaces/5 categories/never_default_read schema matches phiếu description. `oracle_soundness` nested map field present on some surfaces — serde_yaml ignores unknown fields by default (no `deny_unknown_fields`), so omitting it from struct is safe (SOUND, confirmed via grep: no `deny_unknown_fields` in src/).

**Self-close notes (oracle-first §2):**
- [SC1] `oracle_soundness` skip safety
  Claim: serde_yaml will error if struct lacks `oracle_soundness` field present in YAML.
  Oracle: grep `deny_unknown_fields` across src/ → 0 hits; serde default = ignore unknown fields.
  Soundness: SOUND for this claim. Verdict: self-closed via oracle.
- [SC2] `never_default_read` not walked by Task 2 loop
  Claim: phiếu omits path-check for `never_default_read` entries — is this a gap?
  Oracle: Read AGENT_MAP.yaml.example — all `never_default_read` entries are either globs (`docs/Archive/**`) or plain paths. Glob entries would be skipped anyway per phiếu's glob rule. Non-glob entries (e.g., `docs/CHANGELOG.md`) could theoretically be checked, but phiếu scope explicitly says "5 path category: edit, read_shallow, read_deep, research_gate, contract_test" — not `never_default_read`.
  Soundness: SOUND (scope is explicit). Verdict: self-closed — Worker to document in Discovery as known omission.

**Status:** ✅ ACCEPTED — no objections, all 5 anchors verified, Task 4 skipped (main.rs already wired).

### Turn 1 — Architect Response

*(Filled if objection raised. Cap 3 turns.)*

---

## Nhiệm vụ

### Task 1: Define struct match AGENT_MAP schema

**File:** `src/cli/validate_map.rs`

**Tìm:** `todo!()` body `pub fn run` (anchor #1) + top of file.

**Thay bằng / Thêm:** `serde::Deserialize` struct `AgentMap { version: u32, surfaces: BTreeMap<String, Surface>, #[serde(default)] never_default_read: Vec<String> }` + `Surface { #[serde(default)] load_bearing: bool, #[serde(default)] edit: Vec<String>, read_shallow, read_deep, research_gate, contract_test all `Vec<String>` default, blast: String default }`. Skip `oracle_soundness` (doctor không check soundness semantic).

**Lưu ý:** `#[serde(default)]` mọi field optional. KHÔNG `serde_yaml::Value` walk (Constraint 3).

### Task 2: Parse + walk + collect drift

**File:** `src/cli/validate_map.rs`

**Tìm:** Sau struct definition.

**Thay bằng / Thêm:** (1) `read_to_string(&args.map)?`. (2) `serde_yaml::from_str` map fail `anyhow!("yaml parse: {e}")` (caller exit 2). (3) `let mut drifts: Vec<String> = Vec::new();` (4) Loop surface × 5 category × entry: glob `**` → skip; else split `#` → `(path, Option<anchor>)`; `!Path::new(path).exists()` → push `"<surface>.<category>: <entry> — PATH_MISSING"`; có anchor + path exists → Task 3 helper, miss → push `... — ANCHOR_MISSING`. (5) `drifts.is_empty()` → Ok; else sort, `eprintln!` mỗi line, return distinguishable error cho exit 1.

**Lưu ý:** Sort drift deterministic (test assert content). Worker chọn `anyhow::Error` custom message HOẶC `thiserror` enum — pattern khớp P001, document Discovery. Exit code dispatch ở `main.rs` arm.

### Task 3: Anchor flexible match helper

**File:** `src/cli/validate_map.rs` (helper fn)

**Tìm:** Sau Task 2 cấu trúc.

**Thay bằng / Thêm:** `fn anchor_exists(target: &Path, anchor: &str) -> Result<bool>`:
1. Read target content.
2. Normalize anchor: lowercase + `-` → space.
3. Regex `(?mi)^#+\s+(.*)$` heading: capture text, normalize (lowercase, `-`→space, collapse whitespace), eq normalized anchor.
4. Regex `(?m)^(?:pub\s+)?(?:fn|struct|enum|trait|impl)\s+(\w+)` Rust symbol: capture, lowercase compare với anchor lowercase (Rust idents không có `-`).
5. Return true nếu either match.

**Lưu ý:** Flexible vì AGENT_MAP có thể URL-style (`my-section`) trong khi heading `## My Section`. KHÔNG `%20` URL-decode (out of scope SOUND). Worker fixture cover: exact, hyphen-space, Rust fn, miss.

### Task 4: Wire subcmd vào main.rs (CONDITIONAL — chỉ nếu anchor #2 ❌)

**File:** `src/main.rs`

**Tìm:** Clap `Commands` enum + match dispatch (Worker grep anchor #2).

**Thay bằng:** Nếu chưa wire, thêm `ValidateMap(cli::validate_map::Args)` variant + arm. Exit mapping (mirror P001): Ok → 0; Err yaml-parse-flavor → 2; Err drift → 1; Err I/O → 2. Worker decide error classification (substring vs error enum).

**Lưu ý:** Anchor #2 ✅ → SKIP, KHÔNG edit `src/main.rs`. Worker log Discovery.

### Task 5: Integration test với assert_cmd + tempfile

**File:** `tests/validate_map_test.rs` (new)

**Thay bằng / Thêm:** ≥ 4 case dùng `tempfile::tempdir()` workspace + fixture yaml inline (raw string) + dummy target files qua `std::fs::write`:
1. `ok_clean_all_exists` — 2 surface, paths + 1 anchor heading match → exit 0.
2. `fail_path_missing` — yaml ref path không tạo → exit 1, stderr `PATH_MISSING`.
3. `fail_anchor_missing` — path exists nhưng anchor miss → exit 1, stderr `ANCHOR_MISSING`.
4. `fail_yaml_parse` — invalid yaml (`surfaces: [not a map]`) → exit 2.
5. (Optional) `ok_anchor_flexible_hyphen_space` — anchor `My-Section`, target `## My Section` → exit 0.

**Lưu ý:** `assert_cmd::Command::cargo_bin("doctor")`, pattern khớp `tests/lane_check_test.rs` (precedent P001 Worker đã set).

---

## Edit scope (v2.2 §5)

```yaml
edit_allow:
  - src/cli/validate_map.rs
  - src/main.rs            # ONLY if anchor #2 ❌
  - tests/validate_map_test.rs

verify_read:
  - src/cli/validate_map.rs
  - src/cli/lane_check.rs   # reference P001 pattern
  - src/main.rs
  - src/cli/mod.rs
  - Cargo.toml
  - docs/ARCHITECTURE.md
  - docs/AGENT_MAP.yaml.example
  - docs/ticket/P001-lane-check.md
  - tests/validate_map_test.rs

contract_tests:
  - tests/validate_map_test.rs
```

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/cli/validate_map.rs` | Task 1-3: struct + parse + walk + anchor-grep, replace `todo!()` |
| `src/main.rs` | Task 4 (conditional): wire `ValidateMap` arm với exit mapping nếu chưa wire |
| `tests/validate_map_test.rs` | Task 5: integration test 4-5 case dùng tempfile workspace |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/cli/mod.rs` | `pub mod validate_map;` đã expose — Worker grep confirm, KHÔNG edit |
| `src/cli/lane_check.rs` | P001 không regression (`cargo test --test lane_check_test` pass) |
| `src/cli/rotate_check.rs` + `runtime_scan.rs` | Vẫn `todo!()`, compile fine |
| `Cargo.toml` | KHÔNG add dep mới — serde/serde_yaml/regex/anyhow/assert_cmd/tempfile đủ |

---

## Luật chơi (Constraints)

1. **SOUND only** — `Path::exists()` + anchor-grep mechanical. KHÔNG check semantic target file, KHÔNG validate contract_test compile/parse (chốt round 5 Claude Web #3), KHÔNG judge blast wording.
2. **Anchor match flexible** — case-insensitive + hyphen↔space tolerant. Case study: `My-Section` match `## My Section` HOẶC fn `my_section` (Worker quyết underscore Tầng 2). Fixture cover exact + hyphen-space.
3. **Struct typed parse** — `serde::Deserialize` derive trên `AgentMap`/`Surface`, KHÔNG `serde_yaml::Value` untyped walk. `#[serde(default)]` mọi field optional. Glob entries (`**`) → skip path check, doc Discovery.
4. **No `unwrap()` / `panic!`** code path chính — `anyhow::Result<()>` + `?`. Exception: `Regex::new(literal).unwrap()` chấp nhận (precedent P001).
5. **Tổng LOC ≤ 200** cho 3 file (Tầng 2 cap). Vượt → Worker STOP, AskUserQuestion (promote Tầng 1 hoặc split).

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean, no new warnings.
- [ ] `cargo test --test validate_map_test` — 4-5 case pass.
- [ ] `cargo test` full suite — không regression P001.

### Manual Testing
- [ ] `cargo run -- validate-map --map docs/AGENT_MAP.yaml.example` → exit 0 nếu paths exists, HOẶC exit 1 với drift list (example là sample — document outcome Discovery).
- [ ] Fixture invalid yaml → exit 2.
- [ ] Fixture path missing → exit 1 stderr `PATH_MISSING`.
- [ ] Fixture anchor missing → exit 1 stderr `ANCHOR_MISSING`.

### Regression
- [ ] `cargo test --test lane_check_test` — P001 5 test pass.
- [ ] `cargo build` cho `rotate_check.rs` + `runtime_scan.rs` skeleton — `todo!()` compile fine.
- [ ] `doctor --help` show 4 subcmd.

### Docs Gate
- [ ] `CHANGELOG.md` — `P002: validate-map subcmd (5 path categories, anchor flexible match, 3 exit codes)`.
- [ ] `docs/ARCHITECTURE.md` L43-58 — Worker verify khớp contract, update comment nếu drift (glob skip).

### Discovery Report
- [ ] Write `docs/discoveries/P002.md`:
  - Anchor #1-2 result (#2 wire decision)
  - Edit scope: src/main.rs touched? Y/N + lý do
  - Glob handling: skip vs walkdir (Tầng 2 choice + rationale)
  - Anchor flexible match algorithm
  - Exit code pattern (anyhow downcast vs error enum) + lý do
  - 3-field SOUND oracle:
    ```
    Claim: validate-map detects path/anchor drift per §4
    Oracle: cargo test + manual run on AGENT_MAP.yaml.example
    Soundness: SOUND (Path::exists + regex grep exact, no judgment)
    ```
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
