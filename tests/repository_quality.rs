use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("repository file should exist")
}

#[test]
// REQ-CORE-005
fn repository_declares_precommit_and_quality_gates() {
    let precommit = read_file(".pre-commit-config.yaml");
    let quality_script = read_file("scripts/ci/quality-gates.sh");
    let ci_workflow = read_file(".github/workflows/ci.yml");

    assert!(precommit.contains("FEAT-QUALITY-001"));
    assert!(precommit.contains("shellcheck"));
    assert!(precommit.contains("syu-validate-self"));
    assert!(precommit.contains("syu-quality-gates"));
    assert!(precommit.contains("syu-coverage-gate"));

    assert!(quality_script.contains("FEAT-QUALITY-001"));
    assert!(quality_script.contains("run_quality_gates"));
    assert!(quality_script.contains("cargo fmt --all --check"));
    assert!(quality_script.contains("cargo clippy --all-targets --all-features -- -D warnings"));
    assert!(quality_script.contains("cargo test"));
    assert!(quality_script.contains("cargo run -- validate ."));

    assert!(ci_workflow.contains("FEAT-QUALITY-001"));
    assert!(ci_workflow.contains("precommit:"));
    assert!(ci_workflow.contains("quality:"));
    assert!(ci_workflow.contains("actionlint:"));
    assert!(ci_workflow.contains("dependency-audit:"));
    assert!(ci_workflow.contains("installer-smoke:"));
    assert!(ci_workflow.contains("--hook-stage pre-commit"));
    assert!(ci_workflow.contains("--hook-stage pre-push"));
    assert!(ci_workflow.contains("scripts/ci/quality-gates.sh"));
    assert!(ci_workflow.contains("cargo audit"));
    assert!(ci_workflow.contains("scripts/ci/installer-smoke.sh"));
}

#[test]
// REQ-CORE-006
fn repository_declares_coverage_gate_at_one_hundred_percent() {
    let coverage_script = read_file("scripts/ci/coverage.sh");
    let ci_workflow = read_file(".github/workflows/ci.yml");

    assert!(coverage_script.contains("FEAT-QUALITY-001"));
    assert!(coverage_script.contains("run_coverage"));
    assert!(coverage_script.contains("LINE_THRESHOLD=100"));
    assert!(coverage_script.contains("--fail-under-lines 100"));
    assert!(coverage_script.contains("cargo llvm-cov"));

    assert!(ci_workflow.contains("coverage:"));
    assert!(ci_workflow.contains("scripts/ci/coverage.sh lcov"));
    assert!(ci_workflow.contains("cargo-llvm-cov"));
}

#[test]
// REQ-CORE-007
fn repository_declares_release_automation() {
    let release_please = read_file(".github/workflows/release-please.yml");
    let release_artifacts = read_file(".github/workflows/release-artifacts.yml");
    let package_script = read_file("scripts/ci/package-release.sh");
    let publish_script = read_file("scripts/ci/publish-package.sh");
    let release_config = read_file("release-please-config.json");
    let beta_config = read_file("release-please-config.beta.json");
    let alpha_config = read_file("release-please-config.alpha.json");
    let manifest = read_file(".release-please-manifest.json");

    assert!(release_please.contains("FEAT-RELEASE-001"));
    assert!(release_please.contains("googleapis/release-please-action@v4.4.0"));
    assert!(release_please.contains("release-please-stable"));
    assert!(release_please.contains("release-please-beta"));
    assert!(release_please.contains("release-please-alpha"));
    assert!(release_please.contains("release-please skipped"));

    assert!(release_artifacts.contains("FEAT-RELEASE-001"));
    assert!(release_artifacts.contains("release-artifacts"));
    assert!(release_artifacts.contains("x86_64-unknown-linux-gnu"));
    assert!(release_artifacts.contains("aarch64-apple-darwin"));
    assert!(release_artifacts.contains("x86_64-pc-windows-msvc"));
    assert!(release_artifacts.contains("PACKAGE_REPOSITORY"));
    assert!(release_artifacts.contains("publish-package.sh"));

    assert!(package_script.contains("FEAT-RELEASE-001"));
    assert!(package_script.contains("package_release_artifact"));
    assert!(package_script.contains("write_sha256"));
    assert!(publish_script.contains("FEAT-RELEASE-001"));
    assert!(publish_script.contains("publish_package_artifact"));
    assert!(publish_script.contains("oras push"));

    assert!(release_config.contains("\"release-type\": \"rust\""));
    assert!(release_config.contains("\"package-name\": \"syu\""));
    assert!(release_config.contains("\"skip-changelog\": true"));
    assert!(release_config.contains("\"changelog-type\": \"github\""));
    assert!(release_config.contains("\"initial-version\": \"0.0.1\""));
    assert!(beta_config.contains("\"prerelease\": true"));
    assert!(beta_config.contains("\"prerelease-type\": \"beta\""));
    assert!(alpha_config.contains("\"prerelease\": true"));
    assert!(alpha_config.contains("\"prerelease-type\": \"alpha\""));
    assert!(manifest.contains("\".\": \"0.0.0\""));
    assert!(!repo_root().join("CHANGELOG.md").exists());
}

#[test]
// REQ-CORE-008
fn repository_declares_installer_contract() {
    let installer = read_file("scripts/install-syu.sh");
    let installer_smoke = read_file("scripts/ci/installer-smoke.sh");
    let mock_registry = read_file("scripts/ci/mock_package_registry.py");
    let readme = read_file("README.md");

    assert!(installer.contains("FEAT-INSTALL-001"));
    assert!(installer.contains("resolve_repository"));
    assert!(installer.contains("resolve_target_triple"));
    assert!(installer.contains("resolve_package_repository"));
    assert!(installer.contains("resolve_package_tag"));
    assert!(installer.contains("download_package_archive"));
    assert!(installer.contains("install_syu"));
    assert!(installer.contains("SYU_REPOSITORY"));
    assert!(installer.contains("SYU_INSTALL_DIR"));
    assert!(installer.contains("SYU_PACKAGE_REPOSITORY"));
    assert!(installer.contains("ugoite/syu"));
    assert!(installer.contains("ghcr.io"));
    assert!(installer_smoke.contains("FEAT-INSTALL-001"));
    assert!(installer_smoke.contains("run_install_case"));
    assert!(mock_registry.contains("FEAT-INSTALL-001"));
    assert!(mock_registry.contains("build_artifacts"));

    assert!(readme.contains("install-syu.sh"));
    assert!(readme.contains("ugoite/syu"));
    assert!(readme.contains("SYU_VERSION"));
    assert!(readme.contains("GitHub Packages"));
}

#[test]
// REQ-CORE-010
fn repository_declares_documentation_guides() {
    let readme = read_file("README.md");
    let concepts = read_file("docs/guide/concepts.md");
    let getting_started = read_file("docs/guide/getting-started.md");
    let configuration = read_file("docs/guide/configuration.md");

    assert!(readme.contains("docs/guide/concepts.md"));
    assert!(readme.contains("syu init"));
    assert!(readme.contains("syu validate"));
    assert!(readme.contains("examples/polyglot"));

    assert!(concepts.contains("philosophy"));
    assert!(concepts.contains("policy"));
    assert!(concepts.contains("requirements"));
    assert!(concepts.contains("features"));
    assert!(concepts.contains("planned"));
    assert!(concepts.contains("implemented"));
    assert!(getting_started.contains("syu validate . --fix"));
    assert!(getting_started.contains("status: implemented"));
    assert!(configuration.contains("validate.default_fix"));
    assert!(configuration.contains("validate.allow_planned"));
    assert!(configuration.contains("version: 0.0.1"));
}

#[test]
// REQ-CORE-011
fn repository_declares_devcontainer_configuration() {
    let devcontainer = read_file(".devcontainer/devcontainer.json");
    assert!(devcontainer.contains("FEAT-CONTRIB-001"));
    assert!(devcontainer.contains("cargo install cargo-llvm-cov --locked"));
    assert!(devcontainer.contains("ghcr.io/devcontainers/features/python:1"));
}

#[test]
// REQ-CORE-012
fn repository_ships_example_workspaces() {
    let rust_example_requirement = read_file("examples/rust-only/docs/spec/requirements/core.yaml");
    let rust_example_config = read_file("examples/rust-only/syu.yaml");
    let python_example_requirement =
        read_file("examples/python-only/docs/spec/requirements/core.yaml");
    let polyglot_feature = read_file("examples/polyglot/docs/spec/features/polyglot.yaml");
    let example_tests = read_file("tests/example_workspaces.rs");

    assert!(rust_example_requirement.contains("REQ-RUST-001"));
    assert!(rust_example_config.contains("version: 0.0.1"));
    assert!(python_example_requirement.contains("REQ-PY-001"));
    assert!(polyglot_feature.contains("FEAT-MIX-001"));
    assert!(polyglot_feature.contains("status: implemented"));
    assert!(example_tests.contains("rust_only_example_validates"));
    assert!(example_tests.contains("python_only_example_validates"));
    assert!(example_tests.contains("polyglot_example_validates"));
}
