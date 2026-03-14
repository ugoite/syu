use assert_cmd::cargo::CommandCargoExt;
use std::{fs, path::PathBuf, process::Command};
use tempfile::tempdir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workspaces")
        .join(name)
}

#[test]
// REQ-CORE-004
fn report_command_generates_markdown_output() {
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("report")
        .arg(fixture_path("passing"))
        .output()
        .expect("command should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# syu validation report"));
    assert!(stdout.contains("Requirement-to-test traceability: 3/3"));
}

#[test]
// REQ-CORE-004
fn report_command_writes_markdown_file() {
    let tempdir = tempdir().expect("tempdir should be created");
    let output_file = tempdir.path().join("report.md");

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("report")
        .arg(fixture_path("failing"))
        .arg("--output")
        .arg(&output_file)
        .output()
        .expect("command should run");

    assert!(!output.status.success(), "failing fixture should fail");
    let report = fs::read_to_string(&output_file).expect("report file should exist");
    assert!(report.contains("REQ-FAIL-001"));
    assert!(report.contains("REQ-MISSING-999"));
}
