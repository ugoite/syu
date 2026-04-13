// REQ-CORE-021

use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;
use std::{fs, path::Path, process::Command};
use tempfile::{TempDir, tempdir};

fn write_history_workspace() -> TempDir {
    let tempdir = tempdir().expect("tempdir should exist");
    let docs_root = tempdir.path().join("docs/syu");
    fs::create_dir_all(docs_root.join("philosophy")).expect("philosophy dir");
    fs::create_dir_all(docs_root.join("policies")).expect("policies dir");
    fs::create_dir_all(docs_root.join("requirements")).expect("requirements dir");
    fs::create_dir_all(docs_root.join("features/cli")).expect("features dir");
    fs::create_dir_all(tempdir.path().join("src")).expect("src dir");

    fs::write(
        docs_root.join("philosophy/foundation.yaml"),
        "category: Philosophy\nversion: 1\nlanguage: en\nphilosophies:\n  - id: PHIL-HIST-001\n    title: History should stay explorable.\n    product_design_principle: Keep commit history close to trace links.\n    coding_guideline: Prefer one-command repository history lookups.\n    linked_policies:\n      - POL-HIST-001\n",
    )
    .expect("philosophy file");
    fs::write(
        docs_root.join("policies/policies.yaml"),
        "category: Policies\nversion: 1\nlanguage: en\npolicies:\n  - id: POL-HIST-001\n    title: History should be reachable from traces.\n    summary: Git history is useful when it is derived from checked-in trace metadata.\n    description: The repository history should be explorable from requirement and feature traces.\n    linked_philosophies:\n      - PHIL-HIST-001\n    linked_requirements:\n      - REQ-HIST-001\n",
    )
    .expect("policy file");
    fs::write(
        docs_root.join("requirements/core.yaml"),
        "category: Core\nprefix: REQ-HIST\n\nrequirements:\n  - id: REQ-HIST-001\n    title: Requirement history lookup\n    description: Requirement history should show the traced test and checked-in definition.\n    priority: medium\n    status: implemented\n    linked_policies:\n      - POL-HIST-001\n    linked_features:\n      - FEAT-HIST-001\n    tests:\n      rust:\n        - file: src/history_tests.rs\n          symbols:\n            - requirement_history_test\n",
    )
    .expect("requirement file");
    fs::write(
        docs_root.join("features/features.yaml"),
        "version: \"1\"\nfiles:\n  - kind: history\n    file: cli/history.yaml\n",
    )
    .expect("feature registry");
    fs::write(
        docs_root.join("features/cli/history.yaml"),
        "category: History\nversion: 1\nfeatures:\n  - id: FEAT-HIST-001\n    title: Feature history lookup\n    summary: Feature history should show the traced implementation and checked-in definition.\n    status: implemented\n    linked_requirements:\n      - REQ-HIST-001\n    implementations:\n      rust:\n        - file: src/history_feature.rs\n          symbols:\n            - feature_history\n",
    )
    .expect("feature file");
    fs::write(
        tempdir.path().join("src/history_tests.rs"),
        "// REQ-HIST-001\nfn requirement_history_test() {}\n",
    )
    .expect("history test file");
    fs::write(
        tempdir.path().join("src/history_feature.rs"),
        "// FEAT-HIST-001\nfn feature_history() {}\n",
    )
    .expect("history feature file");

    init_git_repository(tempdir.path());
    update_file(
        &tempdir.path().join("docs/syu/requirements/core.yaml"),
        "category: Core\nprefix: REQ-HIST\n\nrequirements:\n  - id: REQ-HIST-001\n    title: Requirement history lookup\n    description: Requirement history should show the traced test, checked-in definition, and maintenance story.\n    priority: medium\n    status: implemented\n    linked_policies:\n      - POL-HIST-001\n    linked_features:\n      - FEAT-HIST-001\n    tests:\n      rust:\n        - file: src/history_tests.rs\n          symbols:\n            - requirement_history_test\n",
    );
    git_commit(tempdir.path(), "docs: refine traced requirement history");

    update_file(
        &tempdir.path().join("src/history_tests.rs"),
        "// REQ-HIST-001\nfn requirement_history_test() {\n    // traced test adjustment\n}\n",
    );
    git_commit(tempdir.path(), "test: adjust traced requirement coverage");

    update_file(
        &tempdir.path().join("src/history_feature.rs"),
        "// FEAT-HIST-001\nfn feature_history() {\n    // traced implementation adjustment\n}\n",
    );
    git_commit(tempdir.path(), "feat: update traced implementation");

    tempdir
}

fn init_git_repository(workspace: &Path) {
    git(workspace, &["init"]);
    git(workspace, &["config", "user.name", "Test User"]);
    git(workspace, &["config", "user.email", "test@example.com"]);
    git(workspace, &["add", "."]);
    git(
        workspace,
        &["commit", "-m", "chore: initial history fixture"],
    );
}

fn git_commit(workspace: &Path, summary: &str) {
    git(workspace, &["add", "."]);
    git(workspace, &["commit", "-m", summary]);
}

fn git(workspace: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .expect("git should run");

    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn update_file(path: &Path, contents: &str) {
    fs::write(path, contents).expect("file should update");
}

fn write_fake_git_for_log_failure(script_dir: &Path) {
    let script_path = script_dir.join("git");
    fs::write(
        &script_path,
        "#!/bin/sh\nset -eu\nworkspace=\"$2\"\ncommand_name=\"$3\"\nif [ \"$command_name\" = \"rev-parse\" ]; then\n  printf '%s\\n' \"$workspace\"\n  exit 0\nfi\nif [ \"$command_name\" = \"log\" ]; then\n  echo 'synthetic git log failure' >&2\n  exit 1\nfi\necho 'unexpected git invocation' >&2\nexit 1\n",
    )
    .expect("fake git script");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&script_path)
            .expect("fake git metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).expect("fake git permissions");
    }
}

#[test]
fn log_command_renders_requirement_definition_and_test_history() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("log")
        .arg("REQ-HIST-001")
        .arg(workspace.path())
        .output()
        .expect("command should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("History: requirement REQ-HIST-001"));
    assert!(stdout.contains("definition\tdocs/syu/requirements/core.yaml"));
    assert!(stdout.contains("test\tsrc/history_tests.rs\trust\t[`requirement_history_test`]"));
    assert!(stdout.contains("docs: refine traced requirement history"));
    assert!(stdout.contains("test: adjust traced requirement coverage"));
    assert!(
        !stdout.contains("feat: update traced implementation"),
        "requirement history should not include feature implementation commits"
    );
}

#[test]
fn log_command_supports_json_kind_and_path_filters() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "log",
            "FEAT-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
            "--kind",
            "implementation",
            "--path",
            "src",
            "--format",
            "json",
        ])
        .output()
        .expect("command should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("output should be valid JSON");
    assert_eq!(json["id"], "FEAT-HIST-001");
    assert_eq!(json["kind"], "implementation");
    assert_eq!(json["path_filter"], "src");
    let commits = json["commits"].as_array().expect("commit array");
    assert_eq!(commits.len(), 2);
    assert_eq!(
        json["commits"][0]["summary"],
        "feat: update traced implementation"
    );
    assert_eq!(json["commits"][0]["reasons"][0]["kind"], "implementation");
    assert_eq!(
        json["commits"][0]["reasons"][0]["path"],
        "src/history_feature.rs"
    );
    assert_eq!(
        json["commits"][1]["summary"],
        "chore: initial history fixture"
    );
}

#[test]
fn log_command_rejects_zero_limit() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "log",
            "REQ-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
            "--limit",
            "0",
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("greater than zero"));
}

#[test]
fn log_command_rejects_unknown_ids() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "log",
            "REQ-HIST-404",
            workspace.path().to_str().expect("utf8 path"),
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("was not found"));
}

#[test]
fn log_command_rejects_non_trace_layers() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "log",
            "PHIL-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requirement and feature IDs only"));
}

#[test]
fn log_command_rejects_incompatible_requirement_kinds() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "log",
            "REQ-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
            "--kind",
            "implementation",
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--kind implementation"));
}

#[test]
fn log_command_rejects_incompatible_feature_kinds() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "log",
            "FEAT-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
            "--kind",
            "test",
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--kind test"));
}

#[test]
fn log_command_rejects_empty_path_selections() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "log",
            "FEAT-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
            "--kind",
            "implementation",
            "--path",
            "docs",
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no tracked history paths remain"));
}

#[test]
fn log_command_rejects_absolute_path_filters_outside_workspace() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .args([
            "log",
            "REQ-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
            "--path",
            "/tmp/outside-history-path",
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("must stay inside workspace"));
}

#[test]
fn log_command_reports_git_spawn_errors() {
    let workspace = write_history_workspace();
    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .env("PATH", "")
        .args([
            "log",
            "REQ-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed to run `git rev-parse`"));
}

#[test]
fn log_command_reports_git_log_failures() {
    let workspace = write_history_workspace();
    let fake_bin = tempdir().expect("tempdir should exist");
    write_fake_git_for_log_failure(fake_bin.path());

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .env("PATH", fake_bin.path())
        .args([
            "log",
            "REQ-HIST-001",
            workspace.path().to_str().expect("utf8 path"),
        ])
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("failed to read git history"),
        "stderr should explain git log failures"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("synthetic git log failure"),
        "stderr should preserve git stderr"
    );
}

#[test]
fn log_command_requires_a_git_repository() {
    let workspace = tempdir().expect("tempdir should exist");
    fs::create_dir_all(workspace.path().join("docs/syu/philosophy")).expect("philosophy dir");
    fs::create_dir_all(workspace.path().join("docs/syu/policies")).expect("policies dir");
    fs::create_dir_all(workspace.path().join("docs/syu/requirements")).expect("requirements dir");
    fs::create_dir_all(workspace.path().join("docs/syu/features")).expect("features dir");
    fs::write(
        workspace.path().join("docs/syu/philosophy/foundation.yaml"),
        "category: Philosophy\nversion: 1\nlanguage: en\nphilosophies:\n  - id: PHIL-001\n    title: Minimal\n    product_design_principle: Minimal.\n    coding_guideline: Minimal.\n    linked_policies: []\n",
    )
    .expect("philosophy file");
    fs::write(
        workspace.path().join("docs/syu/policies/policies.yaml"),
        "category: Policies\nversion: 1\nlanguage: en\npolicies:\n  - id: POL-001\n    title: Minimal\n    summary: Minimal.\n    description: Minimal.\n    linked_philosophies: []\n    linked_requirements: []\n",
    )
    .expect("policy file");
    fs::write(
        workspace.path().join("docs/syu/requirements/core.yaml"),
        "category: Core\nprefix: REQ\n\nrequirements:\n  - id: REQ-001\n    title: Minimal\n    description: Minimal.\n    priority: medium\n    status: implemented\n    linked_policies: []\n    linked_features: []\n    tests:\n      rust:\n        - file: src/minimal.rs\n          symbols:\n            - minimal_requirement\n",
    )
    .expect("requirement file");
    fs::write(
        workspace.path().join("docs/syu/features/features.yaml"),
        "version: \"1\"\nfiles:\n  - kind: minimal\n    file: minimal.yaml\n",
    )
    .expect("feature registry");
    fs::write(
        workspace.path().join("docs/syu/features/minimal.yaml"),
        "category: Minimal\nversion: 1\nfeatures:\n  - id: FEAT-001\n    title: Minimal\n    summary: Minimal.\n    status: implemented\n    linked_requirements: []\n    implementations: {}\n",
    )
    .expect("feature file");

    let output = Command::cargo_bin("syu")
        .expect("binary should build")
        .arg("log")
        .arg("REQ-001")
        .arg(workspace.path())
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("is not inside a Git repository"),
        "stderr should explain the git repository requirement"
    );
}
