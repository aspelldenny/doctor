# PHIẾU P003: `doctor rotate-check` subcmd

> **Loại:** Feature
> **Ưu tiên:** P1
> **Tầng:** 1 (new dep `toml` add Cargo.toml — móng nhà rule; còn lại scope ≤3 files)
> **Lane:** Normal (≤ 250 dòng phiếu, ≤ 5 anchor, ≤ 5 constraint)
> **Ảnh hưởng:** `src/cli/rotate_check.rs`, `Cargo.toml`, `Cargo.lock` (auto), `tests/rotate_check_test.rs` (new)
> **Dependency:** P002 ship (commit 9bde634) — pattern Worker: anyhow Result, clap derive, no unwrap, assert_cmd+tempfile, integration test fixture qua `tempfile::tempdir()`

---

## Context

### Vấn đề hiện tại

Workflow v2.2 §6 chốt dòng cap cho `docs/DISCOVERIES.md` + `docs/CHANGELOG.md` để file log không phình → architect/agent đọc quá tải. Mã phiếu xác chết: DISCOVERIES tarot 265KB = ~5300 dòng = 5× over-cap (1500 hard). Skeleton `src/cli/rotate_check.rs` hiện `todo!()`. Cần dụng cụ SOUND mechanical đếm dòng + compare cap, exit code phân loại severity, gọi từ pre-commit hook hoặc orchestrator.

### Giải pháp

Implement `doctor rotate-check --repo <path>` SOUND line-counter:

1. Resolve repo root: `args.repo` HOẶC `std::env::current_dir()` fallback.
2. Load `.sos-stack.toml` tại repo root nếu exist: parse `[rotate]` table cho `files: Vec<String>`, `soft: usize`, `hard: usize`. Field thiếu → fallback default.
3. Default config: `files = ["docs/DISCOVERIES.md", "docs/CHANGELOG.md"]`, `soft = 1000`, `hard = 1500`.
4. Cho mỗi file: `Path::exists()` false → skip, KHÔNG drift (file chưa tạo là OK — case study `docs/CHANGELOG.md` repo này chưa có). Exist → đếm dòng (`content.lines().count()`).
5. Compare lines vs soft/hard. Track worst severity across all files.
6. Exit: 0 = mọi file < soft; 1 = ≥ 1 file `soft ≤ lines < hard` (warn — print file + lines + soft cap); 2 = ≥ 1 file `lines ≥ hard` (block — print file + lines + hard cap). Worst wins.
7. SOUND only — KHÔNG suggest cắt entry nào (rotate-archive Python pilot vòng 2).

### Scope

- CHỈ sửa: `src/cli/rotate_check.rs` (replace `todo!()`), `Cargo.toml` (+1 dep), `tests/rotate_check_test.rs` (new).
- ĐỘNG `src/main.rs` CHỈ KHI Worker grep xác nhận subcmd `RotateCheck` chưa wire (anchor #2). P001+P002 precedent: skeleton đã wire 4 variants, dự kiến SKIP.
- KHÔNG sửa: 3 subcmd khác, `lib.rs`, `src/cli/mod.rs`, `docs/`.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `src/cli/rotate_check.rs` skeleton có `pub struct Args { pub repo: Option<PathBuf> }` + `pub fn run(args: Args) -> Result<()>` chứa `todo!()` `[unverified — per ARCHITECTURE.md L62-74, mirror P001/P002 skeleton pattern]` | `grep -nE "todo!\|pub fn run\|pub struct Args\|repo.*Option" /Users/nguyenhuuanh/doctor/src/cli/rotate_check.rs` | ✅ L21-29: struct Args + run(_args) + todo!() confirmed. Note: skeleton uses `_args` not `args` (unused param prefix). |
| 2 | `src/main.rs` đã wire `RotateCheck(cli::rotate_check::Args)` enum variant + dispatch arm `cli::rotate_check::run` `[needs Worker verify — P001/P002 anchor #2 đều ✅ wire, expect tương tự]` | `grep -nE "RotateCheck\|rotate_check::run" /Users/nguyenhuuanh/doctor/src/main.rs` | ⏳ TO VERIFY |
| 3 | Cargo.toml KHÔNG có `toml` crate (cần add). `serde 1 derive`, `anyhow 1`, `assert_cmd 2`, `tempfile 3`, `predicates 3` đã có `[verified — Cargo.toml L14-32 Read]` | `grep -E "^(toml\|serde \|anyhow\|assert_cmd\|tempfile\|predicates)" /Users/nguyenhuuanh/doctor/Cargo.toml` | ✅ Cargo.toml L14-32: serde 1 derive (L16), anyhow 1 (L21), tempfile 3 (L23), assert_cmd 2 (L31), predicates 3 (L32). `toml` KHÔNG có → Task 3 add `toml = "0.8"`. |
| 4 | Repo root file presence: `docs/DISCOVERIES.md` exists, `docs/CHANGELOG.md` KHÔNG exist (skip case = no drift), `.sos-stack.toml` KHÔNG exist (fallback default case) `[verified — Glob]` | `ls /Users/nguyenhuuanh/doctor/docs/DISCOVERIES.md /Users/nguyenhuuanh/doctor/docs/CHANGELOG.md /Users/nguyenhuuanh/doctor/.sos-stack.toml` | ✅ DISCOVERIES.md exists; CHANGELOG.md missing (skip rule kicks in); `.sos-stack.toml` missing (default fallback kicks in). Cả 2 negative case test được manual khi `cargo run -- rotate-check`. |
| 5 | `toml = "0.8"` crate API: `toml::from_str::<T>(&content)` parse, serde derive compatible (cùng pattern serde_yaml P002) `[unverified — per crates.io toml 0.8 README]` | Worker verify khi cargo build + viết struct: `#[derive(Deserialize)] struct RotateConfig { #[serde(default)] files: Option<Vec<String>>, soft: Option<usize>, hard: Option<usize> }` parse được. | ✅ SOUND probe at /tmp/test_toml_probe: `toml::from_str::<SosStack>` with full struct compiles and parses `[rotate]\nsoft=10\nhard=20` clean. Self-closed. |

**Anchor #1+#2 quyết định scope:** #2 ✅ → Worker KHÔNG đụng `src/main.rs`, drop khỏi edit_allow (mirror P002). #2 ❌ → Worker wire dispatch trước implement.

**Anchor #4 ý nghĩa MVP test:** repo này tự test `doctor rotate-check --repo .` ra exit 0 (DISCOVERIES nhỏ + CHANGELOG missing skip + .sos-stack.toml missing dùng default). SOUND smoke.

**Anchor #5 oracle:** `cargo build` SOUND cho "toml crate API compatible". Nếu Worker thấy `toml::from_str` không match expected signature → self-close oracle (rebump Cargo.toml dep version HOẶC dùng `toml::Value` walk), log Discovery, KHÔNG cần Architect RESPOND.

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V1 (initial draft)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**
- Anchor #1: ✅ `src/cli/rotate_check.rs` L21-29 — `pub struct Args { pub repo: Option<PathBuf> }` + `pub fn run(_args: Args) -> Result<()>` chứa `todo!()`. Grep pattern ban đầu thất bại do shell escape `!` trong zsh; re-run unquoted confirm.
- Anchor #2: ✅ `src/main.rs` L37 + L52 — `RotateCheck(cli::rotate_check::Args)` và `cli::rotate_check::run(args)` đã wire. Task 4 (conditional wire) → SKIP, KHÔNG edit `src/main.rs`.
- Anchor #3: ✅ `toml` NOT present in Cargo.toml. `serde 1 derive`, `anyhow 1`, `tempfile 3`, `assert_cmd 2`, `predicates 3` đều có. Task 1 add `toml = "0.8"` cần thiết.
- Anchor #4: ✅ `docs/DISCOVERIES.md` exists; `docs/CHANGELOG.md` missing (skip rule); `.sos-stack.toml` missing (default fallback). Cả 2 negative case cover được.
- Anchor #5: ✅ Self-closed via oracle.
  Claim: `toml = "0.8"` + serde derive → `toml::from_str::<SosStack>(&content)` parse `[rotate]` table với optional fields.
  Oracle: `cargo build` probe tại `/tmp/test_toml_probe` — struct `SosStack { rotate: Option<RotateConfig> }` + `RotateConfig { files, soft, hard: Option<...> }` compile và parse clean (Finished, no errors).
  Soundness: SOUND — compiler phán đúng claim "API compatible".

**Objections:** None — no Tầng 1 architectural issues found.

Worker accepted V1 — no challenges. Anchor verification: 5/5 ✅. Ready for Chủ nhà approval gate.

**Status:** ✅ ACCEPTED — NO ARCHITECT RESPONSE NEEDED

### Turn 1 — Architect Response

*(Architect fills RESPOND mode.)*

---

## Nhiệm vụ

### Task 1: Add `toml` dep vào Cargo.toml

**File:** `Cargo.toml`

**Tìm:** `[dependencies]` block, sau dòng `walkdir = "2"` (L25 per anchor #3 Read).

**Thay bằng / Thêm:** Thêm 1 dòng `toml = "0.8"` (alphabetical order acceptable — Worker chọn vị trí cuối block hoặc giữ alphabetical). Cargo.lock sẽ auto-update khi `cargo build`.

**Lưu ý:** KHÔNG đụng dep khác. Worker verify `cargo build` clean sau add.

### Task 2: Define config struct + default

**File:** `src/cli/rotate_check.rs`

**Tìm:** Top of file (use statements) + body `pub fn run` chứa `todo!()` (anchor #1).

**Thay bằng / Thêm:**
1. `use serde::Deserialize;` + std use (`PathBuf`, `Path`, `read_to_string`).
2. `#[derive(Debug, Deserialize)] struct SosStack { #[serde(default)] rotate: Option<RotateConfig> }`.
3. `#[derive(Debug, Deserialize)] struct RotateConfig { #[serde(default)] files: Option<Vec<String>>, #[serde(default)] soft: Option<usize>, #[serde(default)] hard: Option<usize> }`.
4. Const `DEFAULT_FILES: &[&str] = &["docs/DISCOVERIES.md", "docs/CHANGELOG.md"];`, `DEFAULT_SOFT: usize = 1000;`, `DEFAULT_HARD: usize = 1500;`.
5. Helper `fn resolve_config(repo_root: &Path) -> Result<(Vec<String>, usize, usize)>`: nếu `repo_root.join(".sos-stack.toml").exists()` → `read_to_string` + `toml::from_str::<SosStack>` (parse error → `anyhow!("toml parse: {e}")`); rotate field None → all defaults; field Some → per-field unwrap_or default. KHÔNG exist → all defaults.

**Lưu ý:** `#[serde(default)]` mọi field optional. Per-file cap KHÔNG implement v1 (recommend flat schema trước, Discovery note nâng cấp). Worker quyết error wrap (`anyhow!` literal vs `thiserror` enum) — pattern khớp P001/P002.

### Task 3: Count + compare + exit logic

**File:** `src/cli/rotate_check.rs`

**Tìm:** Sau Task 2 helper.

**Thay bằng / Thêm:** Trong `pub fn run`:
1. `let repo_root = args.repo.unwrap_or_else(|| PathBuf::from("."));` (KHÔNG fallback git rev-parse — Tầng 1 simplicity).
2. `let (files, soft, hard) = resolve_config(&repo_root)?;`
3. `let mut worst: u8 = 0; let mut report: Vec<String> = Vec::new();`
4. Loop `files.iter()`: `let full = repo_root.join(rel);` → `!full.exists()` → continue (KHÔNG drift). Exist → `let lines = read_to_string(&full)?.lines().count();` → if `lines >= hard` → `worst = worst.max(2); report.push(format!("BLOCK {}: {} dòng (hard cap {})", rel, lines, hard));` → else if `lines >= soft` → `worst = worst.max(1); report.push(format!("WARN  {}: {} dòng (soft cap {})", rel, lines, soft));`.
5. Print report ra `eprintln!` mỗi line nếu `!report.is_empty()`.
6. Return `Ok(())` nếu worst==0; else error mang exit code (Worker chọn pattern: `anyhow!("rotate-check severity {worst}")` substring downcast HOẶC custom error enum — khớp P001/P002 decision).

**Lưu ý:** Empty file → `lines.count() = 0` → clean. File chỉ có newline cuối → count tùy `lines()` impl (Worker doc Discovery). Worst severity wins — 1 file block trumps 4 file warn.

### Task 4: Wire main.rs exit code dispatch (CONDITIONAL — chỉ nếu anchor #2 ❌)

**File:** `src/main.rs`

**Tìm:** Clap `Commands` enum + match dispatch (Worker grep anchor #2 trước).

**Thay bằng:** Nếu chưa wire, thêm `RotateCheck(cli::rotate_check::Args)` variant + arm. Exit mapping mirror P001/P002: Ok → 0; Err severity 1 → 1; Err severity 2 → 2; Err I/O/toml-parse → 2. Worker decide classification (substring vs enum).

**Lưu ý:** Anchor #2 ✅ → SKIP, KHÔNG edit `src/main.rs`, log Discovery.

### Task 5: Integration test fixture

**File:** `tests/rotate_check_test.rs` (new)

**Thay bằng / Thêm:** ≥ 5 case dùng `tempfile::tempdir()` workspace + `std::fs::write` cho file fixture, gọi `assert_cmd::Command::cargo_bin("doctor").args(["rotate-check", "--repo", tmp_path])`:
1. `no_config_no_files_clean` — empty workspace (không `docs/`, không `.sos-stack.toml`) → exit 0 (skip rule).
2. `default_clean_under_soft` — tạo `docs/DISCOVERIES.md` 100 dòng → exit 0.
3. `default_warn_soft_cap` — tạo file ≥ 1000 dòng nhưng < 1500 → exit 1, stderr `WARN` + `soft cap 1000`.
4. `default_block_hard_cap` — tạo file ≥ 1500 dòng → exit 2, stderr `BLOCK` + `hard cap 1500`.
5. `mixed_worst_wins` — 1 file warn + 1 file block → exit 2 (worst).
6. (Optional) `custom_config_override` — `.sos-stack.toml` với `[rotate]` `soft=10 hard=20` + file 15 dòng → exit 1.
7. (Optional) `invalid_toml_parse_error` — `.sos-stack.toml` content `not valid toml [[[` → exit 2.

**Lưu ý:** Tạo file dòng N qua `"x\n".repeat(N)`. Pattern khớp `tests/validate_map_test.rs` (precedent P002).

---

## Edit scope (v2.2 §5)

```yaml
edit_allow:
  - src/cli/rotate_check.rs
  - Cargo.toml
  - Cargo.lock                  # auto-update khi cargo build sau add dep
  - src/main.rs                 # ONLY if anchor #2 ❌
  - tests/rotate_check_test.rs

verify_read:
  - src/cli/rotate_check.rs
  - src/cli/validate_map.rs     # reference P002 pattern (error wrap, exit dispatch)
  - src/cli/lane_check.rs       # reference P001 pattern
  - src/main.rs
  - src/cli/mod.rs
  - Cargo.toml
  - docs/ARCHITECTURE.md
  - docs/ticket/P002-validate-map.md
  - tests/validate_map_test.rs
  - tests/rotate_check_test.rs

contract_tests:
  - tests/rotate_check_test.rs
```

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/cli/rotate_check.rs` | Task 2-3: config struct + count + compare + exit, replace `todo!()` |
| `Cargo.toml` | Task 1: add `toml = "0.8"` 1 dòng vào `[dependencies]` |
| `Cargo.lock` | Auto-update bởi cargo build sau Task 1 |
| `src/main.rs` | Task 4 (conditional): wire `RotateCheck` arm với exit mapping nếu chưa wire |
| `tests/rotate_check_test.rs` | Task 5: integration test 5-7 case dùng tempfile workspace |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/cli/mod.rs` | `pub mod rotate_check;` đã expose — Worker grep confirm, KHÔNG edit |
| `src/cli/lane_check.rs` | P001 không regression (`cargo test --test lane_check_test` pass) |
| `src/cli/validate_map.rs` | P002 không regression (`cargo test --test validate_map_test` pass) |
| `src/cli/runtime_scan.rs` | Vẫn `todo!()`, compile fine |

---

## Luật chơi (Constraints)

1. **SOUND only** — `Path::exists()` + `read_to_string().lines().count()` + numeric compare. KHÔNG suggest cắt entry nào, KHÔNG phân loại "stale entry" (rotate-archive Python pilot vòng 2 chịu trách nhiệm).
2. **File missing = OK skip, KHÔNG drift** — `docs/CHANGELOG.md` chưa tạo case (anchor #4) phải pass clean. Khác P002 `PATH_MISSING` (AGENT_MAP entry buộc exist, rotate target không buộc).
3. **Default cap & file list khi config thiếu** — `.sos-stack.toml` không exist HOẶC thiếu `[rotate]` HOẶC field thiếu → áp `files = ["docs/DISCOVERIES.md", "docs/CHANGELOG.md"]`, `soft = 1000`, `hard = 1500`. Per-field fallback (không all-or-nothing).
4. **toml crate add chỉ cho phiếu này** — `toml = "0.8"` Cargo.toml. KHÔNG dùng `serde_yaml` cho TOML config. Per-file cap KHÔNG implement v1 (flat `files`/`soft`/`hard`), Discovery note nâng cấp khi cần.
5. **Tổng LOC ≤ 250** cho 3 file (Tầng 1 cap nhẹ hơn vì +new dep). Vượt → Worker STOP, AskUserQuestion.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean sau add `toml` dep, no new warnings.
- [ ] `cargo test --test rotate_check_test` — 5-7 case pass.
- [ ] `cargo test` full suite — không regression P001 + P002.

### Manual Testing
- [ ] `cargo run -- rotate-check --repo .` → exit 0 (DISCOVERIES.md nhỏ + CHANGELOG.md missing skip + .sos-stack.toml missing default).
- [ ] Fixture file ≥ 1500 dòng → exit 2 stderr `BLOCK` + `hard cap 1500`.
- [ ] Fixture file 1000-1499 dòng → exit 1 stderr `WARN` + `soft cap 1000`.
- [ ] Fixture `.sos-stack.toml` invalid → exit 2.
- [ ] Fixture custom cap override (soft=10, hard=20) → trigger đúng.

### Regression
- [ ] `cargo test --test lane_check_test` — P001 pass.
- [ ] `cargo test --test validate_map_test` — P002 pass.
- [ ] `cargo build` cho `runtime_scan.rs` skeleton — `todo!()` compile fine.
- [ ] `doctor --help` show 4 subcmd.

### Docs Gate
- [ ] `CHANGELOG.md` — tạo file mới hoặc thêm entry: `P003: rotate-check subcmd (line cap check, .sos-stack.toml [rotate] config, 3 exit codes, file-missing skip)`. (NOTE: file CHANGELOG.md hiện chưa tồn tại — Worker tạo nếu thiếu.)
- [ ] `docs/ARCHITECTURE.md` L60-74 — Worker verify khớp contract, update comment nếu drift (per-field default, file-missing skip).

### Discovery Report
- [ ] Write `docs/discoveries/P003.md`:
  - Anchor #1-2-5 result (#2 wire decision, #5 toml crate API compat)
  - Edit scope: src/main.rs touched? Y/N + lý do
  - File-missing skip behavior (khác P002 PATH_MISSING)
  - Per-field default vs all-or-nothing decision
  - Per-file cap deferral rationale (v1 flat schema)
  - Exit code pattern (anyhow downcast vs enum) — khớp P001/P002 hay khác?
  - 3-field SOUND oracle:
    ```
    Claim: rotate-check enforces line cap per §6
    Oracle: cargo test fixture (5+ case) + manual `cargo run -- rotate-check --repo .`
    Soundness: SOUND (file count + numeric compare, no entry classification)
    ```
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
