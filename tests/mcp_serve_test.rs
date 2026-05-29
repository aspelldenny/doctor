//! Smoke tests for `doctor serve` MCP mode (phiếu P005).
//!
//! 2 test cases (constraint #3 — smoke only, not full E2E 4-tool):
//!   1. tools_list_returns_4_tools — verify all 4 tool names exposed via JSON-RPC.
//!   2. lane_check_tool_call_happy_path — tools/call lane_check with Normal-lane fixture → is_error false.
//!
//! Pattern: spawn binary, write JSON-RPC messages to stdin (one per line),
//! close stdin to signal EOF, read stdout, parse JSON-RPC responses.
//! Mirror of advisory-inbox tests/mcp_tools_cli.rs (precedent — P011).

use assert_cmd::cargo::CommandCargoExt;
use std::io::Write;
use std::process::{Command, Stdio};

// ──────────────────────────────────────────────────────────────
// JSON-RPC constants
// ──────────────────────────────────────────────────────────────

const INIT_REQUEST: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"0.0.0"}}}"#;

const TOOLS_LIST_REQUEST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;

// ──────────────────────────────────────────────────────────────
// Session helper
// ──────────────────────────────────────────────────────────────

/// Spawn `doctor serve`, write all `messages` to stdin (newline-separated),
/// close stdin, collect stdout lines parsed as JSON.
fn run_mcp_session(messages: &[&str]) -> Vec<serde_json::Value> {
    let mut child = Command::cargo_bin("doctor")
        .expect("cargo_bin doctor")
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn doctor serve");

    {
        let stdin = child.stdin.as_mut().expect("stdin handle");
        for msg in messages {
            writeln!(stdin, "{}", msg).expect("write MCP message");
        }
    }
    // Close stdin — signals EOF; rmcp resolves .waiting() and process exits cleanly.
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait for child");
    assert!(
        output.status.success(),
        "doctor serve exited non-zero — status: {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .collect()
}

/// Find the JSON-RPC response with the given numeric `id`.
fn find_response(lines: &[serde_json::Value], id: u64) -> Option<&serde_json::Value> {
    lines
        .iter()
        .find(|v| v.get("id").and_then(|i| i.as_u64()) == Some(id))
}

// ──────────────────────────────────────────────────────────────
// Test 1: tools/list returns exactly 5 tool names
// ──────────────────────────────────────────────────────────────

#[test]
fn tools_list_returns_5_tools() {
    let lines = run_mcp_session(&[INIT_REQUEST, TOOLS_LIST_REQUEST]);

    let response = find_response(&lines, 2).expect("no tools/list response found in stdout");

    let tools = response
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .expect("result.tools array missing");

    assert_eq!(
        tools.len(),
        5,
        "expected 5 tools, got {}. Response: {}",
        tools.len(),
        response
    );

    let names: std::collections::HashSet<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    let expected: std::collections::HashSet<&str> = [
        "lane_check",
        "validate_map",
        "rotate_check",
        "runtime_scan",
        "verify_setup",
    ]
    .iter()
    .copied()
    .collect();

    assert_eq!(
        names, expected,
        "tool names mismatch — got: {:?}",
        names
    );
}

// ──────────────────────────────────────────────────────────────
// Test 2: tools/call lane_check happy path (Normal lane fixture)
// ──────────────────────────────────────────────────────────────

#[test]
fn lane_check_tool_call_happy_path() {
    // Write a minimal Normal-lane phiếu fixture to a tmp file.
    let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
    // 30 lines, 2 anchors, 2 constraints — well under Normal budget.
    let mut content = "> **Lane:** Normal\n\n## Task 0\n\n".to_string();
    content.push_str("| 1 | anchor_a |\n");
    content.push_str("| 2 | anchor_b |\n");
    content.push_str("\n## Luật chơi\n\n1. constraint one\n2. constraint two\n");
    for _ in 0..20 {
        content.push_str("padding line\n");
    }
    tmp.write_all(content.as_bytes()).expect("write fixture");

    let ticket_path = tmp.path().to_string_lossy().replace('\\', "/");

    // Build the tools/call request with the fixture path.
    let call_request = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"lane_check","arguments":{{"ticket":"{}"}}}}}}"#,
        ticket_path
    );

    let lines = run_mcp_session(&[INIT_REQUEST, &call_request]);

    let response = find_response(&lines, 3).expect("no tools/call response found in stdout");

    // Response should have result (not error at protocol level).
    let result = response
        .get("result")
        .expect("response should have result field (not error)");

    // is_error should be false — Normal lane within budget.
    let is_error = result.get("isError").and_then(|v| v.as_bool());
    assert_eq!(
        is_error,
        Some(false),
        "expected isError=false for Normal lane fixture, got: {:?}. Full result: {}",
        is_error,
        result
    );

    // Content should contain "OK".
    let content_arr = result
        .get("content")
        .and_then(|c| c.as_array())
        .expect("result.content array missing");

    assert!(
        !content_arr.is_empty(),
        "result.content is empty — expected at least one text block"
    );

    let text = content_arr[0]
        .get("text")
        .and_then(|t| t.as_str())
        .expect("content[0].text missing");

    assert!(
        text.contains("OK"),
        "expected content text to contain 'OK', got: {:?}",
        text
    );
}
