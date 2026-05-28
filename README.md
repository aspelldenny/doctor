# doctor

v2.2 mechanical gate enforcement — 1 Rust binary, 4 SOUND subcmd, cài 1 lần dùng nhiều repo.

## Mission

Workflow v2.2 (sos-kit doctrine) defines mechanical gates: lane budget, AGENT_MAP path/anchor, dòng cap rotate, runtime token leak. **doctor implements them all.**

Until doctor cài, v2.2 doctrine ship "không có răng" — gates exist on paper, KHÔNG enforce. Doctor là cái khác biệt giữa doctrine in markdown và doctrine in runtime.

## MVP subcmd (4 SOUND gates)

```
doctor lane-check --ticket <path>      # §1 lane budget enforcement
doctor validate-map --map <path>       # §4 AGENT_MAP path/anchor exist
doctor rotate-check [--repo <path>]    # §6 DISCOVERIES/CHANGELOG dòng cap
doctor runtime-scan [--include-home]   # Sub-mech F token leak grep
doctor serve                           # MCP server stdio JSON-RPC 2.0
```

Each subcmd answers ONE question with mã phiếu xác chết (lỗi thật đã xảy ra).

## Install

```bash
git clone https://github.com/aspelldenny/doctor ~/doctor
cd ~/doctor
cargo install --path .
# Binary cài vào ~/.cargo/bin/doctor

# Verify
doctor --version
doctor --help
```

## MCP wire (Claude Code session future)

Add to `~/.mcp.json`:
```json
{
  "mcpServers": {
    "doctor": {
      "command": "/Users/<you>/.cargo/bin/doctor",
      "args": ["serve"]
    }
  }
}
```

Claude session invokes via `mcp__doctor__lane_check`, `mcp__doctor__validate_map`, etc.

## Doctrine source

Doctor follows Workflow v2.2:
- Spec: [sos-kit/docs/WORKFLOW_V2.2.md](https://github.com/aspelldenny/sos-kit/blob/main/docs/WORKFLOW_V2.2.md)
- Retro forge (7 rounds): [sos-kit/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md](https://github.com/aspelldenny/sos-kit/blob/main/docs/retro/WORKFLOW_V2.2_RETRO_advisory-inbox.md)

## 3 hard lines (xem `docs/SOUL.md`)

1. MECHANICAL SOUND ONLY — không ôm judgment.
2. RUST PORTABLE — 1 binary, mọi repo.
3. KHÔNG nuốt pilot thí nghiệm — gặp PARTIAL, defer Python pilot vòng 2.

## Stack

- Rust 2024 edition (rust-version 1.85+)
- clap 4 derive
- serde + serde_yaml + serde_json
- rmcp 1.7.0 (MCP serve)
- tempfile, regex, walkdir

## Repos liên quan

- [aspelldenny/sos-kit](https://github.com/aspelldenny/sos-kit) — doctrine source
- [aspelldenny/advisory-inbox](https://github.com/aspelldenny/advisory-inbox) — precedent Rust binary CLI + MCP dual mode

## Status

🚧 **Skeleton ship 2026-05-28.** 4 MVP subcmd CHƯA implement (todo!() placeholder). Build P001-P005 qua v2.2 workflow trong sprint kế tiếp.

## License

MIT
