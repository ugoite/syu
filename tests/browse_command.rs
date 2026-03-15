// REQ-CORE-015

use assert_cmd::{Command, cargo::CommandCargoExt};
use std::{path::PathBuf, process::Command as StdCommand};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workspaces")
        .join(name)
}

#[test]
fn browse_command_shows_linked_details_without_failing_validation_errors() {
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("browse")
        .arg(fixture_path("failing"))
        .write_stdin("5\n1\n\n0\n0\n")
        .output()
        .expect("browse command should run");

    assert!(output.status.success(), "browse UI should not fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("errors ("));
    assert!(stdout.contains("missing-reference"));
    assert!(stdout.contains("Linked definitions must exist"));
}

#[test]
fn bare_syu_prints_help_when_not_attached_to_a_terminal() {
    let output = StdCommand::cargo_bin("syu")
        .expect("binary should build")
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("browse"));
    assert!(stdout.contains("validate"));
}
