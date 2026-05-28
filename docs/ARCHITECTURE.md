# ARCHITECTURE — doctor

## Module layout

```
src/
├── main.rs              # clap subcmd dispatch; Serve arm builds tokio runtime + block_on mcp::serve()
├── lib.rs               # (deferred — not needed with subcmd-in-cli/ pattern)
├── cli/
│   ├── mod.rs           # subcmd module exports + shared pub struct RunOutput
│   ├── lane_check.rs    # §1 lane budget (P001) — pub fn execute() + pub fn run()
│   ├── validate_map.rs  # §4 AGENT_MAP path/anchor (P002) — execute/run split
│   ├── rotate_check.rs  # §6 dòng cap (P003) — execute/run split
│   └── runtime_scan.rs  # Sub-mech F token leak (P004) — execute/run split
└── mcp/
    └── mod.rs           # DoctorServer (#[tool_router] + #[tool_handler]), serve() async fn (P005)
```

### RunOutput shared struct (P005 — src/cli/mod.rs)

```rust
pub struct RunOutput {
    pub exit_code: u8,    // 0=ok, 1=soft fail, 2=hard error
    pub stdout: String,   // captured stdout
    pub stderr: String,   // captured stderr
}
```

Each subcmd exposes:
- `pub fn execute(args: Args) -> Result<RunOutput>` — pure logic, no side effects, IO errors bubble as Err
- `pub fn run(args: Args) -> Result<()>` — thin CLI wrapper: call execute, print, std::process::exit(N)

## 4 subcmd contract

### lane-check (P001)

```rust
pub struct Args {
    #[arg(long)] pub ticket: PathBuf,
}

pub fn run(args: Args) -> Result<()> {
    // 1. Read ticket markdown
    // 2. Parse "Lane: Normal/Guarded/Fast" header field (regex)
    // 3. Count: total lines, anchor table rows (Markdown table parse), constraint count (regex "[ ]")
    // 4. Compare vs lane budgets (Normal ≤ 250 dòng, ≤ 5 anchor, ≤ 5 constraint)
    // 5. Exit 0 = OK, exit 1 = budget exceeded with reason, exit 2 = missing lane field
}
```

### validate-map (P002)

```rust
pub struct Args {
    #[arg(long)] pub map: PathBuf,
}

pub fn run(args: Args) -> Result<()> {
    // 1. Read map yaml, serde_yaml::from_str
    // 2. For each surface:
    //    - walk edit/read_shallow/read_deep/research_gate/contract_test arrays
    //    - For each path, check std::path::Path::exists() — accumulate misses
    //    - For paths with "#anchor", grep target file for "^#+\s+anchor"
    // 3. Exit 0 = clean, exit 1 = drift with list, exit 2 = parse error
}
```

### rotate-check (P003)

```rust
pub struct Args {
    #[arg(long)] pub repo: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    // 1. Locate repo root (args.repo or cwd or git rev-parse)
    // 2. Read .sos-stack.toml [rotate] section (or default soft=1000, hard=1500)
    // 3. Count lines: docs/DISCOVERIES.md, docs/CHANGELOG.md
    // 4. Compare vs cap
    // 5. Exit 0 = under soft, exit 1 = soft cap warn, exit 2 = hard cap block
}
```

### runtime-scan (P004)

```rust
pub struct Args {
    #[arg(long)] pub repo: Option<PathBuf>,
    #[arg(long)] pub include_home: bool,
}

pub fn run(args: Args) -> Result<()> {
    // 1. Targets: .git/config, .env*, .mcp.json (in repo root)
    //    Optional: ~/.ssh/config, ~/.gitconfig (if --include-home)
    // 2. Patterns: ghp_*, gho_*, ghu_*, ghs_*, github_pat_*, AKIA*, sk-*, etc.
    // 3. Grep each file, accumulate matches with file:line
    // 4. Exit 0 = clean, exit 1 = leak found with list
}
```

## MCP serve (P005 — SHIPPED)

Pattern: rmcp 1.7.0 `#[tool_router]` + `#[tool_handler]` two-step (advisory-inbox precedent).
- `DoctorServer` unit struct (no field) in `src/mcp/mod.rs`
- 4 sync tool fns: `lane_check`, `validate_map`, `rotate_check`, `runtime_scan`
  - Each: `fn &self, Parameters(p): Parameters<XInput>) -> Result<CallToolResult, ErrorData>`
  - Body: build `cli::<subcmd>::Args`, call `<subcmd>::execute(args)`, map via `run_to_call_result()`
- `run_to_call_result(Result<RunOutput>) -> Result<CallToolResult, ErrorData>`: exit_code!=0 → is_error=true
- `get_info()` uses `Implementation::new(env!("CARGO_PKG_NAME"), ...)` — reads doctor's Cargo.toml, not rmcp's
- `pub async fn serve()` in `src/mcp/mod.rs` — rmcp stdio transport
- main.rs Serve arm: `tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(mcp::serve())`

## Distribution

- `cargo install --path .` → `~/.cargo/bin/doctor`
- `~/.mcp.json` entry: `"doctor": { "command": "~/.cargo/bin/doctor", "args": ["serve"] }`
- No crates.io publish (precedent: 6 binary đã ship cũng chưa publish)

## Stack constraints

- Rust 2024 edition (rust-version 1.85+)
- Tokio rt feature only (NO rt-multi-thread — advisory-inbox precedent matched)
- rmcp serve feature (transport-io for stdin/stdout JSON-RPC)
- Binary size target < 5 MB (advisory-inbox 2.16 MB precedent)

## Sub-mechanism coverage

| Sub-mech | Covered by | Status |
|----------|-----------|--------|
| A (trigger gap) | scripts/block-unsafe-merge.sh, scripts/architect-guard.sh | Skeleton ship |
| B (capability) | (project-specific, no doctor subcmd needed for Rust scaffold) | N/A |
| C (migration) | DEFERRED — `migrate-check` ship khi migration phiếu fire | Defer |
| D (persistence) | DEFERRED — `doctrine-check` defer | Defer |
| E (env drift) | (CI gate cargo build --frozen) | Future |
| **F (runtime state)** | `runtime-scan` subcmd (P004) | MVP |

## Out of scope v0.1.0

- Auto-fix mode (doctor surfaces evidence, không patch).
- Custom rule plugins (per-repo INV-LOCAL stays in INVARIANTS.md, boundary-check subagent receives via inject — KHÔNG nuốt vào doctor).
- Watch mode / daemon (one-shot CLI only).
- Cross-language type checking (Rust only; partial-oracle stacks delegated to language-native tooling).
