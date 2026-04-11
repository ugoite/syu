// REQ-CORE-009
// FEAT-INIT-004
// FEAT-INIT-002

use anyhow::{Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    cli::{InitArgs, OutputFormat, StarterTemplate},
    command::shell_quote_path,
    config::{SyuConfig, render_config},
};

#[cfg(test)]
const GENERATED_PATHS: &[&str] = &[
    "syu.yaml",
    "docs/syu/philosophy/foundation.yaml",
    "docs/syu/policies/policies.yaml",
    "docs/syu/requirements/core/core.yaml",
    "docs/syu/features/features.yaml",
    "docs/syu/features/core/core.yaml",
];

// FEAT-INIT-001
pub fn run_init_command(args: &InitArgs) -> Result<i32> {
    let workspace = prepare_workspace_root(&args.workspace)?;
    let project_name = args
        .name
        .clone()
        .unwrap_or_else(|| infer_project_name(&workspace));

    let files = scaffold_files(&project_name, args.template);
    ensure_writable_targets(
        &workspace,
        files.iter().map(|(path, _)| PathBuf::from(path)),
        args.force,
    )?;

    for (relative_path, contents) in &files {
        let full_path = workspace.join(relative_path);
        let parent = full_path
            .parent()
            .expect("generated scaffold paths should always have a parent");
        fs::create_dir_all(parent)?;
        fs::write(full_path, contents)?;
    }

    let created_paths: Vec<&str> = files.iter().map(|(path, _)| path.as_str()).collect();

    match args.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "workspace": workspace.display().to_string(),
                    "created_files": created_paths,
                })
            );
        }
        OutputFormat::Text => {
            let workspace_arg = shell_quote_path(&workspace);
            let spec_root = workspace.join("docs/syu");
            let requirement_path = workspace.join(requirement_document_path(args.template));
            let feature_path = workspace.join(feature_document_path(args.template));
            println!("initialized syu workspace at {}", workspace.display());
            println!();
            println!("Created files:");
            for path in &created_paths {
                println!("  {path}");
            }
            println!();
            println!("What to do next:");
            println!(
                "  1. Edit the spec files in {}/ to describe your project",
                spec_root.display()
            );
            println!("     - docs/syu/philosophy/foundation.yaml  (core principles)");
            println!("     - docs/syu/policies/policies.yaml       (governance rules)");
            println!(
                "     - {}  (concrete requirements)",
                path_label(
                    requirement_path
                        .strip_prefix(&workspace)
                        .expect("workspace-relative requirement path")
                )
            );
            println!(
                "     - {}  (feature definitions)",
                path_label(
                    feature_path
                        .strip_prefix(&workspace)
                        .expect("workspace-relative feature path")
                )
            );
            println!("  2. Run `syu validate {workspace_arg}` to check your spec for consistency");
            println!(
                "  3. Run `syu browse {workspace_arg}` for terminal exploration, or `syu app {workspace_arg}` for the browser UI"
            );
            println!("  4. Commit the generated files to version control");
        }
    }

    Ok(0)
}

fn prepare_workspace_root(path: &Path) -> Result<PathBuf> {
    if path.exists() && !path.is_dir() {
        bail!("workspace path `{}` is not a directory", path.display());
    }

    fs::create_dir_all(path)?;
    path.canonicalize().map_err(Into::into)
}

fn infer_project_name(workspace: &Path) -> String {
    workspace
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("project")
        .to_string()
}

fn ensure_writable_targets(
    workspace: &Path,
    targets: impl Iterator<Item = PathBuf>,
    force: bool,
) -> Result<()> {
    if force {
        return Ok(());
    }

    let existing: Vec<_> = targets
        .map(|target| workspace.join(target))
        .filter(|path| path.exists())
        .collect();

    if existing.is_empty() {
        return Ok(());
    }

    let paths = existing
        .iter()
        .map(|path| format!("`{}`", path.display()))
        .collect::<Vec<_>>()
        .join(", ");
    bail!("refusing to overwrite existing files without --force: {paths}");
}

fn scaffold_files(project_name: &str, template: StarterTemplate) -> Vec<(String, String)> {
    vec![
        (
            "syu.yaml".to_string(),
            render_default_config().expect("config template should render"),
        ),
        (
            "docs/syu/philosophy/foundation.yaml".to_string(),
            philosophy_template(project_name, template),
        ),
        (
            "docs/syu/policies/policies.yaml".to_string(),
            policy_template(project_name, template),
        ),
        (
            requirement_document_path(template).to_string(),
            requirement_template(project_name, template),
        ),
        (
            "docs/syu/features/features.yaml".to_string(),
            feature_registry_template(template),
        ),
        (
            feature_document_path(template).to_string(),
            feature_template(project_name, template),
        ),
    ]
}

fn render_default_config() -> Result<String> {
    render_config(&SyuConfig::default())
}

fn philosophy_template(project_name: &str, template: StarterTemplate) -> String {
    match template {
        StarterTemplate::Generic => format!(
            "category: Philosophy\nversion: 1\nlanguage: en\n\nphilosophies:\n  - id: PHIL-001\n    title: {project_name} should turn intent into executable agreements\n    product_design_principle: |\n      The project should keep philosophy, policy, requirements, and features\n      explicit enough that contributors can validate changes mechanically.\n    coding_guideline: |\n      Prefer stable IDs, typed data, and explicit traceability over conventions\n      that live only in contributor memory.\n    linked_policies:\n      - POL-001\n"
        ),
        StarterTemplate::RustOnly => format!(
            "category: Philosophy\nversion: 1\nlanguage: en\n\nphilosophies:\n  - id: PHIL-RUST-001\n    title: {project_name} should keep Rust traces explicit\n    product_design_principle: |\n      The project should keep Rust-first traceability small, reviewable, and\n      obvious to contributors reading the code.\n    coding_guideline: |\n      Prefer stable IDs and Rust doc comments on traced symbols from the first\n      requirement onward.\n    linked_policies:\n      - POL-RUST-001\n"
        ),
        StarterTemplate::PythonOnly => format!(
            "category: Philosophy\nversion: 1\nlanguage: en\n\nphilosophies:\n  - id: PHIL-PY-001\n    title: {project_name} should keep Python traces explicit\n    product_design_principle: |\n      The project should keep Python traceability small, reviewable, and easy\n      to understand from docstrings alone.\n    coding_guideline: |\n      Prefer stable IDs and clear docstrings on traced Python symbols from the\n      first requirement onward.\n    linked_policies:\n      - POL-PY-001\n"
        ),
        StarterTemplate::Polyglot => format!(
            "category: Philosophy\nversion: 1\nlanguage: en\n\nphilosophies:\n  - id: PHIL-MIX-001\n    title: {project_name} should keep polyglot traces coherent\n    product_design_principle: |\n      The project should prove that one specification can stay understandable\n      even when implementation and tests span multiple languages.\n    coding_guideline: |\n      Prefer stable IDs and short language-native docs on every traced symbol.\n    linked_policies:\n      - POL-MIX-001\n"
        ),
    }
}

fn policy_template(project_name: &str, template: StarterTemplate) -> String {
    match template {
        StarterTemplate::Generic => format!(
            "category: Policies\nversion: 1\nlanguage: en\n\npolicies:\n  - id: POL-001\n    title: Every change in {project_name} should remain traceable\n    summary: Define rules that turn philosophy into a verifiable workflow.\n    description: |\n      A specification entry is only useful when contributors can trace it to\n      concrete requirements, features, code, and tests inside the repository.\n    linked_philosophies:\n      - PHIL-001\n    linked_requirements:\n      - REQ-001\n"
        ),
        StarterTemplate::RustOnly => format!(
            "category: Policies\nversion: 1\nlanguage: en\n\npolicies:\n  - id: POL-RUST-001\n    title: Rust requirement and feature traces must stay documented in {project_name}\n    summary: Start with a Rust-first workflow that stays explicit in code review.\n    description: |\n      Rust requirement and feature traces should point to symbols whose doc\n      comments carry both the stable ID and a short explanation.\n    linked_philosophies:\n      - PHIL-RUST-001\n    linked_requirements:\n      - REQ-RUST-001\n"
        ),
        StarterTemplate::PythonOnly => format!(
            "category: Policies\nversion: 1\nlanguage: en\n\npolicies:\n  - id: POL-PY-001\n    title: Python requirement and feature traces must stay documented in {project_name}\n    summary: Start with a Python-first workflow that stays explicit in docstrings.\n    description: |\n      Python requirement and feature traces should point to symbols whose\n      docstrings carry both the stable ID and a short explanation.\n    linked_philosophies:\n      - PHIL-PY-001\n    linked_requirements:\n      - REQ-PY-001\n"
        ),
        StarterTemplate::Polyglot => format!(
            "category: Policies\nversion: 1\nlanguage: en\n\npolicies:\n  - id: POL-MIX-001\n    title: Polyglot requirement and feature traces must stay verifiable in {project_name}\n    summary: Start with one specification flow that can grow across languages.\n    description: |\n      The starter workspace should make it obvious how one requirement and one\n      feature can stay linked even when implementation later spans Rust,\n      Python, and TypeScript.\n    linked_philosophies:\n      - PHIL-MIX-001\n    linked_requirements:\n      - REQ-MIX-001\n"
        ),
    }
}

fn requirement_template(project_name: &str, template: StarterTemplate) -> String {
    match template {
        StarterTemplate::Generic => format!(
            "category: Core Requirements\nprefix: REQ\n\nrequirements:\n  - id: REQ-001\n    title: Bootstrap {project_name} with a four-layer specification\n    description: |\n      The project should keep philosophy, policy, requirements, and features in\n      YAML so contributors can evolve behavior deliberately.\n    priority: high\n    status: planned\n    linked_policies:\n      - POL-001\n    linked_features:\n      - FEAT-001\n    tests: {{}}\n"
        ),
        StarterTemplate::RustOnly => format!(
            "category: Rust Requirements\nprefix: REQ-RUST\n\nrequirements:\n  - id: REQ-RUST-001\n    title: Bootstrap {project_name} with Rust-first trace conventions\n    description: |\n      The project should start with a Rust-oriented requirement that can later\n      claim documented Rust test symbols without restructuring the workspace.\n    priority: high\n    status: planned\n    linked_policies:\n      - POL-RUST-001\n    linked_features:\n      - FEAT-RUST-001\n    tests: {{}}\n"
        ),
        StarterTemplate::PythonOnly => format!(
            "category: Python Requirements\nprefix: REQ-PY\n\nrequirements:\n  - id: REQ-PY-001\n    title: Bootstrap {project_name} with Python-first trace conventions\n    description: |\n      The project should start with a Python-oriented requirement that can later\n      claim documented Python test symbols without restructuring the workspace.\n    priority: high\n    status: planned\n    linked_policies:\n      - POL-PY-001\n    linked_features:\n      - FEAT-PY-001\n    tests: {{}}\n"
        ),
        StarterTemplate::Polyglot => format!(
            "category: Polyglot Requirements\nprefix: REQ-MIX\n\nrequirements:\n  - id: REQ-MIX-001\n    title: Bootstrap {project_name} with polyglot trace conventions\n    description: |\n      The project should start with one requirement that can later trace into\n      Rust, Python, and TypeScript without changing the layered layout.\n    priority: high\n    status: planned\n    linked_policies:\n      - POL-MIX-001\n    linked_features:\n      - FEAT-MIX-001\n    tests: {{}}\n"
        ),
    }
}

fn feature_registry_template(template: StarterTemplate) -> String {
    format!(
        "version: \"{}\"\nupdated: \"generated by syu init\"\n\nfiles:\n  - kind: {}\n    file: {}\n",
        SyuConfig::default().version,
        feature_kind(template),
        feature_document_path(template)
            .strip_prefix("docs/syu/features/")
            .expect("feature path should stay under docs/syu/features")
    )
}

fn feature_template(project_name: &str, template: StarterTemplate) -> String {
    match template {
        StarterTemplate::Generic => format!(
            "category: Core Features\nversion: 1\n\nfeatures:\n  - id: FEAT-001\n    title: Bootstrap the {project_name} spec workspace\n    summary: Provide a starter structure that contributors can extend.\n    status: planned\n    linked_requirements:\n      - REQ-001\n    implementations: {{}}\n"
        ),
        StarterTemplate::RustOnly => format!(
            "category: Rust Features\nversion: 1\n\nfeatures:\n  - id: FEAT-RUST-001\n    title: Bootstrap the {project_name} Rust spec workspace\n    summary: Provide a Rust-oriented starter structure that contributors can extend.\n    status: planned\n    linked_requirements:\n      - REQ-RUST-001\n    implementations: {{}}\n"
        ),
        StarterTemplate::PythonOnly => format!(
            "category: Python Features\nversion: 1\n\nfeatures:\n  - id: FEAT-PY-001\n    title: Bootstrap the {project_name} Python spec workspace\n    summary: Provide a Python-oriented starter structure that contributors can extend.\n    status: planned\n    linked_requirements:\n      - REQ-PY-001\n    implementations: {{}}\n"
        ),
        StarterTemplate::Polyglot => format!(
            "category: Polyglot Features\nversion: 1\n\nfeatures:\n  - id: FEAT-MIX-001\n    title: Bootstrap the {project_name} polyglot spec workspace\n    summary: Provide a starter structure that can grow across Rust, Python, and TypeScript.\n    status: planned\n    linked_requirements:\n      - REQ-MIX-001\n    implementations: {{}}\n"
        ),
    }
}

fn requirement_document_path(template: StarterTemplate) -> &'static str {
    match template {
        StarterTemplate::Generic => "docs/syu/requirements/core/core.yaml",
        StarterTemplate::RustOnly => "docs/syu/requirements/core/rust.yaml",
        StarterTemplate::PythonOnly => "docs/syu/requirements/core/python.yaml",
        StarterTemplate::Polyglot => "docs/syu/requirements/core/polyglot.yaml",
    }
}

fn feature_document_path(template: StarterTemplate) -> &'static str {
    match template {
        StarterTemplate::Generic => "docs/syu/features/core/core.yaml",
        StarterTemplate::RustOnly => "docs/syu/features/languages/rust.yaml",
        StarterTemplate::PythonOnly => "docs/syu/features/languages/python.yaml",
        StarterTemplate::Polyglot => "docs/syu/features/languages/polyglot.yaml",
    }
}

fn feature_kind(template: StarterTemplate) -> &'static str {
    match template {
        StarterTemplate::Generic => "core",
        StarterTemplate::RustOnly => "rust",
        StarterTemplate::PythonOnly => "python",
        StarterTemplate::Polyglot => "polyglot",
    }
}

fn path_label(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::cli::{InitArgs, StarterTemplate};

    use super::{
        GENERATED_PATHS, ensure_writable_targets, feature_document_path, feature_kind,
        infer_project_name, requirement_document_path, run_init_command, scaffold_files,
    };

    #[test]
    fn infer_project_name_uses_workspace_directory() {
        assert_eq!(
            infer_project_name(std::path::Path::new("/tmp/example-project")),
            "example-project"
        );
    }

    #[test]
    fn scaffold_files_include_all_expected_templates() {
        let files = scaffold_files("demo", StarterTemplate::Generic);
        let paths: Vec<_> = files.into_iter().map(|(path, _)| path).collect();
        assert_eq!(paths.len(), GENERATED_PATHS.len());
        for expected in GENERATED_PATHS {
            assert!(paths.iter().any(|path| path == expected));
        }
    }

    #[test]
    fn ensure_writable_targets_reports_existing_files_without_force() {
        let tempdir = tempdir().expect("tempdir should exist");
        fs::write(tempdir.path().join("syu.yaml"), "existing").expect("file should exist");

        let error = ensure_writable_targets(
            tempdir.path(),
            [std::path::PathBuf::from("syu.yaml")].into_iter(),
            false,
        )
        .expect_err("overwrite should be rejected");

        assert!(error.to_string().contains("refusing to overwrite"));
    }

    #[test]
    fn init_command_creates_scaffolded_workspace() {
        let tempdir = tempdir().expect("tempdir should exist");
        let workspace = tempdir.path().join("demo");
        let args = InitArgs {
            workspace: workspace.clone(),
            name: Some("demo".to_string()),
            template: StarterTemplate::Generic,
            force: false,
            format: crate::cli::OutputFormat::Text,
        };

        let code = run_init_command(&args).expect("init should succeed");
        assert_eq!(code, 0);

        for path in GENERATED_PATHS {
            assert!(
                workspace.join(path).exists(),
                "missing generated file: {path}"
            );
        }
    }

    #[test]
    fn init_command_honors_force_overwrite() {
        let tempdir = tempdir().expect("tempdir should exist");
        fs::create_dir_all(tempdir.path()).expect("workspace dir");
        fs::write(tempdir.path().join("syu.yaml"), "old").expect("old config");

        let args = InitArgs {
            workspace: tempdir.path().to_path_buf(),
            name: Some("forced".to_string()),
            template: StarterTemplate::Generic,
            force: true,
            format: crate::cli::OutputFormat::Text,
        };

        run_init_command(&args).expect("force init should succeed");
        let config =
            fs::read_to_string(tempdir.path().join("syu.yaml")).expect("config should exist");
        assert!(config.contains("default_fix: false"));
        assert!(config.contains("allow_planned: true"));
        assert!(config.contains(crate::config::current_cli_version()));
    }

    #[test]
    fn scaffold_files_default_to_planned_status() {
        let files = scaffold_files("demo", StarterTemplate::Generic);
        let requirement = files
            .iter()
            .find(|(path, _)| path == "docs/syu/requirements/core/core.yaml")
            .expect("requirement template");
        let feature = files
            .iter()
            .find(|(path, _)| path == "docs/syu/features/core/core.yaml")
            .expect("feature template");

        assert!(requirement.1.contains("status: planned"));
        assert!(feature.1.contains("status: planned"));
    }

    #[test]
    fn init_command_rejects_file_workspaces() {
        let tempdir = tempdir().expect("tempdir should exist");
        let file_path = tempdir.path().join("workspace-file");
        fs::write(&file_path, "not a directory").expect("file should exist");

        let error = run_init_command(&InitArgs {
            workspace: file_path.clone(),
            name: None,
            template: StarterTemplate::Generic,
            force: false,
            format: crate::cli::OutputFormat::Text,
        })
        .expect_err("file workspace should be rejected");

        assert!(error.to_string().contains(&format!(
            "workspace path `{}` is not a directory",
            file_path.display()
        )));
    }

    #[test]
    fn init_command_reports_directory_creation_failures() {
        let tempdir = tempdir().expect("tempdir should exist");
        let workspace = tempdir.path().join("demo");
        fs::create_dir_all(&workspace).expect("workspace should exist");
        fs::write(workspace.join("docs"), "blocking file").expect("blocking file should exist");

        let error = run_init_command(&InitArgs {
            workspace,
            name: Some("demo".to_string()),
            template: StarterTemplate::Generic,
            force: false,
            format: crate::cli::OutputFormat::Text,
        })
        .expect_err("directory creation should fail");

        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn init_command_reports_file_write_failures() {
        let tempdir = tempdir().expect("tempdir should exist");
        let workspace = tempdir.path().join("demo");
        fs::create_dir_all(workspace.join("docs/syu/features/core/core.yaml"))
            .expect("blocking directory should exist");

        let error = run_init_command(&InitArgs {
            workspace,
            name: Some("demo".to_string()),
            template: StarterTemplate::Generic,
            force: true,
            format: crate::cli::OutputFormat::Text,
        })
        .expect_err("file write should fail");

        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn init_command_json_format_includes_created_files() {
        let tempdir = tempdir().expect("tempdir should exist");
        let workspace = tempdir.path().join("demo");
        let args = InitArgs {
            workspace: workspace.clone(),
            name: Some("demo".to_string()),
            template: StarterTemplate::Generic,
            force: false,
            format: crate::cli::OutputFormat::Json,
        };

        let code = run_init_command(&args).expect("init should succeed");
        assert_eq!(code, 0);

        for path in GENERATED_PATHS {
            assert!(
                workspace.join(path).exists(),
                "missing generated file: {path}"
            );
        }
    }

    #[test]
    fn scaffold_files_support_language_oriented_templates() {
        for template in [
            StarterTemplate::RustOnly,
            StarterTemplate::PythonOnly,
            StarterTemplate::Polyglot,
        ] {
            let files = scaffold_files("demo", template);
            let paths: Vec<_> = files.iter().map(|(path, _)| path.as_str()).collect();
            assert!(paths.contains(&"syu.yaml"));
            assert!(paths.contains(&"docs/syu/philosophy/foundation.yaml"));
            assert!(paths.contains(&"docs/syu/policies/policies.yaml"));
            assert!(paths.contains(&requirement_document_path(template)));
            assert!(paths.contains(&feature_document_path(template)));

            let registry = files
                .iter()
                .find(|(path, _)| path == "docs/syu/features/features.yaml")
                .expect("feature registry should exist");
            assert!(registry.1.contains(feature_kind(template)));
        }
    }
}
