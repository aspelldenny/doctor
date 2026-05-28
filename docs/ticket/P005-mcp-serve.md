# PHIẾU P005: `doctor serve` MCP mode (4 tools)

> **Loại:** Feature
> **Ưu tiên:** P1 (MVP wrap — phiếu cuối sprint)
> **Tầng:** 1 (móng nhà — cross-cutting refactor 4 subcmd signature + new MCP server surface + new module `src/mcp/`)
> **Lane:** Guarded (cap relax — full quyền cho refactor + new server surface; phiếu vẫn cố ≤ 400 dòng)
> **Ảnh hưởng:** `src/cli/lane_check.rs`, `src/cli/validate_map.rs`, `src/cli/rotate_check.rs`, `src/cli/runtime_scan.rs`, `src/main.rs`, `src/mcp/mod.rs` (new), `src/mcp/tools.rs` (new — optional split), `tests/mcp_serve_test.rs` (new)
> **Dependency:** P001 (a4fddd4), P002 (9bde634), P003 (7c1f9ef), P004 (c34048a) all shipped. 27/27 tests pass baseline.

---

## Context

### Vấn đề hiện tại

MVP sprint cuối — 4 CLI subcmd đã ship dùng nội bộ qua shell. Để orchestrator (main Claude session) gọi doctor như tool 1st-class (không spawn subprocess + parse stderr), cần MCP stdio JSON-RPC 2.0 server. Pattern precedent: `~/advisory-inbox` `serve` subcmd ship 3 ngày trước theo `CLAUDE.md ## Related repos` (rmcp 1.7.0, stdio transport, `#[tool_router]` macro, `ServerHandler` impl two-step pattern).

Hiện 4 subcmd hardcode CLI side effect:
- `pub fn run(args) -> Result<()>` — return Ok/Err, main.rs map Err → `std::process::exit(N)` qua anyhow downcast/substring (per P001-P004 precedent).
- Stdout/stderr write trực tiếp qua `println!`/`eprintln!`.

MCP wrapper cần result **không** exit, cần capture stdout/stderr thành String để format JSON-RPC tool response. Re-implement logic trong MCP layer = duplicate code + drift risk. **Refactor:** mỗi subcmd expose function trả tuple `(exit_code, captured_output)`; CLI thin wrapper gọi function + exit; MCP wrapper gọi function + format `Content::text` response + `is_error` flag từ exit code.

### Giải pháp

1. **Refactor 4 subcmd signature** (`pub fn run`) → tách logic thuần khỏi CLI side effect:
   - Strategy B (per ARCHITECTURE.md L93-100): mỗi `src/cli/<name>.rs` thêm `pub fn execute(args: Args) -> Result<RunOutput>` trả struct `RunOutput { exit_code: u8, stdout: String, stderr: String }`.
   - Existing `pub fn run(args: Args) -> Result<()>` giữ nguyên public signature, internal refactor → gọi `execute(args)?` + print stdout/stderr + map exit_code qua anyhow Err (mirror P001-P004 main.rs dispatch pattern).
   - **Backward compat:** CLI behavior unchanged — main.rs vẫn dispatch như cũ, exit code 0/1/2 mapping giữ nguyên. 27/27 existing test PHẢI pass không sửa.

2. **New module `src/mcp/`:**
   - `src/mcp/mod.rs` — module export + `DoctorServer` struct + tool input structs với `#[derive(JsonSchema, Deserialize)]`.
   - `src/mcp/tools.rs` (optional split — Worker decide layout): `#[tool_router]` impl block expose 4 tool:
     - `lane_check(LaneCheckInput { ticket: String }) -> CallToolResult`
     - `validate_map(ValidateMapInput { map: String }) -> CallToolResult`
     - `rotate_check(RotateCheckInput { repo: Option<String> }) -> CallToolResult`
     - `runtime_scan(RuntimeScanInput { repo: Option<String>, include_home: Option<bool> }) -> CallToolResult`
   - Mỗi tool fn: convert input string → `PathBuf` → build `<subcmd>::Args` struct → call `<subcmd>::execute(args)` → format `RunOutput` thành `CallToolResult { content: vec![Content::text(stdout + stderr)], is_error: Some(exit_code != 0) }`.
   - `#[tool_handler]` on `ServerHandler` impl với two-step pattern (advisory-inbox P011 retro: avoid `from_build_env` reading rmcp crate name).

3. **Refactor `Serve` Cli variant dispatch body trong `src/main.rs`:**
   - `Serve` variant ĐÃ tồn tại trong `Commands` enum (skeleton stub at `src/main.rs:43` — confirmed by Worker Turn 1 anchor #2). Stub hiện `eprintln!("MCP serve mode — phiếu P005 (deferred)") + exit(2)`.
   - **Task 3 = REPLACE stub body** với tokio runtime spawn + `mcp::serve().await`. KHÔNG add new variant.
   - Tokio runtime: manual `tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(...)` (advisory-inbox precedent: rt feature only, no rt-multi-thread per ARCHITECTURE.md L110 — confirmed by Worker anchor #7).

4. **Schemars derive cho input struct** — `#[derive(JsonSchema, Deserialize)]` produce clean JSON schema cho MCP `tools/list`. Tránh nested anyhow Result trong field.

5. **Test scaffold** — `tests/mcp_serve_test.rs` smoke only (2 case):
   - `tools_list_returns_4_tools` — spawn `doctor serve` qua `tokio::process::Command`, write JSON-RPC initialize + `tools/list` request qua stdin, parse JSON-RPC response, assert tool names = `{lane_check, validate_map, rotate_check, runtime_scan}`.
   - `lane_check_tool_call_happy_path` — spawn server, send `tools/call` cho `lane_check` với tmp ticket fixture (Normal lane, 10 dòng), assert response `is_error: false`.
   - End-to-end test 3 tool còn lại = oracle cost cao → defer dogfood (constraint #3).

### Scope

- CHỈ sửa: 4 file `src/cli/*.rs`, `src/main.rs`, `src/mcp/mod.rs` (new), `src/mcp/tools.rs` (new, optional split), `tests/mcp_serve_test.rs` (new).
- ĐỘNG `Cargo.toml` CHỈ KHI Worker verify dep cần thêm chưa có — anchor #1 confirm `rmcp 1.7.0` + `schemars 1.0` + `tokio` đã có. Dự kiến SKIP.
- KHÔNG sửa: `Cargo.toml` (per anchor #1), `src/cli/mod.rs` (chỉ Worker grep verify `pub mod` clean), `docs/` (Worker update CHANGELOG/ARCHITECTURE post-execute), 4 existing test file (regression PHẢI pass nguyên).

---

## Task 0 — Verification Anchors

| # | Assumption | Verify by | Result |
|---|-----------|-----------|--------|
| 1 | `Cargo.toml` đã có `rmcp = { version = "1.7.0", features = ["server", "transport-io", "macros", "schemars"] }` (L27), `schemars = "1.0"` (L28), `tokio = { version = "1", features = ["rt", "macros", "io-std"] }` (L20). KHÔNG cần add dep mới `[verified — Cargo.toml Read L14-28]` | `grep -nE "^(rmcp\|schemars\|tokio) = " /Users/nguyenhuuanh/doctor/Cargo.toml` | ✅ |
| 2 | `src/main.rs` `Commands` enum đã có `Serve` variant pre-wired stub (skeleton commit 84cbb7b). Task 3 = REFACTOR dispatch body, KHÔNG add variant `[verified by Worker Turn 1 — src/main.rs:43 stub exit(2)]` | `grep -nE "Serve\|Commands\|enum Commands" /Users/nguyenhuuanh/doctor/src/main.rs` | ✅ Serve variant ĐÃ tồn tại (src/main.rs:43) as stub. Task 3 scope = refactor dispatch body only. |
| 3 | 4 subcmd hiện expose `pub fn run(args: Args) -> Result<()>` (per P001-P004 precedent + ARCHITECTURE.md L34/L50/L67/L84). Function chỉ in stdout/stderr + return Ok/Err — KHÔNG return struct `[needs Worker verify — đọc 4 file src/cli/*.rs]` | `grep -nE "pub fn run\|pub fn execute" /Users/nguyenhuuanh/doctor/src/cli/lane_check.rs /Users/nguyenhuuanh/doctor/src/cli/validate_map.rs /Users/nguyenhuuanh/doctor/src/cli/rotate_check.rs /Users/nguyenhuuanh/doctor/src/cli/runtime_scan.rs` | ✅ |
| 4 | 4 subcmd `pub fn run` body dùng `println!`/`eprintln!` trực tiếp (chứ không return String). Refactor sang `execute` trả `RunOutput { stdout, stderr }` cần capture thay print `[needs Worker verify — pattern khớp anyhow Err substring downcast P001-P004]` | `grep -nE "println!\|eprintln!" /Users/nguyenhuuanh/doctor/src/cli/lane_check.rs /Users/nguyenhuuanh/doctor/src/cli/validate_map.rs /Users/nguyenhuuanh/doctor/src/cli/rotate_check.rs /Users/nguyenhuuanh/doctor/src/cli/runtime_scan.rs` | ✅ |
| 5 | rmcp 1.7.0 API: `#[tool_router]` macro + `#[tool(...)]` per-fn attribute + `#[tool_handler]` on `ServerHandler` impl. `CallToolResult` có field `content: Vec<Content>` + `is_error: Option<bool>`. **Tool fns là SYNC, không async** (per advisory-inbox `src/mcp/tools.rs:176-398`). `[verified V2 — advisory-inbox precedent CLAUDE.md ## Related repos]` | Worker verify: open `~/advisory-inbox/src/mcp/` (precedent crate) HOẶC `cargo doc --package rmcp --open` offline. Confirm macro signature + tool fn return type. | ✅ V2: tool fns SYNC `fn lane_check(&self, params: Parameters<T>) -> Result<CallToolResult, ErrorData>` per advisory-inbox. |
| 6 | schemars 1.0 API compat với rmcp 1.7.0 macro: `#[derive(JsonSchema, Deserialize)]` on input struct → rmcp tool macro pickup schema clean (no manual JsonSchema impl needed) `[unverified — Cargo.toml L27 rmcp feature "schemars" implies bridge bundled]` | `cargo build --release` clean sau khi derive on tool input struct. Pattern fail → STOP, AskUserQuestion (downgrade schemars 0.8 hay nâng rmcp). | ⏳ SOUND oracle defer EXECUTE (compile verify) |
| 7 | Tokio runtime feature `rt + macros + io-std` (Cargo.toml L20) đủ cho rmcp stdio transport. KHÔNG cần `rt-multi-thread` (per ARCHITECTURE.md L110 + advisory-inbox precedent) `[verified — Cargo.toml Read L20, ARCHITECTURE.md L110]` | Worker verify: build server với `#[tokio::main(flavor = "current_thread")]` HOẶC manual `Builder::new_current_thread()` — compile clean. | ✅ advisory-inbox serve.rs L17-38 confirm Option B pattern exact. |

**Anchor #1 quyết định dep scope:** ✅ → `Cargo.toml` drop khỏi edit_allow (KHÔNG động). ❌ → STOP, AskUserQuestion (skeleton commit drift — orchestrator phải re-sync).

**Anchor #2 quyết định main.rs scope:** ✅ Serve ĐÃ wire stub → Task 3 = REFACTOR dispatch body (replace stub with rmcp stdio server spawn). KHÔNG add new variant.

**Anchor #3+#4 quyết định refactor cost:** ✅ (4 subcmd cùng pattern `pub fn run`) → Strategy B execute extraction áp đều. ❌ (signature đã khác kỳ vọng) → STOP, AskUserQuestion (precedent drift).

**Anchor #5 oracle:** `cargo build` SOUND cho "rmcp macro compile clean". Pattern fail → Worker đọc advisory-inbox source (precedent), match macro idiom, KHÔNG cần Architect RESPOND. Log Discovery với rmcp API quirk gặp phải.

**Anchor #6 oracle:** `cargo build` SOUND cho "schemars derive + rmcp pickup". Fail → downgrade schemars version HOẶC manual `JsonSchema` impl, log Discovery.

**Anchor #7 oracle:** `cargo build` SOUND cho "tokio rt enough for stdio transport".

---

## Debate Log

> Schema: 1 turn = 1 cặp Worker Challenge + Architect Response. Cap = 3 turns. Guarded lane → CHALLENGE MANDATORY (Tầng 1 phiếu, cross-cutting refactor).

**Phiếu version:** V2 (Architect Response Turn 1 applied)

### Turn 1 — Worker Challenge (phiếu V1)

**Anchor verification:**

| # | Result | Notes |
|---|--------|-------|
| 1 | ✅ | `rmcp = { version = "1.7.0", features = ["server", "transport-io", "macros", "schemars"] }` L27, `schemars = "1.0"` L28, tokio L20. Cargo.toml KHÔNG cần động. |
| 2 | ⚠️ | `Serve` variant ĐÃ tồn tại trong `src/main.rs` L43 — skeleton commit 84cbb7b đã pre-wire stub: `eprintln!("MCP serve mode — phiếu P005 (deferred)")` + `exit(2)`. Phiếu nói "dự kiến chưa wire" nhưng code thực tế khác. Task 3 cần REFACTOR dispatch arm body, KHÔNG add new variant. |
| 3 | ✅ | 4 subcmd đều expose `pub fn run(args: Args) -> Result<()>`. Không có `pub fn execute`. Pattern đồng nhất, Strategy B execute extraction áp đều được. |
| 4 | ✅ | `println!`/`eprintln!` xác nhận trong cả 4 file. Refactor sang `RunOutput` buffer cần thiết. |
| 5 | ⚠️ | rmcp 1.7.0 API verified qua advisory-inbox precedent (src L167-399). **2 divergence tìm thấy — xem objections bên dưới.** |
| 6 | ⏳ | Schemars 1.0 + rmcp `#[derive(JsonSchema, Deserialize)]` pattern khớp advisory-inbox L51-153. Compile verify = SOUND oracle → defer to EXECUTE `cargo build`. |
| 7 | ✅ | advisory-inbox `src/cli/serve.rs` L17-38 confirm: `Builder::new_current_thread().enable_all().build()?.block_on(...)` — NO `#[tokio::main]` on main fn. Option B chính xác. |

**Objections (Tầng 1 only):**

- [O1.1] **Tool fn trong Task 2 code snippet dùng `async fn` — sai so với rmcp 1.7.0 precedent.**
  Phiếu Task 2 snippet (L215-219) viết `async fn lane_check(&self, ...) -> Result<CallToolResult, rmcp::ErrorData>`. Advisory-inbox precedent `~/advisory-inbox/src/mcp/tools.rs:176-398`: tất cả 6 tool fn là **sync** (không có `async fn`). Rmcp `#[tool_router]` macro không đòi async.
  Claim: rmcp 1.7.0 `#[tool_router]` tool fns phải là async
  Oracle: advisory-inbox compile thành công với 6 sync tool fns + `#[tool_router]` (SOUND precedent) + `cargo build` SOUND
  Soundness: SOUND — precedent + compiler phán đúng claim
  Verdict: **self-closed** — tool fns PHẢI là sync fn (matching advisory-inbox). EXECUTE mode sẽ viết sync fns, không async.

- [O1.2] **Tool fn return type trong phiếu (`Result<CallToolResult, rmcp::ErrorData>`) khác advisory-inbox (`Result<Json<OutputType>, ErrorData>`).**
  Phiếu Task 2 snippet trả `CallToolResult` trực tiếp. Advisory-inbox trả `Result<Json<ParseReportOutput>, ErrorData>` (structured output wrapper). Doctor tools đơn giản hơn (text-only output, không cần structured JSON output type), nên `CallToolResult::success(vec![Content::text(...)])` trả trực tiếp là hợp lệ — nhưng đây là design choice ảnh hưởng MCP tool output shape.
  Claim: doctor tool fns nên trả `CallToolResult` trực tiếp (text-only) hay wrap qua `Json<T>` (structured)
  Oracle: KHÔNG có SOUND oracle — rmcp cho phép cả hai pattern (compiler câm về design). `[design]` — không self-close.
  Soundness: NONE for "which pattern is correct for doctor"
  Verdict: **cần Architect respond** — hai options:
  - **A. (Worker recommend):** Trả `CallToolResult` trực tiếp via `CallToolResult::success(vec![Content::text(combined_output)])` hoặc `CallToolResult::error(...)`. DRY, không cần define 4 output structs, phù hợp doctor là text-output tool. Precedent: phiếu constraint #2 đã nói "format `CallToolResult { content: vec![Content::text(...)], is_error: ... }`".
  - **B.** Trả `Result<Json<RunOutput>, ErrorData>` — structured, cho phép client parse `exit_code`/`stdout`/`stderr` riêng. Overhead: 4 output structs + schemars derive cho output, không có trong phiếu scope hiện tại.

**Status:** ✅ RESPONDED — Architect Turn 1 below.

### Turn 1 — Architect Response (phiếu V2)

- **[O1.1] → ACCEPT** — rmcp 1.7.0 tool fns are sync (advisory-inbox `src/mcp/tools.rs:176-398` precedent). Snippet fixed in V2: `fn lane_check(&self, params: Parameters<LaneCheckInput>) -> Result<CallToolResult, ErrorData>` (no `async`). Anchor #5 result column updated to ✅.

- **[O1.2] → ACCEPT Option A** — `CallToolResult` direct, no extra output structs. Reasons:
  1. Match phiếu constraint #2 V1 đã ghi (`Content::text(stdout+stderr) + is_error flag`).
  2. Doctor là CLI gate — output là text stdout/stderr, không phải structured data. JSON output struct extra layer không giúp Claude session reason tốt hơn (orchestrator parse text như nó parse stderr CLI hiện tại).
  3. DRY — 4 subcmd hiện `run() -> Result<()>` sau refactor → `execute() -> Result<RunOutput>` map 1-1 sang `CallToolResult` với `Content::text(stdout+stderr)` + `is_error = (exit_code != 0)`. Zero extra struct.
  4. Edit_allow scope giữ nguyên — Worker không phải mở rộng.

- **[Anchor #2] → CLARIFY** — `Serve` variant đã tồn tại stub `src/main.rs:43`. Task 3 wording updated V2: "Refactor `Serve` variant dispatch body (replace stub with rmcp stdio server spawn)", KHÔNG "add Serve variant". Anchor #2 Result cột bumped to ✅.

**Status:** ✅ RESPONDED — phiếu bumped to V2. Pending Worker CHALLENGE re-verify HOẶC Sếp APPROVAL gate (Architect response purely mechanical — accept Worker recommendation + fix snippet drift, không introduce new claim).

### Final consensus
- Phiếu version: V2
- Total turns: 1
- Approved by Sếp: [pending APPROVAL gate]

---

## Nhiệm vụ

### Task 1: Refactor 4 subcmd — extract `pub fn execute()` returning `RunOutput`

**File:** `src/cli/lane_check.rs`, `src/cli/validate_map.rs`, `src/cli/rotate_check.rs`, `src/cli/runtime_scan.rs` (all 4)

**Tìm:** Existing `pub fn run(args: Args) -> Result<()>` body trong mỗi file (anchor #3).

**Thay bằng / Thêm:**
1. Define common struct (Worker chọn location — đề xuất `src/cli/mod.rs` HOẶC inline mỗi file with `pub use`):
   ```rust
   #[derive(Debug, Default)]
   pub struct RunOutput {
       pub exit_code: u8,
       pub stdout: String,
       pub stderr: String,
   }
   ```
2. Mỗi subcmd thêm `pub fn execute(args: Args) -> Result<RunOutput>`:
   - Move logic từ `run` body sang `execute`.
   - Mọi `println!(...)` → `writeln!(out.stdout, ...)?` (use `std::fmt::Write`) HOẶC `out.stdout.push_str(&format!(...))`.
   - Mọi `eprintln!(...)` → `writeln!(out.stderr, ...)` tương tự.
   - Exit code logic: thay vì `anyhow!("rotate-check severity {worst}")` substring, set `out.exit_code = N` trực tiếp + return `Ok(out)`. IO/parse error → vẫn `?` bubble `Err`.
3. `pub fn run(args: Args) -> Result<()>` rewrite thành thin wrapper:
   ```rust
   pub fn run(args: Args) -> Result<()> {
       let out = execute(args)?;
       if !out.stdout.is_empty() { print!("{}", out.stdout); }
       if !out.stderr.is_empty() { eprint!("{}", out.stderr); }
       if out.exit_code != 0 {
           // mirror existing anyhow Err pattern P001-P004 để main.rs dispatch giữ nguyên
           anyhow::bail!("<subcmd_name>: exit {}", out.exit_code);
       }
       Ok(())
   }
   ```
4. **Critical:** `src/main.rs` dispatch arm logic (anyhow Err → exit code map) KHÔNG đổi. Existing test 27/27 PHẢI pass không sửa.

**Lưu ý:**
- Worker chọn pattern: (A) inline `pub struct RunOutput` mỗi file with `pub use crate::cli::lane_check::RunOutput as _;` HOẶC (B) shared `pub struct RunOutput` trong `src/cli/mod.rs` re-export. Recommend B (DRY, schema schemars dùng cùng struct nếu cần).
- Capture buffer: `String` đủ cho output ngắn (mỗi subcmd output ≤ vài KB). KHÔNG cần `Vec<u8>` / `Cursor`.
- Worker grep verify 4 file dùng cùng anyhow Err pattern hiện tại trước refactor — nếu lệch (P004 dùng enum, P003 dùng substring), giữ pattern per-file, KHÔNG normalize trong phiếu này.

### Task 2: New module `src/mcp/mod.rs` — server struct + tool router

**File:** `src/mcp/mod.rs` (new file)

**Thay bằng / Thêm:**
1. Module declaration + imports:
   ```rust
   use anyhow::Result;
   use rmcp::{
       handler::server::tool::{Parameters, ToolRouter},
       model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
       tool, tool_handler, tool_router, ErrorData,
       ServerHandler,
   };
   use schemars::JsonSchema;
   use serde::Deserialize;
   use std::path::PathBuf;

   use crate::cli::{lane_check, validate_map, rotate_check, runtime_scan, RunOutput};
   ```
   (Worker verify import path qua anchor #5 — advisory-inbox precedent.)
2. Tool input structs (1 per tool):
   ```rust
   #[derive(Debug, Deserialize, JsonSchema)]
   pub struct LaneCheckInput { pub ticket: String }

   #[derive(Debug, Deserialize, JsonSchema)]
   pub struct ValidateMapInput { pub map: String }

   #[derive(Debug, Deserialize, JsonSchema)]
   pub struct RotateCheckInput { #[serde(default)] pub repo: Option<String> }

   #[derive(Debug, Deserialize, JsonSchema)]
   pub struct RuntimeScanInput {
       #[serde(default)] pub repo: Option<String>,
       #[serde(default)] pub include_home: Option<bool>,
   }
   ```
3. `DoctorServer` struct + tool router impl. **Tool fns SYNC** (V2 — per advisory-inbox precedent `src/mcp/tools.rs:176-398`, NO `async`):
   ```rust
   #[derive(Clone)]
   pub struct DoctorServer {
       tool_router: ToolRouter<Self>,
   }

   #[tool_router]
   impl DoctorServer {
       pub fn new() -> Self {
           Self { tool_router: Self::tool_router() }
       }

       #[tool(description = "Check phiếu lane budget (v2.2 §1)")]
       fn lane_check(&self, params: Parameters<LaneCheckInput>) -> Result<CallToolResult, ErrorData> {
           let args = lane_check::Args { ticket: PathBuf::from(&params.0.ticket) };
           run_to_call_result(lane_check::execute(args))
       }
       // ... 3 tool tương tự (sync fn, không async)
   }
   ```
   Worker verify chính xác signature `Parameters<T>` qua advisory-inbox source — anchor #5.
4. Helper `fn run_to_call_result(result: Result<RunOutput>) -> Result<CallToolResult, ErrorData>` — **SYNC fn, NO async**:
   - `Ok(out)` → `Ok(CallToolResult::success(vec![Content::text(format!("{}{}", out.stdout, out.stderr))]))` then set `is_error = Some(out.exit_code != 0)` if rmcp 1.7.0 API expose mutation HOẶC build literal `CallToolResult { content, is_error: Some(out.exit_code != 0), structured_content: None, meta: None }` (field cụ thể per rmcp 1.7.0 — Worker verify qua advisory-inbox precedent).
   - `Err(e)` → `Err(ErrorData::internal_error(e.to_string(), None))`.
5. `#[tool_handler]` impl on `ServerHandler` với two-step pattern:
   ```rust
   #[tool_handler]
   impl ServerHandler for DoctorServer {
       fn get_info(&self) -> ServerInfo {
           // two-step pattern (avoid from_build_env reading rmcp crate name — advisory-inbox P011 retro)
           let mut info = ServerInfo::default();
           info.server_info.name = "doctor".to_string();
           info.server_info.version = env!("CARGO_PKG_VERSION").to_string();
           info.capabilities = ServerCapabilities::builder().enable_tools().build();
           info
       }
   }
   ```
6. `pub async fn serve() -> Result<()>` (serve fn IS async — only `serve()` + rmcp internal use async; tool fns themselves sync):
   ```rust
   pub async fn serve() -> Result<()> {
       use rmcp::{transport::stdio, ServiceExt};
       let server = DoctorServer::new();
       let service = server.serve(stdio()).await?;
       service.waiting().await?;
       Ok(())
   }
   ```

**Lưu ý:**
- rmcp 1.7.0 API exact signature có thể drift — Worker đọc advisory-inbox source (`~/advisory-inbox/src/mcp/`) trước viết. Mỗi mismatch log Discovery.
- Worker tự decide split file: tất cả trong `mod.rs` (≤ 200 LOC) HOẶC split `mod.rs` (struct + serve) + `tools.rs` (`#[tool_router]` impl). Phiếu chấp nhận cả hai.
- `Parameters<T>` wrapper: rmcp macro expect tool fn nhận `Parameters<InputStruct>` (advisory-inbox precedent confirmed). Worker verify exact import path.
- **Sync vs async clarity (V2 fix):** Tool fns (`fn lane_check`, `fn validate_map`, `fn rotate_check`, `fn runtime_scan`) là SYNC. Helper `run_to_call_result` SYNC. Chỉ `pub async fn serve()` async (rmcp transport).

### Task 3: Refactor `Serve` variant dispatch body trong `src/main.rs`

**File:** `src/main.rs`

**Tìm:** `Commands::Serve` dispatch arm hiện tại — stub `eprintln!("MCP serve mode — phiếu P005 (deferred)") + exit(2)` (skeleton commit 84cbb7b, src/main.rs:43 — anchor #2 ✅).

**Thay bằng / Thêm:**
1. Add module declaration `mod mcp;` cạnh `mod cli;` (top of `src/main.rs`).
2. **Serve variant ĐÃ tồn tại** (skeleton stub) — KHÔNG add new variant. Chỉ refactor dispatch body.
3. Main fn signature: keep sync main, manual tokio runtime block_on chỉ Serve arm (Option B per V1 — confirmed by Worker anchor #7 advisory-inbox `src/cli/serve.rs:17-38` precedent).
4. Dispatch arm — REPLACE stub body:
   ```rust
   Commands::Serve => {
       let rt = tokio::runtime::Builder::new_current_thread()
           .enable_all()
           .build()
           .context("build tokio runtime")?;
       rt.block_on(mcp::serve())
   }
   ```
5. Exit code: Serve return Ok khi shutdown clean (stdin EOF) = exit 0. Err = exit 2 (mirror IO error pattern P001-P004).

**Lưu ý:**
- Worker DOUBLE-CHECK anchor #2 trước edit — confirm `Serve` variant + stub body location. Nếu skeleton drift (variant đổi tên hoặc removed), STOP, AskUserQuestion.
- KHÔNG đổi 4 dispatch arm hiện tại (P001-P004 anyhow Err substring downcast). Existing exit code map giữ nguyên.
- Cargo build clean SOUND check refactor không vỡ.

### Task 4: Test scaffold `tests/mcp_serve_test.rs`

**File:** `tests/mcp_serve_test.rs` (new)

**Thay bằng / Thêm:** 2 case smoke (per constraint #3 — KHÔNG full E2E):

1. `tools_list_returns_4_tools` — spawn server qua `tokio::process::Command::new(env!("CARGO_BIN_EXE_doctor")).arg("serve").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?`. Write JSON-RPC initialize + tools/list request bytes qua stdin. Read response từ stdout (parse line-by-line JSON-RPC). Assert response chứa 4 tool name `lane_check`, `validate_map`, `rotate_check`, `runtime_scan`. Kill process.
2. `lane_check_tool_call_happy_path` — same spawn pattern. Write JSON-RPC tools/call cho `lane_check` với arg `{"ticket": "<tmp ticket Normal lane 10 dòng>"}` (tạo tmpfile qua `tempfile::NamedTempFile` chứa minimal phiếu markdown `> **Lane:** Normal`). Parse response, assert `is_error: false`. Kill process.

JSON-RPC request format (advisory-inbox precedent):
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

**Lưu ý:**
- Use `tokio_test` HOẶC `#[tokio::test(flavor = "current_thread")]` (dev-dep `tokio-test = "0.4"` đã có Cargo.toml L31).
- Timeout 5s per test — server hang → fail fast.
- Drop server (kill child process) cuối mỗi test — KHÔNG leak.
- Pattern khớp advisory-inbox `tests/mcp_*_test.rs` precedent. Worker tham khảo.

---

## Edit scope (v2.2 §5)

```yaml
edit_allow:
  - src/cli/lane_check.rs
  - src/cli/validate_map.rs
  - src/cli/rotate_check.rs
  - src/cli/runtime_scan.rs
  - src/cli/mod.rs              # Task 1: optional add shared RunOutput struct
  - src/main.rs                 # Task 3: refactor Serve dispatch body (variant đã tồn tại)
  - src/mcp/mod.rs              # Task 2: new module
  - src/mcp/tools.rs            # Task 2: optional split (Worker chọn layout)
  - tests/mcp_serve_test.rs     # Task 4: new test
  - Cargo.toml                  # ONLY if anchor #1 ❌ (dự kiến SKIP — rmcp/schemars/tokio đã có)

verify_read:
  - src/cli/lane_check.rs
  - src/cli/validate_map.rs
  - src/cli/rotate_check.rs
  - src/cli/runtime_scan.rs
  - src/cli/mod.rs
  - src/main.rs
  - Cargo.toml
  - docs/ARCHITECTURE.md
  - docs/ticket/P001-lane-check.md
  - docs/ticket/P002-validate-map.md
  - docs/ticket/P003-rotate-check.md
  - docs/ticket/P004-runtime-scan.md
  - ~/advisory-inbox/src/mcp/   # precedent crate — rmcp 1.7.0 API
  - ~/advisory-inbox/Cargo.toml # version compat reference
  - tests/lane_check_test.rs    # regression baseline
  - tests/validate_map_test.rs
  - tests/rotate_check_test.rs
  - tests/runtime_scan_test.rs

contract_tests:
  - tests/mcp_serve_test.rs
  - tests/lane_check_test.rs     # regression — PHẢI pass sau refactor
  - tests/validate_map_test.rs
  - tests/rotate_check_test.rs
  - tests/runtime_scan_test.rs
```

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/cli/lane_check.rs` | Task 1: refactor — `pub fn execute()` returning `RunOutput`, `run()` thành thin wrapper |
| `src/cli/validate_map.rs` | Task 1: same refactor pattern |
| `src/cli/rotate_check.rs` | Task 1: same refactor pattern |
| `src/cli/runtime_scan.rs` | Task 1: same refactor pattern |
| `src/cli/mod.rs` | Task 1 (optional): add shared `pub struct RunOutput` re-export |
| `src/main.rs` | Task 3: add `mod mcp;`, REFACTOR existing Serve dispatch arm body với tokio runtime block_on (variant đã pre-wired stub) |
| `src/mcp/mod.rs` | Task 2 (new): `DoctorServer` struct, tool input structs, `#[tool_router]` impl (SYNC tool fns), `serve()` fn (async) |
| `src/mcp/tools.rs` | Task 2 (new, optional split): `#[tool_router]` impl block tách khỏi mod.rs nếu Worker chọn |
| `tests/mcp_serve_test.rs` | Task 4 (new): 2 smoke case tools/list + tools/call lane_check |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `Cargo.toml` | Anchor #1 confirm rmcp/schemars/tokio đã có — KHÔNG add dep |
| `tests/lane_check_test.rs` | P001 regression PHẢI pass sau Task 1 refactor (run() signature giữ nguyên) |
| `tests/validate_map_test.rs` | P002 regression PHẢI pass |
| `tests/rotate_check_test.rs` | P003 regression PHẢI pass |
| `tests/runtime_scan_test.rs` | P004 regression PHẢI pass |
| `docs/ARCHITECTURE.md` | Worker post-execute update L93-100 MCP section khớp final impl |

---

## Luật chơi (Constraints)

1. **KHÔNG re-implement logic trong MCP wrapper** — mỗi tool fn chỉ gọi `<subcmd>::execute(args)` rồi format `CallToolResult`. Duplicate logic = drift risk, vi phạm DRY. Vi phạm → Worker STOP, AskUserQuestion.

2. **Output structured qua `RunOutput { exit_code: u8, stdout: String, stderr: String }`** — CLI `run()` print + map exit; MCP wrapper capture + `is_error: Some(exit_code != 0)`. Exit code ánh xạ MCP isError flag chứ KHÔNG ném exception qua JSON-RPC error response (trừ khi IO/parse error thực sự — bubble qua `ErrorData::internal_error`). **Tool fn return type `Result<CallToolResult, ErrorData>` trực tiếp** (V2 Architect decision — Option A: text-only output, no `Json<T>` wrapper, no extra output structs).

3. **Smoke test only — 2 case** — `tools_list` + `lane_check tools/call` happy path. KHÔNG test 4 tool full end-to-end qua MCP (oracle cost cao, defer dogfood). E2E coverage = 4 CLI subcmd test 27/27 existing đã đảm bảo logic, MCP wrapper layer chỉ smoke verify wiring.

4. **rmcp 1.7.0 idiom — follow advisory-inbox precedent** — Worker đọc `~/advisory-inbox/src/mcp/` source trước viết `src/mcp/mod.rs`. Specifically: `#[tool_router]` macro + `#[tool(description=...)]` per-fn + `#[tool_handler]` on `ServerHandler` impl với two-step pattern (KHÔNG `from_build_env` per P011 retro). **Tool fns là SYNC `fn`, KHÔNG `async fn`** (V2 — advisory-inbox `src/mcp/tools.rs:176-398` precedent). Mismatch nào với precedent → log Discovery + cite line.

5. **4 CLI subcmd P001-P004 vẫn pass test sau refactor (regression critical)** — `cargo test --test lane_check_test`, `validate_map_test`, `rotate_check_test`, `runtime_scan_test` PHẢI 27/27 pass không sửa test code. Pass = `pub fn run()` signature + main.rs dispatch behavior unchanged. Fail → Worker STOP, revert refactor, AskUserQuestion.

6. **Schemars derive cho input struct — JSON schema phải clean** — `#[derive(JsonSchema, Deserialize)]` produce flat schema (string + Option<string> + Option<bool>). KHÔNG nest `Result`, anyhow, PathBuf qua schema (PathBuf chuyển String trong input struct, convert sang PathBuf trong tool fn body).

7. **Lane Guarded — phiếu này được phép > 250 dòng, > 5 anchor, > 5 constraint** nhưng vẫn cố ≤ 400 dòng. Vượt 400 → Worker note Discovery (lane Guarded sang phiếu Tầng 1 thực sự, cần split phiếu lần sau).

---

## Nghiệm thu

### Automated
- [ ] `cargo build` clean — refactor 4 subcmd + new `src/mcp/` module compile no warnings.
- [ ] `cargo build --release` clean — production build verify.
- [ ] `cargo test` full suite — **27 existing + 2 new = 29 pass minimum** (regression critical).
- [ ] `cargo test --test mcp_serve_test` — 2 smoke case pass.
- [ ] `cargo test --test lane_check_test` — 27/27 P001-P004 baseline khớp.

### Manual Testing
- [ ] `cargo run -- serve` — server start, không panic, chờ stdin.
- [ ] Echo JSON-RPC request manual:
  ```bash
  printf '%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"manual","version":"1.0"}}}' \
    '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
    '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
    | cargo run -- serve
  ```
  → response chứa 4 tool name (lane_check, validate_map, rotate_check, runtime_scan).
- [ ] Tools/call `lane_check` qua manual JSON-RPC với tmp ticket → response `is_error: false`, content chứa exit code/output.
- [ ] `cargo run -- --help` show 5 subcmd (4 cũ + `serve`).
- [ ] `cargo run -- serve --help` (clap auto-gen) — show description.

### Regression
- [ ] `cargo run -- lane-check --ticket docs/ticket/P004-runtime-scan.md` — exit 0 (P004 đã ship Normal lane).
- [ ] `cargo run -- validate-map --map docs/AGENT_MAP.yaml` (nếu file exist) — exit khớp baseline.
- [ ] `cargo run -- rotate-check --repo .` — exit 0 (DISCOVERIES nhỏ baseline).
- [ ] `cargo run -- runtime-scan --repo .` — exit 0 (no leak baseline).
- [ ] `doctor --version` ra `doctor 0.1.0`.

### Install binary verify
- [ ] `cargo install --path .` — install vào `~/.cargo/bin/doctor`.
- [ ] `which doctor` → `~/.cargo/bin/doctor`.
- [ ] `doctor --version` → `doctor 0.1.0`.
- [ ] `doctor --help` show 5 subcmd.
- [ ] `doctor serve` start — manual JSON-RPC tools/list verify như Manual Testing.

### Docs Gate
- [ ] `docs/CHANGELOG.md` — thêm entry: `P005: MCP serve mode (4 tool — lane_check, validate_map, rotate_check, runtime_scan; rmcp 1.7.0 stdio JSON-RPC 2.0; refactor 4 subcmd execute/run split; binary ship + ~/.mcp.json wire). MVP sprint COMPLETE.`
- [ ] `docs/ARCHITECTURE.md` L93-100 — Worker update khớp final layout (`src/mcp/mod.rs` content, `RunOutput` struct location, tokio runtime pattern chốt).
- [ ] `~/.mcp.json` entry sample paste vào CHANGELOG hoặc README (user-facing wire):
  ```json
  {
    "mcpServers": {
      "doctor": {
        "command": "/Users/nguyenhuuanh/.cargo/bin/doctor",
        "args": ["serve"]
      }
    }
  }
  ```

### Discovery Report
- [ ] Write `docs/discoveries/P005.md`:
  - Anchor #1-#7 result table (final ✅/❌, dep version exact, Serve variant pre-existing stub confirmed)
  - Refactor scope: 4 subcmd `pub fn run` → `execute + run` split — LOC diff, regression test impact
  - `RunOutput` struct location chốt (`src/cli/mod.rs` shared HOẶC inline per file)
  - rmcp 1.7.0 API quirk gặp phải (Parameters<T> wrapper, sync tool fn pattern, ServerInfo two-step, ErrorData constructor)
  - schemars 1.0 compat — derive clean hay phải downgrade/manual impl
  - Tokio runtime pattern chốt (Option B manual `block_on` chỉ Serve arm — confirmed)
  - File split chốt: `src/mcp/mod.rs` all-in-one HOẶC tách `tools.rs`
  - MVP sprint retrospective note — 5/5 phiếu ship, 27+2 test pass, binary install verify
  - 3-field SOUND oracle:
    ```
    Claim: doctor serve expose 4 SOUND mechanical gate qua MCP stdio JSON-RPC 2.0
    Oracle: cargo test (smoke 2 case tools/list + lane_check tools/call) + manual echo JSON-RPC + cargo install verify
    Soundness: SOUND for tools/list shape + 1 tools/call happy path; PARTIAL for 4-tool full E2E (3 tool còn lại defer dogfood — orchestrator real usage = integration test)
    ```
- [ ] Append 1-line index entry to `docs/DISCOVERIES.md`: `P005 MCP serve ship — 4 tool, rmcp 1.7.0, refactor execute/run split, MVP COMPLETE.`
