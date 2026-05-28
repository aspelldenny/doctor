//! Integration tests for `doctor validate-map` subcmd (phiếu P002).
//!
//! Pattern: assert_cmd + tempfile workspace (mirror of lane_check_test.rs from P001).
//! Each test builds a temporary directory with YAML map + target files via std::fs::write.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn doctor() -> Command {
    Command::cargo_bin("doctor").expect("doctor binary")
}

/// Helper: create a tempdir with a map file at `map.yaml` plus any additional files.
/// Returns (TempDir, map_path_string). TempDir must stay alive for the duration of the test.
fn setup_dir() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

// ---------------------------------------------------------------------------
// Test 1: ok_clean_all_exists — 2 surfaces, paths exist, heading anchor matches
// ---------------------------------------------------------------------------
#[test]
fn ok_clean_all_exists() {
    let dir = setup_dir();
    let base = dir.path();

    // Create target files
    fs::write(base.join("src_lib.rs"), "pub fn my_function() {}\n").unwrap();
    fs::write(base.join("docs_guide.md"), "# My Guide\n\nContent here.\n").unwrap();

    let map_yaml = format!(
        r#"version: 1
surfaces:
  core:
    load_bearing: true
    edit:
      - {src_lib}
    read_shallow:
      - {docs_guide}#my-guide
"#,
        src_lib = base.join("src_lib.rs").display(),
        docs_guide = base.join("docs_guide.md").display(),
    );

    let map_path = base.join("map.yaml");
    fs::write(&map_path, &map_yaml).unwrap();

    doctor()
        .arg("validate-map")
        .arg("--map")
        .arg(&map_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

// ---------------------------------------------------------------------------
// Test 2: fail_path_missing — yaml references a non-existent path
// ---------------------------------------------------------------------------
#[test]
fn fail_path_missing() {
    let dir = setup_dir();
    let base = dir.path();

    let map_yaml = format!(
        r#"version: 1
surfaces:
  broken:
    load_bearing: false
    edit:
      - {missing}
"#,
        missing = base.join("does_not_exist.rs").display(),
    );

    let map_path = base.join("map.yaml");
    fs::write(&map_path, &map_yaml).unwrap();

    doctor()
        .arg("validate-map")
        .arg("--map")
        .arg(&map_path)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("PATH_MISSING"));
}

// ---------------------------------------------------------------------------
// Test 3: fail_anchor_missing — path exists but anchor not found in file
// ---------------------------------------------------------------------------
#[test]
fn fail_anchor_missing() {
    let dir = setup_dir();
    let base = dir.path();

    // File exists but does NOT contain "## Missing Section"
    fs::write(base.join("doc.md"), "# Present Section\n\nSome text.\n").unwrap();

    let map_yaml = format!(
        r#"version: 1
surfaces:
  docs_surface:
    read_shallow:
      - {doc}#missing-section
"#,
        doc = base.join("doc.md").display(),
    );

    let map_path = base.join("map.yaml");
    fs::write(&map_path, &map_yaml).unwrap();

    doctor()
        .arg("validate-map")
        .arg("--map")
        .arg(&map_path)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("ANCHOR_MISSING"));
}

// ---------------------------------------------------------------------------
// Test 4: fail_yaml_parse — invalid yaml structure
// ---------------------------------------------------------------------------
#[test]
fn fail_yaml_parse() {
    let dir = setup_dir();
    let map_path = dir.path().join("map.yaml");

    // surfaces must be a map, not a list
    fs::write(&map_path, "version: 1\nsurfaces: [not a map]\n").unwrap();

    doctor()
        .arg("validate-map")
        .arg("--map")
        .arg(&map_path)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("yaml parse failed"));
}

// ---------------------------------------------------------------------------
// Test 5: ok_anchor_flexible_hyphen_space — anchor `My-Section` matches `## My Section`
// ---------------------------------------------------------------------------
#[test]
fn ok_anchor_flexible_hyphen_space() {
    let dir = setup_dir();
    let base = dir.path();

    // Heading uses spaces; AGENT_MAP anchor uses hyphens (URL-style)
    fs::write(base.join("notes.md"), "# Intro\n\n## My Section\n\nDetails.\n").unwrap();

    let map_yaml = format!(
        r#"version: 1
surfaces:
  notes:
    read_deep:
      - {notes}#My-Section
"#,
        notes = base.join("notes.md").display(),
    );

    let map_path = base.join("map.yaml");
    fs::write(&map_path, &map_yaml).unwrap();

    doctor()
        .arg("validate-map")
        .arg("--map")
        .arg(&map_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

// ---------------------------------------------------------------------------
// Test 6: ok_glob_skipped — entries with ** are silently skipped
// ---------------------------------------------------------------------------
#[test]
fn ok_glob_skipped() {
    let dir = setup_dir();
    let base = dir.path();

    // Glob entry — the directory or files may not exist; still exit 0
    let map_yaml = format!(
        r#"version: 1
surfaces:
  archive:
    read_shallow:
      - {base}/archive/**
"#,
        base = base.display(),
    );

    let map_path = base.join("map.yaml");
    fs::write(&map_path, &map_yaml).unwrap();

    doctor()
        .arg("validate-map")
        .arg("--map")
        .arg(&map_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

// ---------------------------------------------------------------------------
// Test 7: ok_rust_symbol_anchor — anchor matches Rust fn symbol
// ---------------------------------------------------------------------------
#[test]
fn ok_rust_symbol_anchor() {
    let dir = setup_dir();
    let base = dir.path();

    fs::write(
        base.join("lib.rs"),
        "pub fn my_function() -> bool { true }\n",
    )
    .unwrap();

    let map_yaml = format!(
        r#"version: 1
surfaces:
  core:
    contract_test:
      - {lib}#my_function
"#,
        lib = base.join("lib.rs").display(),
    );

    let map_path = base.join("map.yaml");
    fs::write(&map_path, &map_yaml).unwrap();

    doctor()
        .arg("validate-map")
        .arg("--map")
        .arg(&map_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}
