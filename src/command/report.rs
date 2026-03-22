// REQ-CORE-004

use std::{fs, path::PathBuf};

use anyhow::Result;

use crate::{
    cli::ReportArgs, command::check::collect_check_result, config::load_config,
    report::render_markdown_report,
};

// FEAT-REPORT-001
pub fn run_report_command(args: &ReportArgs) -> Result<i32> {
    let result = collect_check_result(&args.workspace);
    let markdown = render_markdown_report(&result);
    let output = resolve_report_output(args)?;

    if let Some(output) = &output {
        if let Some(parent) = output.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(output, markdown)?;
        println!("wrote report to {}", output.display());
    } else {
        println!("{markdown}");
    }

    Ok(if result.is_success() { 0 } else { 1 })
}

fn resolve_report_output(args: &ReportArgs) -> Result<Option<PathBuf>> {
    if let Some(output) = &args.output {
        return Ok(Some(output.clone()));
    }

    let loaded = load_config(&args.workspace)?;
    Ok(loaded.config.report.output.map(|output| {
        if output.is_absolute() {
            output
        } else {
            args.workspace.join(output)
        }
    }))
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::{Path, PathBuf},
    };

    use tempfile::tempdir;

    use crate::cli::ReportArgs;

    use super::{resolve_report_output, run_report_command};

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/workspaces")
            .join(name)
    }

    fn write_config(root: &Path, output: Option<&str>) {
        let report_section = output
            .map(|output| format!("report:\n  output: {output}\n"))
            .unwrap_or_default();
        fs::write(
            root.join("syu.yaml"),
            format!(
                "version: {}\nspec:\n  root: docs/syu\nvalidate:\n  default_fix: false\n  allow_planned: true\n  require_non_orphaned_items: true\n  require_symbol_trace_coverage: false\n{report_section}runtimes:\n  python:\n    command: auto\n  node:\n    command: auto\n",
                env!("CARGO_PKG_VERSION"),
            ),
        )
        .expect("config");
    }

    #[test]
    fn report_command_returns_zero_for_passing_workspace() {
        let args = ReportArgs {
            workspace: fixture_path("passing"),
            output: None,
        };

        let code = run_report_command(&args).expect("report should succeed");
        assert_eq!(code, 0);
    }

    #[test]
    fn resolve_report_output_defaults_to_stdout_when_config_is_missing() {
        let tempdir = tempdir().expect("tempdir should exist");
        let output = resolve_report_output(&ReportArgs {
            workspace: tempdir.path().to_path_buf(),
            output: None,
        })
        .expect("default output should resolve");
        assert_eq!(output, None);
    }

    #[test]
    fn resolve_report_output_uses_workspace_relative_config_paths() {
        let tempdir = tempdir().expect("tempdir should exist");
        write_config(tempdir.path(), Some("docs/generated/syu-report.md"));

        let output = resolve_report_output(&ReportArgs {
            workspace: tempdir.path().to_path_buf(),
            output: None,
        })
        .expect("configured output should resolve");

        assert_eq!(
            output,
            Some(tempdir.path().join("docs/generated/syu-report.md"))
        );
    }

    #[test]
    fn resolve_report_output_preserves_absolute_config_paths() {
        let tempdir = tempdir().expect("tempdir should exist");
        let absolute = tempdir.path().join("report.md");
        write_config(tempdir.path(), Some(absolute.to_string_lossy().as_ref()));

        let output = resolve_report_output(&ReportArgs {
            workspace: tempdir.path().to_path_buf(),
            output: None,
        })
        .expect("absolute output should resolve");

        assert_eq!(output, Some(absolute));
    }

    #[test]
    fn resolve_report_output_prefers_cli_paths_over_config() {
        let tempdir = tempdir().expect("tempdir should exist");
        write_config(tempdir.path(), Some("docs/generated/syu-report.md"));

        let output = resolve_report_output(&ReportArgs {
            workspace: tempdir.path().to_path_buf(),
            output: Some(PathBuf::from("custom/report.md")),
        })
        .expect("cli output should win");

        assert_eq!(output, Some(PathBuf::from("custom/report.md")));
    }

    #[test]
    fn report_command_returns_one_for_failing_workspace() {
        let args = ReportArgs {
            workspace: fixture_path("failing"),
            output: None,
        };

        let code = run_report_command(&args).expect("report should still render");
        assert_eq!(code, 1);
    }

    #[test]
    fn report_command_errors_when_output_parent_is_a_file() {
        let tempdir = tempdir().expect("tempdir should exist");
        let occupied = tempdir.path().join("occupied");
        fs::write(&occupied, "not a directory").expect("occupied file should exist");

        let args = ReportArgs {
            workspace: fixture_path("passing"),
            output: Some(occupied.join("report.md")),
        };

        let error = run_report_command(&args).expect_err("writing through file parent should fail");
        assert!(
            error.to_string().contains("Not a directory")
                || error.to_string().contains("File exists")
        );
    }

    #[test]
    fn report_command_writes_to_relative_paths_without_parent_directories() {
        let tempdir = tempdir().expect("tempdir should exist");
        let previous = env::current_dir().expect("cwd should be readable");
        env::set_current_dir(tempdir.path()).expect("should chdir into tempdir");

        let args = ReportArgs {
            workspace: fixture_path("passing"),
            output: Some(std::path::PathBuf::from("report.md")),
        };

        let result = run_report_command(&args).expect("relative report path should work");
        let report = fs::read_to_string(tempdir.path().join("report.md"))
            .expect("relative report should be written");
        env::set_current_dir(previous).expect("should restore cwd");

        assert_eq!(result, 0);
        assert!(report.contains("# syu validation report"));
    }

    #[cfg(unix)]
    #[test]
    fn report_command_handles_root_paths_without_parent() {
        let args = ReportArgs {
            workspace: fixture_path("passing"),
            output: Some(std::path::PathBuf::from("/")),
        };

        let error = run_report_command(&args).expect_err("writing to root directory should fail");
        assert!(!error.to_string().is_empty());
    }
}
