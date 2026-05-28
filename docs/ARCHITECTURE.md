# ARCHITECTURE — doctor

## Module layout

```
src/
├── main.rs              # clap subcmd dispatch
├── lib.rs               # (deferred — chưa cần khi subcmd trong cli/ module)
└── cli/
    ├── mod.rs           # subcmd module exports
    ├── lane_check.rs    # §1 lane budget (P001)
    ├── validate_map.rs  # §4 AGENT_MAP path/anchor (P002)
    ├── rotate_check.rs  # §6 dòng cap (P003)
    └── runtime_scan.rs  # Sub-mech F token leak (P004)
```

Future addition (P005 MCP):
```
src/
└── mcp/
    ├── mod.rs
    └── tools.rs         # rmcp tool_router with 4 tools
```

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

## MCP serve (P005, post-MVP)

Pattern khớp advisory-inbox `serve` subcmd:
- Use `rmcp` 1.7.0 với feature `["macros", "schemars"]`
- `#[tool_router]` on impl block
- Each tool wraps existing `cli::*::run()` function (Strategy B: extract `pub fn execute()` from CLI run)
- `#[tool_handler]` on `ServerHandler` impl với two-step pattern (avoid `from_build_env` reading rmcp crate name — see advisory-inbox P011 retro)

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
