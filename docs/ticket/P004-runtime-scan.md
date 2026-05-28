# PHIẾU P004: `doctor runtime-scan` subcmd

> **Loại:** Feature
> **Ưu tiên:** P1 (security gate — token leak detection)
> **Tầng:** 2 (lặt vặt — ≤3 files thực sự sửa, KHÔNG schema/API/auth/new dep change, anchor rõ)
> **Lane:** Normal (≤ 250 dòng phiếu, ≤ 5 anchor, ≤ 5 constraint)
> **Ảnh hưởng:** `src/cli/runtime_scan.rs`, `tests/runtime_scan_test.rs` (new)
> **Dependency:** P003 ship (commit 7c1f9ef) — pattern Worker xác lập: anyhow Result, clap derive, no unwrap, assert_cmd+tempfile, exit code 0/1/2 (2 = parse/config error), per-phiếu Discovery file.

---

## Context

### Vấn đề hiện tại

Workflow v2.2 §7 Sub-mech F (runtime state token leak) — mã phiếu xác chết P305 tarot: token plaintext local + VPS (Sub-mech F instance #10). Skeleton `src/cli/runtime_scan.rs` hiện `todo!()`. Cần dụng cụ SOUND mechanical grep regex pattern qua các config file local (in-repo + optional home) → exit code phân loại (clean / leak). Gọi từ pre-commit hook hoặc orchestrator pre-push.

### Giải pháp

Implement `doctor runtime-scan --repo <path> [--include-home]` SOUND token scanner:

1. Resolve repo root: `args.repo` HOẶC `std::env::current_dir()` fallback.
2. Target file list (in-repo, always):
   - `.git/config`
   - `.mcp.json`
   - Glob `.env*` ở repo root (KHÔNG recursive — `.env` thường ở root). Strategy: `std::fs::read_dir(repo_root)` + filter `file_name().starts_with(".env")` + `file_type().is_file()`.
3. Target file list (home, chỉ khi `--include-home`):
   - `~/.ssh/config`
   - `~/.gitconfig`
   - `~/.netrc`
   - Resolve home: `std::env::var("HOME")` (Unix-only, doctor stays Unix per Cargo.toml `tokio rt` precedent).
4. Patterns (5 vendor whitelist, literal regex):
   - GitHub: `ghp_[A-Za-z0-9]{36,}`, `gho_[A-Za-z0-9]{36,}`, `ghu_[A-Za-z0-9]{36,}`, `ghs_[A-Za-z0-9]{36,}`, `github_pat_[A-Za-z0-9_]{82,}`
   - AWS access: `AKIA[0-9A-Z]{16}`
   - AWS secret: `(?i)aws_secret_access_key\s*=\s*[A-Za-z0-9/+=]{40}`
   - OpenAI: `sk-[A-Za-z0-9]{32,}` (legacy sk-* format only; sk-proj- defer Python pilot vòng 2)
   - Anthropic: `sk-ant-[A-Za-z0-9_-]{32,}`
5. For each target file: `Path::exists()` false → skip (KHÔNG drift); exist → `read_to_string` → iterate `.lines().enumerate()`, mỗi pattern check `regex.is_match(line)`, collect `(path, line_num, pattern_name)`.
6. Output: 1 leak per line stderr, format `<path>:<line>: <pattern_name>`. KHÔNG print matched text (sanitize — không leak secret xuống log/CI).
7. Exit: 0 = clean (no match across all files). 1 = ≥1 leak found. KHÔNG exit 2 (không có config parse).
8. SOUND only — patterns whitelist 5 vendor cố định. KHÔNG generic high-entropy detection (PARTIAL — Python pilot vòng 2 chịu trách nhiệm).

### Scope

- CHỈ sửa: `src/cli/runtime_scan.rs` (replace `todo!()`), `tests/runtime_scan_test.rs` (new).
- ĐỘNG `src/main.rs` CHỈ KHI Worker grep xác nhận `RuntimeScan` chưa wire (anchor #2). P001/P002/P003 precedent: skeleton đã wire 4 variants, dự kiến SKIP.
- KHÔNG sửa: `Cargo.toml` (regex + walkdir đã có per anchor #3), 3 subcmd khác, `lib.rs`, `src/cli/mod.rs`, `docs/`.

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `src/cli/runtime_scan.rs` skeleton có `pub struct Args { pub repo: Option<PathBuf>, pub include_home: bool }` + `pub fn run(args: Args) -> Result<()>` chứa `todo!()` `[unverified — per ARCHITECTURE.md L78-91, mirror P001/P002/P003 skeleton pattern]` | `grep -nE "todo\|pub fn run\|pub struct Args\|include_home\|repo.*Option" /Users/nguyenhuuanh/doctor/src/cli/runtime_scan.rs` | ✅ L19 `pub struct Args`, L22 `pub repo: Option<PathBuf>`, L26 `pub include_home: bool`, L29 `pub fn run(_args: Args)`, L30 `todo!()` — all present. |
| 2 | `src/main.rs` đã wire `RuntimeScan(cli::runtime_scan::Args)` enum variant + dispatch arm `cli::runtime_scan::run(args)` `[needs Worker verify — P001/P002/P003 anchor #2 đều ✅ wire, expect tương tự]` | `grep -nE "RuntimeScan\|runtime_scan::run" /Users/nguyenhuuanh/doctor/src/main.rs` | ✅ L40 `RuntimeScan(cli::runtime_scan::Args)`, L53 `Commands::RuntimeScan(args) => cli::runtime_scan::run(args)`. Worker KHÔNG đụng `src/main.rs`. |
| 3 | Cargo.toml đã có `regex = "1"` (L24) và `walkdir = "2"` (L26). KHÔNG cần add dep mới `[verified — Cargo.toml Read L14-28]` | `grep -E "^(regex\|walkdir) = " /Users/nguyenhuuanh/doctor/Cargo.toml` | ✅ Cargo.toml L24 `regex = "1"`, L26 `walkdir = "2"`. P004 KHÔNG add dep. |
| 4 | Glob `.env*` strategy: `std::fs::read_dir` + filter `file_name().to_str().starts_with(".env")` đủ cho repo-root-only scan. KHÔNG cần walkdir (walkdir reserved cho future recursive case) `[unverified — std lib API per Rust 1.85 docs]` | Worker verify: `std::fs::read_dir(&repo_root)?` iter `DirEntry` → `entry.file_name()` → `to_string_lossy().starts_with(".env")` → `entry.file_type()?.is_file()`. SOUND probe khi cargo build clean. | ✅ `std::fs::read_dir` stable since Rust 1.0; `DirEntry::file_name()` + `file_type()` API confirmed. `cargo build` clean. Strategy sound. |
| 5 | `regex::Regex::new(...)` SOUND oracle cho pattern syntax — 5 vendor pattern compile clean tại module-level (e.g., `once_cell::Lazy<Regex>` HOẶC `LazyLock` Rust 1.80+). Worker chọn pattern: hardcode literal slice `&[(&str, &str)]` (name, regex_src) → build `Vec<(String, Regex)>` once trong `run`. `[unverified — regex 1.x API]` | `cargo build` + unit test "all 5 patterns compile". Pattern fail compile → STOP, AskUserQuestion. | ✅ All 9 regex patterns compile clean (Python regex probe + `cargo build` clean). Fixtures match patterns (#3 ghp_+36A, #4 sk-ant-+32A, #5 AKIA+16 alphanum). No false positive on `sk-proj-` (dash in body breaks `[A-Za-z0-9]{32,}`). No sk-ant/openai overlap. `⚠️ NOTE`: phiếu V1 comment "(catch sk-proj- prefix)" was inaccurate — fixed in V2 (see O1.1 below). |

**Anchor #1+#2 quyết định scope:** #2 ✅ → Worker KHÔNG đụng `src/main.rs`, drop khỏi edit_allow (mirror P002/P003). #2 ❌ → Worker wire dispatch trước implement.

**Anchor #4 fallback:** Nếu `read_dir` không đủ (e.g., `.env` nằm subfolder repo riêng — chưa case study), Worker upgrade `walkdir::WalkDir::new(repo_root).max_depth(1)` — Cargo.toml đã có walkdir. Log Discovery quyết định.

**Anchor #5 oracle:** `cargo build` SOUND cho "regex syntax valid". Pattern compile fail → Worker fix regex literal HOẶC switch crate (vd `fancy-regex`) — log Discovery, KHÔNG cần Architect RESPOND.

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns.

**Phiếu version:** V2 (Architect RESPOND Turn 1)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**
- Anchor #1: ✅ skeleton struct + fn run + todo!() all present at expected lines
- Anchor #2: ✅ main.rs L40 + L53 wired — Worker drops `src/main.rs` from edit_allow
- Anchor #3: ✅ already verified in phiếu draft
- Anchor #4: ✅ `std::fs::read_dir` + `DirEntry` API stable; `cargo build` clean confirms
- Anchor #5: ✅ all 9 patterns compile; fixtures match; no false positive/overlap surprises — with one notation below

**Objections:**

- [O1.1] Phiếu Context §4 OpenAI pattern says `sk-[A-Za-z0-9]{32,}` with comment "(catch sk-proj- prefix)" — this comment is factually wrong. `sk-proj-AbCdEfGh…` has a dash (`-`) in the body after `sk-`, so the `[A-Za-z0-9]{32,}` quantifier fails at the first `-`. `sk-proj-*` tokens are silently NOT caught.
  - Code reference: `docs/ticket/P004-runtime-scan.md` line 33 (pattern definition with misleading comment)
  - Claim: does `sk-[A-Za-z0-9]{32,}` match `sk-proj-<body>` format OpenAI tokens?
  - Oracle: `re.search(r"sk-[A-Za-z0-9]{32,}", "sk-proj-AbCdEf...")` = None — SOUND (regex deterministic)
  - Soundness: SOUND for the claim "does this pattern catch sk-proj-?"
  - Verdict: NOT self-closed — this is a [design] question, not purely mechanical. Two paths:
    - A. (Recommended) Delete the comment "(catch sk-proj- prefix)" — accept sk-proj- as out-of-scope per SOUL line 3 (PARTIAL → Python pilot). Pattern stays `sk-[A-Za-z0-9]{32,}`, comment fixed to "(classic API keys only; sk-proj- not caught — deferred)". No code logic change.
    - B. Extend pattern to `sk-(?:proj-)?[A-Za-z0-9]{32,}` OR `sk-[A-Za-z0-9-]{32,}` — broader but risks false positives (any `sk-` + mixed alnum/dash content). This is a new pattern variant, requires Architect sign-off.
  - Worker recommends A — aligns with constraint #1 (5 vendor cố định, KHÔNG add/extend this sprint) and SOUL line 3.

**Proposed alternatives:**
- A. Fix comment only: remove "(catch sk-proj- prefix)", add "(classic sk- API keys; sk-proj- deferred to Python pilot vòng 2)". Pattern logic unchanged. Zero code risk. (Worker lean — matches SOUL + constraint #1)
- B. Extend openai pattern to cover sk-proj- variant. Requires Architect to confirm scope expansion + new test fixture.

**Status:** ✅ RESOLVED (see Architect Response below)

### Turn 1 — Architect Response (phiếu V2)

- [O1.1] → **ACCEPT (Option A)** — comment factually wrong, fix only. Behavior unchanged.
  - **Reason 1:** SOUL line 3 hard rule "KHÔNG NUỐT PILOT THÍ NGHIỆM" — defer PARTIAL detection to Python pilot vòng 2. sk-proj- format detection is PARTIAL (OpenAI token format đang evolve, đã thay đổi nhiều lần). Mở regex giờ = nuốt pilot.
  - **Reason 2:** Constraint #1 phiếu V1 đã ghi "5 vendor whitelist literal, KHÔNG generic". Comment "(catch sk-proj-)" mâu thuẫn chính phiếu. Comment sai, không phải pattern sai.
  - **Reason 3:** Zero scope cost — chỉ sửa comment. Zero LOC behavior change. Zero test fixture change.
  - **Action taken:** Edited Context §4 OpenAI line — comment đổi thành `(legacy sk-* format only; sk-proj- defer Python pilot vòng 2)`. Anchor #5 result note cập nhật reference "V1 comment was inaccurate — fixed in V2".
  - **Routing per v2.2 §2:** Originally flagged `[design]` by Worker (correctly — wording choice ≠ purely mechanical). Architect respond required, oracle self-close không applicable.

**Status:** ✅ RESPONDED — phiếu bumped to V2

### Final consensus

- Phiếu version: **V2**
- Total turns: 1
- Resolution: O1.1 ACCEPT Option A (comment fix, zero behavior change)
- Approved by Quản đốc (Sếp delegated): pending APPROVAL gate

---

## Nhiệm vụ

### Task 1: Define patterns + helper struct

**File:** `src/cli/runtime_scan.rs`

**Tìm:** Top of file (use statements) + body `pub fn run` chứa `todo!()` (anchor #1).

**Thay bằng / Thêm:**
1. `use anyhow::{Result, Context};` + `use regex::Regex;` + std use (`PathBuf`, `Path`, `read_to_string`, `read_dir`).
2. Const `PATTERNS: &[(&str, &str)] = &[ ("github_classic", r"ghp_[A-Za-z0-9]{36,}"), ("github_oauth", r"gho_[A-Za-z0-9]{36,}"), ("github_user", r"ghu_[A-Za-z0-9]{36,}"), ("github_server", r"ghs_[A-Za-z0-9]{36,}"), ("github_pat_fine_grained", r"github_pat_[A-Za-z0-9_]{82,}"), ("aws_access_key", r"AKIA[0-9A-Z]{16}"), ("aws_secret_key", r"(?i)aws_secret_access_key\s*=\s*[A-Za-z0-9/+=]{40}"), ("openai_key", r"sk-[A-Za-z0-9]{32,}"), ("anthropic_key", r"sk-ant-[A-Za-z0-9_-]{32,}") ];` — TỔNG 9 regex spanning 5 vendor.
3. Helper `fn compile_patterns() -> Result<Vec<(&'static str, Regex)>>`: map PATTERNS → `Regex::new(src).with_context(|| format!("compile regex {name}"))`. Pattern fail → bubble error.
4. Struct `Leak { path: String, line: usize, pattern: &'static str }` HOẶC tuple `(String, usize, &'static str)` — Worker chọn, format `Display`/`to_string` ra `<path>:<line>: <pattern>`.

**Lưu ý:** `sk-[A-Za-z0-9]{32,}` overlap với `sk-ant-…` (Anthropic). Worker decide: dedupe by pattern_name match priority (Anthropic check trước OpenAI) HOẶC chấp nhận 2 hit cùng line (output duplicates — vẫn SOUND, KHÔNG miss). Recommend latter — đơn giản, không loss.

### Task 2: Collect target file list

**File:** `src/cli/runtime_scan.rs`

**Tìm:** Sau Task 1 helper.

**Thay bằng / Thêm:** Helper `fn collect_targets(repo_root: &Path, include_home: bool) -> Result<Vec<PathBuf>>`:
1. Push `repo_root.join(".git/config")` và `repo_root.join(".mcp.json")` (vô điều kiện — `Path::exists` check riêng ở Task 3 scan loop).
2. Loop `read_dir(repo_root)?` (`Err` → `with_context("read_dir repo root")`): cho mỗi entry `Ok` → `name = entry.file_name(); name_str = name.to_string_lossy();` → nếu `name_str.starts_with(".env")` và `entry.file_type()?.is_file()` → push `entry.path()`. Entry `Err` → log eprintln warning + skip (anti-fragile).
3. Nếu `include_home`: `let home = std::env::var("HOME").context("HOME env required for --include-home")?;` → push `PathBuf::from(&home).join(".ssh/config")`, `.gitconfig`, `.netrc`.
4. Return Vec.

**Lưu ý:** `read_dir` trả về cả files + dirs — `is_file()` filter mandatory. Symlink `.env` → `is_file()` follow theo std behavior (KHÔNG đặc biệt xử lý — SOUND mặc định).

### Task 3: Scan + collect + exit

**File:** `src/cli/runtime_scan.rs`

**Tìm:** Sau Task 2 helper, trong `pub fn run`.

**Thay bằng / Thêm:**
1. `let repo_root = args.repo.unwrap_or_else(|| PathBuf::from("."));`
2. `let patterns = compile_patterns()?;`
3. `let targets = collect_targets(&repo_root, args.include_home)?;`
4. `let mut leaks: Vec<String> = Vec::new();`
5. Loop `for target in &targets`: `!target.exists()` → continue. Exist → `let content = read_to_string(target).with_context(|| format!("read {}", target.display()))?;` → loop `for (idx, line) in content.lines().enumerate()`: loop `for (name, re) in &patterns`: `re.is_match(line)` → `leaks.push(format!("{}:{}: {}", target.display(), idx + 1, name));`.
6. Nếu `leaks.is_empty()` → return `Ok(())` (exit 0).
7. Else: `for leak in &leaks { eprintln!("{leak}"); }` → return `Err(anyhow::anyhow!("runtime-scan: {} leak(s) found", leaks.len()))` (main.rs dispatch maps to exit 1).

**Lưu ý:** `idx + 1` because `enumerate` zero-indexed, output 1-indexed line. KHÔNG print matched substring (sanitize — chỉ path + line + pattern_name). File read error (permission denied, e.g.) → bubble `?` (main.rs exit 2 — config/IO failure, mirror P001/P002/P003 pattern). Skip-on-missing áp dụng `exists()=false`, KHÔNG cho IO error.

### Task 4: Wire main.rs exit code dispatch (CONDITIONAL — chỉ nếu anchor #2 ❌)

**File:** `src/main.rs`

**Tìm:** Clap `Commands` enum + match dispatch (Worker grep anchor #2 trước).

**Thay bằng:** Nếu chưa wire, thêm `RuntimeScan(cli::runtime_scan::Args)` variant + arm. Exit mapping: Ok → 0; Err với message chứa "leak(s) found" → 1; Err khác (IO/regex compile) → 2. Pattern khớp P001/P002/P003 (anyhow substring downcast HOẶC enum — Worker giữ nguyên cách 3 phiếu trước).

**Lưu ý:** Anchor #2 ✅ → SKIP, KHÔNG edit `src/main.rs`, log Discovery.

### Task 5: Integration test fixture

**File:** `tests/runtime_scan_test.rs` (new)

**Thay bằng / Thêm:** ≥ 5 case dùng `tempfile::tempdir()` workspace + `std::fs::write` (tự tạo `.git/`, `.env`, `.mcp.json` con trong tmpdir), gọi `assert_cmd::Command::cargo_bin("doctor").args(["runtime-scan", "--repo", tmp_path])`:

1. `empty_workspace_clean` — tmpdir trống → exit 0 (mọi target không exist, skip).
2. `clean_env_no_secrets` — `.env` content `FOO=bar\nBAZ=qux\n` → exit 0.
3. `github_token_detected` — `.env` content `GITHUB_TOKEN=ghp_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n` (36 chữ A — fake match `ghp_*` pattern) → exit 1, stderr chứa `.env:1:` + `github_classic`.
4. `mcp_json_anthropic_key` — `.mcp.json` content `{"key":"sk-ant-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}` (32 char) → exit 1, stderr chứa `.mcp.json:1:` + `anthropic_key`.
5. `aws_access_key_in_git_config` — tạo `.git/config` content `aws = AKIAABCDEFGHIJKLMNOP\n` (AKIA + 16 upper alphanum) → exit 1, stderr chứa `aws_access_key`.
6. `stderr_no_secret_content` — verify exit 1 case stderr KHÔNG chứa substring `AAAAAAAA…` (i.e., chỉ path:line:pattern_name, không leak match text).
7. (Optional) `include_home_flag_off_by_default` — không pass `--include-home` → KHÔNG scan home (skip — test khó verify negative, có thể defer).

**Lưu ý:** Fake match dùng pattern alphabet `A` repeat đủ length min để regex match (e.g., `ghp_` + 36 ký tự `A` = `ghp_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA`). KHÔNG hardcode token thật. Pattern khớp `tests/rotate_check_test.rs` (precedent P003) — `tempfile::tempdir()` + `Command::cargo_bin("doctor")` + `predicates::str::contains`.

---

## Edit scope (v2.2 §5)

```yaml
edit_allow:
  - src/cli/runtime_scan.rs
  - src/main.rs                 # ONLY if anchor #2 ❌
  - tests/runtime_scan_test.rs

verify_read:
  - src/cli/runtime_scan.rs
  - src/cli/rotate_check.rs     # reference P003 pattern (config + exit dispatch)
  - src/cli/validate_map.rs     # reference P002 pattern
  - src/cli/lane_check.rs       # reference P001 pattern
  - src/main.rs
  - src/cli/mod.rs
  - Cargo.toml
  - docs/ARCHITECTURE.md
  - docs/ticket/P003-rotate-check.md
  - tests/rotate_check_test.rs

contract_tests:
  - tests/runtime_scan_test.rs
```

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/cli/runtime_scan.rs` | Task 1-3: patterns + target collect + scan loop, replace `todo!()` |
| `src/main.rs` | Task 4 (conditional): wire `RuntimeScan` arm với exit mapping nếu chưa wire |
| `tests/runtime_scan_test.rs` | Task 5: integration test 5-7 case dùng tempfile workspace + fake tokens |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `Cargo.toml` | `regex` + `walkdir` đã có (anchor #3) — KHÔNG add dep |
| `src/cli/mod.rs` | `pub mod runtime_scan;` đã expose — Worker grep confirm, KHÔNG edit |
| `src/cli/lane_check.rs` | P001 không regression (`cargo test --test lane_check_test` pass) |
| `src/cli/validate_map.rs` | P002 không regression (`cargo test --test validate_map_test` pass) |
| `src/cli/rotate_check.rs` | P003 không regression (`cargo test --test rotate_check_test` pass) |

---

## Luật chơi (Constraints)

1. **SOUND only — patterns whitelist 5 vendor cố định** (GitHub / AWS access / AWS secret / OpenAI / Anthropic, 9 regex tổng). KHÔNG add vendor mới trong phiếu này. KHÔNG generic high-entropy/Shannon-entropy detection (PARTIAL — Python pilot vòng 2 chịu trách nhiệm). OpenAI pattern `sk-[A-Za-z0-9]{32,}` chỉ catch legacy sk-* format; sk-proj-* format defer Python pilot vòng 2 (per V2 O1.1 resolution).
2. **`--include-home` DEFAULT FALSE** — Sub-mech F: scan home dir là privacy-sensitive, phải explicit opt-in. Clap derive `#[arg(long)] pub include_home: bool` (bool flag tự default false).
3. **Output sanitize** — format `<path>:<line>: <pattern_name>` only. KHÔNG print matched substring xuống stderr/stdout/log (secret không leak xuống CI/log). Test #6 enforce.
4. **File missing = OK skip, KHÔNG drift** — `.git/config` / `.mcp.json` / `.env*` / home files không tồn tại → continue, KHÔNG error. Khớp P003 file-missing skip rule.
5. **Tổng LOC ≤ 200** cho 2 file thực sự sửa (`src/cli/runtime_scan.rs` + `tests/runtime_scan_test.rs`). Tầng 2 cap. Vượt → Worker STOP, AskUserQuestion.

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean, no new warnings.
- [ ] `cargo test --test runtime_scan_test` — 5-7 case pass.
- [ ] `cargo test` full suite — 19/19 existing pass + new case = không regression P001+P002+P003.

### Manual Testing
- [ ] `cargo run -- runtime-scan --repo .` trong repo doctor → exit 0 (no leak — verify clean state baseline).
- [ ] Fixture `.env` chứa `ghp_AAAA…` (36 A) → exit 1 stderr `.env:1: github_classic`.
- [ ] Fixture `.mcp.json` chứa `sk-ant-AAAA…` (32 char) → exit 1 stderr `anthropic_key`.
- [ ] Fixture `.git/config` chứa `AKIAABCDEFGHIJKLMNOP` → exit 1 stderr `aws_access_key`.
- [ ] Stderr KHÔNG chứa substring secret value (sanitize verify).
- [ ] `cargo run -- runtime-scan --repo . --include-home` → exit code phụ thuộc home state (manual chấp nhận).

### Regression
- [ ] `cargo test --test lane_check_test` — P001 pass.
- [ ] `cargo test --test validate_map_test` — P002 pass.
- [ ] `cargo test --test rotate_check_test` — P003 pass.
- [ ] `doctor --help` show 4 subcmd (lane-check, validate-map, rotate-check, runtime-scan).
- [ ] `doctor runtime-scan --help` show `--repo` + `--include-home`.

### Docs Gate
- [ ] `docs/CHANGELOG.md` — thêm entry: `P004: runtime-scan subcmd (Sub-mech F token leak — 5 vendor pattern, --include-home opt-in, exit 0/1, output sanitize)`. (File CHANGELOG.md hiện chưa tồn tại — Worker tạo nếu thiếu, khớp P003 case.)
- [ ] `docs/ARCHITECTURE.md` L76-91 — Worker verify khớp contract; update inline comment nếu drift (final pattern list, target list logic).

### Discovery Report
- [ ] Write `docs/discoveries/P004.md`:
  - Anchor #1-#5 result (#2 wire decision, #4 glob strategy chốt, #5 regex compile oracle)
  - Edit scope: `src/main.rs` touched? Y/N + lý do
  - sk-* vs sk-ant-* overlap handling (Worker choice — dedupe priority hay duplicate output)
  - `.env*` glob impl chốt (read_dir vs walkdir depth 1)
  - `--include-home` flag — default false rationale (privacy)
  - Output sanitize — verify test #6 pass
  - V2 RESPOND note: O1.1 ACCEPT Option A — sk-proj- format deferred per SOUL line 3; OpenAI pattern catches legacy sk-* only
  - 3-field SOUND oracle:
    ```
    Claim: runtime-scan enforces Sub-mech F token leak detection per §7
    Oracle: cargo test fixture (5+ case, fake tokens matching 5 vendor pattern) + regex::Regex::new compile
    Soundness: SOUND for whitelisted 5 vendor (legacy sk-* OpenAI only); PARTIAL for generic entropy + sk-proj- variant (deferred Python pilot vòng 2 per SOUL line 3)
    ```
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`.
