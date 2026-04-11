use assert_cmd::cargo::CommandCargoExt;
use std::{fs, path::Path, process::Command};
use tempfile::tempdir;

fn init_workspace(workspace: &Path, extra_args: &[&str]) {
    let mut command = Command::cargo_bin("syu").expect("binary should build");
    command.arg("init").arg(workspace);
    command.args(extra_args);
    let output = command.output().expect("init should run");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
// REQ-CORE-020
fn add_command_appends_philosophy_to_the_default_file() {
    let tempdir = tempdir().expect("tempdir should exist");
    let workspace = tempdir.path().join("workspace");
    init_workspace(&workspace, &[]);

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args(["add", "philosophy", "PHIL-002"])
        .arg(&workspace)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let file = fs::read_to_string(workspace.join("docs/syu/philosophy/foundation.yaml"))
        .expect("philosophy file should exist");
    assert!(file.contains("- id: PHIL-002"));
    assert!(file.contains("title: New philosophy"));
    assert!(file.contains("linked_policies: []"));
}

#[test]
// REQ-CORE-020
fn add_command_creates_requirement_files_from_the_id_prefix() {
    let tempdir = tempdir().expect("tempdir should exist");
    let workspace = tempdir.path().join("workspace");
    init_workspace(&workspace, &[]);

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args(["add", "requirement", "REQ-AUTH-001"])
        .arg(&workspace)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let file = workspace.join("docs/syu/requirements/auth/auth.yaml");
    let contents = fs::read_to_string(&file).expect("requirement file should exist");
    assert!(contents.contains("prefix: REQ-AUTH"));
    assert!(contents.contains("id: REQ-AUTH-001"));
    assert!(contents.contains("status: planned"));
}

#[test]
// REQ-CORE-020
fn add_command_updates_the_feature_registry_for_new_feature_files() {
    let tempdir = tempdir().expect("tempdir should exist");
    let workspace = tempdir.path().join("workspace");
    init_workspace(&workspace, &[]);

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args(["add", "feature", "FEAT-AUTH-LOGIN-001", "--kind", "auth"])
        .arg(&workspace)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let feature_file = workspace.join("docs/syu/features/auth/login.yaml");
    let registry = fs::read_to_string(workspace.join("docs/syu/features/features.yaml"))
        .expect("feature registry should exist");
    assert!(feature_file.exists(), "feature file should be created");
    assert!(registry.contains("kind: auth"));
    assert!(registry.contains("file: auth/login.yaml"));
}

#[test]
// REQ-CORE-020
fn add_command_honors_explicit_files_in_custom_spec_roots() {
    let tempdir = tempdir().expect("tempdir should exist");
    let workspace = tempdir.path().join("workspace");
    init_workspace(&workspace, &["--spec-root", "docs/spec"]);

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "add",
            "feature",
            "FEAT-AUTH-001",
            "--kind",
            "auth",
            "--file",
            "docs/spec/features/auth/flows.yaml",
        ])
        .arg(&workspace)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let feature_file = workspace.join("docs/spec/features/auth/flows.yaml");
    let registry = fs::read_to_string(workspace.join("docs/spec/features/features.yaml"))
        .expect("feature registry should exist");
    assert!(
        feature_file.exists(),
        "explicit feature file should be created"
    );
    assert!(registry.contains("file: auth/flows.yaml"));
}

#[test]
// REQ-CORE-020
fn add_command_rejects_duplicate_ids() {
    let tempdir = tempdir().expect("tempdir should exist");
    let workspace = tempdir.path().join("workspace");
    init_workspace(&workspace, &[]);

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args(["add", "requirement", "REQ-001"])
        .arg(&workspace)
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
}

#[test]
// REQ-CORE-020
fn add_command_rejects_feature_kind_for_non_feature_layers() {
    let tempdir = tempdir().expect("tempdir should exist");
    let workspace = tempdir.path().join("workspace");
    init_workspace(&workspace, &[]);

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args(["add", "requirement", "REQ-AUTH-001", "--kind", "auth"])
        .arg(&workspace)
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("only supported when scaffolding features")
    );
}
