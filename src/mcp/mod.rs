//! MCP (Model Context Protocol) server surface for doctor.
//!
//! P005 ships 4 tools matching the 4 CLI subcmds:
//! - `lane_check`    — §1 lane budget gate (WORKFLOW_V2.2 §1)
//! - `validate_map`  — §4 AGENT_MAP path/anchor check
//! - `rotate_check`  — §6 dòng cap enforcement
//! - `runtime_scan`  — Sub-mech F runtime token leak scan
//!
//! Pattern: rmcp 1.7.0 `#[tool_router]` + `#[tool_handler]` two-step.
//! Tool fns are SYNC (rmcp 1.7.0 supports sync tool fns — advisory-inbox precedent).
//! Only `serve()` is async (rmcp transport layer).
//!
//! Arch note: each tool fn delegates to `cli::<subcmd>::execute(args)` — zero logic
//! duplication (constraint #1 phiếu P005).

use anyhow::Result;
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

use crate::cli::{RunOutput, lane_check, rotate_check, runtime_scan, validate_map};

// ──────────────────────────────────────────────────────────────
// Tool input structs — #[derive(JsonSchema, Deserialize)]
// PathBuf inputs converted from String inside tool fn body (constraint #6).
// ──────────────────────────────────────────────────────────────

/// Input for the lane_check tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LaneCheckInput {
    /// Path to the phiếu markdown file to check lane budget for.
    pub ticket: String,
}

/// Input for the validate_map tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateMapInput {
    /// Path to the AGENT_MAP.yaml file to validate.
    pub map: String,
}

/// Input for the rotate_check tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RotateCheckInput {
    /// Path to repo root (defaults to cwd if omitted).
    #[serde(default)]
    pub repo: Option<String>,
}

/// Input for the runtime_scan tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RuntimeScanInput {
    /// Path to repo root (defaults to cwd if omitted).
    #[serde(default)]
    pub repo: Option<String>,
    /// Also scan home directory files (~/.ssh/config, ~/.gitconfig, ~/.netrc).
    #[serde(default)]
    pub include_home: Option<bool>,
}

// ──────────────────────────────────────────────────────────────
// Helper: map anyhow Result<RunOutput> → Result<CallToolResult, ErrorData>
// ──────────────────────────────────────────────────────────────

/// Convert subcmd execute() result into a MCP CallToolResult.
///
/// - IO errors (Err) → ErrorData::internal_error (JSON-RPC error).
/// - exit_code != 0 → CallToolResult with is_error=true + combined stdout+stderr text.
/// - exit_code == 0 → CallToolResult with is_error=false + combined stdout+stderr text.
fn run_to_call_result(result: Result<RunOutput>) -> Result<CallToolResult, ErrorData> {
    let out = result.map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
    let text = format!("{}{}", out.stdout, out.stderr);
    if out.exit_code != 0 {
        Ok(CallToolResult::error(vec![Content::text(text)]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

// ──────────────────────────────────────────────────────────────
// DoctorServer struct + #[tool_router] impl
// ──────────────────────────────────────────────────────────────

/// MCP service exposing 4 doctor CLI gates as tools.
///
/// Unit struct (no state) — `#[tool_router]` generates `Self::tool_router()`.
pub struct DoctorServer;

#[tool_router]
impl DoctorServer {
    /// Check phiếu lane budget (v2.2 §1): Normal ≤250 lines/5 anchors/5 constraints; Fast ≤100 lines; Guarded uncapped.
    #[tool(
        name = "lane_check",
        description = "Check phiếu lane budget (v2.2 §1): Normal ≤250 lines/5 anchors/5 constraints; Fast ≤100 lines; Guarded uncapped."
    )]
    fn lane_check(
        &self,
        Parameters(p): Parameters<LaneCheckInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = lane_check::Args {
            ticket: PathBuf::from(&p.ticket),
        };
        run_to_call_result(lane_check::execute(args))
    }

    /// Validate AGENT_MAP.yaml: check all path entries exist and anchors are reachable.
    #[tool(
        name = "validate_map",
        description = "Validate AGENT_MAP.yaml: check all path entries exist and anchors are reachable (§4)."
    )]
    fn validate_map(
        &self,
        Parameters(p): Parameters<ValidateMapInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = validate_map::Args {
            map: PathBuf::from(&p.map),
        };
        run_to_call_result(validate_map::execute(args))
    }

    /// Check DISCOVERIES/CHANGELOG line cap (v2.2 §6): soft 1000 warn, hard 1500 block.
    #[tool(
        name = "rotate_check",
        description = "Check DISCOVERIES/CHANGELOG line cap (v2.2 §6): soft 1000 warn, hard 1500 block."
    )]
    fn rotate_check(
        &self,
        Parameters(p): Parameters<RotateCheckInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = rotate_check::Args {
            repo: p.repo.map(PathBuf::from),
        };
        run_to_call_result(rotate_check::execute(args))
    }

    /// Scan runtime config files for hardcoded secrets (Sub-mech F): .git/config, .mcp.json, .env*.
    #[tool(
        name = "runtime_scan",
        description = "Scan runtime config files for hardcoded secrets (Sub-mech F): .git/config, .mcp.json, .env*."
    )]
    fn runtime_scan(
        &self,
        Parameters(p): Parameters<RuntimeScanInput>,
    ) -> Result<CallToolResult, ErrorData> {
        let args = runtime_scan::Args {
            repo: p.repo.map(PathBuf::from),
            include_home: p.include_home.unwrap_or(false),
        };
        run_to_call_result(runtime_scan::execute(args))
    }
}

// ──────────────────────────────────────────────────────────────
// ServerHandler impl — custom get_info reads from doctor Cargo.toml,
// not rmcp's own build env (advisory-inbox P011 retro).
// ──────────────────────────────────────────────────────────────

#[tool_handler]
impl ServerHandler for DoctorServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(
            Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )
    }
}

// ──────────────────────────────────────────────────────────────
// serve() — async entry point called from main.rs Serve arm
// ──────────────────────────────────────────────────────────────

/// Start MCP server on stdio transport (JSON-RPC 2.0).
///
/// Blocks until stdin EOF (client disconnect). Returns Ok on clean shutdown.
pub async fn serve() -> Result<()> {
    use rmcp::{ServiceExt, transport::io::stdio};
    let server = DoctorServer;
    let transport = stdio();
    let running = server
        .serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("MCP server init error: {}", e))?;
    running
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("MCP server runtime error: {}", e))?;
    Ok(())
}
