// REQ-CORE-022

use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};
use tempfile::tempdir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workspaces")
        .join(name)
}

fn write_gap_fixture_workspace() -> tempfile::TempDir {
    let tempdir = tempdir().expect("tempdir should exist");
    let docs_root = tempdir.path().join("docs/syu");
    fs::create_dir_all(docs_root.join("philosophy")).expect("philosophy dir");
    fs::create_dir_all(docs_root.join("policies")).expect("policies dir");
    fs::create_dir_all(docs_root.join("requirements")).expect("requirements dir");
    fs::create_dir_all(docs_root.join("features")).expect("features dir");

    fs::write(
        docs_root.join("philosophy/foundation.yaml"),
        "category: Philosophy\nversion: 1\nlanguage: en\nphilosophies:\n  - id: PHIL-GAP-001\n    title: Gap philosophy\n    product_design_principle: Keep gaps visible.\n    coding_guideline: Prefer inspection commands.\n    linked_policies: []\n",
    )
    .expect("philosophy file");
    fs::write(
        docs_root.join("policies/policies.yaml"),
        "category: Policies\nversion: 1\nlanguage: en\npolicies:\n  - id: POL-GAP-001\n    title: Gap policy\n    summary: Leave one sparse requirement.\n    description: Gap policy.\n    linked_philosophies:\n      - PHIL-GAP-001\n    linked_requirements: []\n",
    )
    .expect("policy file");
    fs::write(
        docs_root.join("requirements/core.yaml"),
        "category: Core\nprefix: REQ-GAP\n\nrequirements:\n  - id: REQ-GAP-001\n    title: Sparse requirement\n    description: This requirement intentionally leaves links empty.\n    priority: medium\n    status: planned\n    linked_policies: []\n    linked_features: []\n    tests: {}\n",
    )
    .expect("requirement file");
    fs::write(
        docs_root.join("features/features.yaml"),
        "version: \"1\"\nfiles:\n  - kind: dummy\n    file: dummy.yaml\n",
    )
    .expect("feature registry");
    fs::write(
        docs_root.join("features/dummy.yaml"),
        "category: Dummy\nversion: 1\nfeatures:\n  - id: FEAT-DUMMY-001\n    title: Dummy feature\n    summary: Keeps the workspace loadable.\n    status: planned\n    linked_requirements: []\n    implementations: {}\n",
    )
    .expect("feature file");

    tempdir
}

#[test]
fn relate_command_traverses_the_connected_graph_from_a_requirement() {
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("relate")
        .arg("REQ-TRACE-001")
        .arg(fixture_path("passing"))
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Selection: definition REQ-TRACE-001"));
    assert!(stdout.contains("philosophy PHIL-TRACE-001"));
    assert!(stdout.contains("policy POL-TRACE-001"));
    assert!(stdout.contains("feature FEAT-TRACE-001"));
    assert!(stdout.contains("src/rust_trace_tests.rs"));
    assert!(stdout.contains("src/rust_feature.rs"));
    assert!(stdout.contains("Gaps:\n- none"));
}

#[test]
fn relate_command_supports_json_output_for_path_selection() {
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "relate",
            "src/rust_feature.rs",
            fixture_path("passing").to_str().expect("utf8 path"),
            "--format",
            "json",
        ])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(json["selection"]["kind"], "path");
    assert_eq!(json["selection"]["query"], "src/rust_feature.rs");
    assert_eq!(
        json["direct_matches"]["traces"][0]["owner_id"],
        "FEAT-TRACE-001"
    );
    assert_eq!(json["features"][0]["id"], "FEAT-TRACE-001");
    assert_eq!(json["requirements"][0]["id"], "REQ-TRACE-001");
}

#[test]
fn relate_command_matches_source_symbols() {
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("relate")
        .arg("feature_trace_rust")
        .arg(fixture_path("passing"))
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Selection: symbol feature_trace_rust"));
    assert!(stdout.contains("feature FEAT-TRACE-001 implementation rust\tsrc/rust_feature.rs"));
    assert!(stdout.contains("(direct match)"));
}

#[test]
fn relate_command_surfaces_sparse_graph_gaps() {
    let workspace = write_gap_fixture_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("relate")
        .arg("REQ-GAP-001")
        .arg(workspace.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("requirement `REQ-GAP-001` does not link to any policies"));
    assert!(stdout.contains("requirement `REQ-GAP-001` does not link to any features"));
    assert!(stdout.contains("requirement `REQ-GAP-001` does not declare any test traces"));
}

#[test]
fn relate_command_rejects_unknown_selectors() {
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("relate")
        .arg("missing_selector")
        .arg(fixture_path("passing"))
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("did not match any definition ID, traced path, or traced source symbol")
    );
}
