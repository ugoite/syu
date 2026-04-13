// FEAT-LOG-001
// REQ-CORE-021

use std::{
    collections::BTreeSet,
    fmt::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::{
    cli::{HistoryKind, LogArgs, OutputFormat},
    model::{Feature, Requirement, TraceReference},
    workspace::load_workspace,
};

use super::{
    lookup::{WorkspaceEntity, WorkspaceLookup},
    shell_quote_path,
};

const GIT_RECORD_SEPARATOR: u8 = 0x1e;

#[derive(Debug, Serialize)]
struct JsonLogOutput {
    id: String,
    entity_kind: &'static str,
    title: String,
    repository_root: String,
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    path_filter: Option<String>,
    tracked_paths: Vec<TrackedPath>,
    commits: Vec<MatchedCommit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TrackedPath {
    kind: &'static str,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct MatchedCommit {
    sha: String,
    short_sha: String,
    summary: String,
    author: String,
    authored_at: String,
    reasons: Vec<TrackedPath>,
}

#[derive(Debug, Clone)]
struct HistoryTarget {
    id: String,
    entity_kind: &'static str,
    title: String,
    tracked_paths: Vec<TrackedPath>,
}

pub fn run_log_command(args: &LogArgs) -> Result<i32> {
    if args.limit == 0 {
        bail!("`--limit` must be greater than zero");
    }

    let workspace = load_workspace(&args.workspace)?;
    let lookup = WorkspaceLookup::new(&workspace);
    let path_filter = args
        .path
        .as_deref()
        .map(|path| normalize_path_filter(&workspace.root, path))
        .transpose()?;
    let target = build_history_target(
        lookup,
        &workspace.root,
        &args.id,
        args.kind,
        path_filter.as_deref(),
    )?;
    let repository_root = resolve_git_repository_root(&workspace.root)?;
    let commits = load_git_history(&workspace.root, args.limit, &target.tracked_paths)?;

    match args.format {
        OutputFormat::Text => print!(
            "{}",
            render_history_text(
                &target,
                &repository_root,
                args.kind,
                path_filter.as_deref(),
                &commits,
            )
        ),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&JsonLogOutput {
                id: target.id.clone(),
                entity_kind: target.entity_kind,
                title: target.title.clone(),
                repository_root: repository_root.display().to_string(),
                kind: args.kind.label(),
                path_filter: path_filter.map(|path| path.display().to_string()),
                tracked_paths: target.tracked_paths.clone(),
                commits,
            })
            .expect("serializing log output to JSON should succeed")
        ),
    }

    Ok(0)
}

fn build_history_target(
    lookup: WorkspaceLookup<'_>,
    workspace_root: &Path,
    id: &str,
    kind: HistoryKind,
    path_filter: Option<&Path>,
) -> Result<HistoryTarget> {
    let Some(entity) = lookup.find(id) else {
        let workspace_arg = shell_quote_path(workspace_root);
        bail!(
            "definition `{id}` was not found in `{}`\n\nhint: Run `syu list {workspace_arg}` to see all available IDs, or `syu search {id} {workspace_arg}` to find related items.",
            workspace_root.display()
        );
    };

    let definition_path = lookup
        .document_path_for_id(id)?
        .with_context(|| format!("checked-in definition path for `{id}` was not found"))?;

    let (entity_kind, title, traces) = match entity {
        WorkspaceEntity::Requirement(item) => (
            "requirement",
            item.title.clone(),
            tracked_paths_for_requirement(item, kind, &definition_path)?,
        ),
        WorkspaceEntity::Feature(item) => (
            "feature",
            item.title.clone(),
            tracked_paths_for_feature(item, kind, &definition_path)?,
        ),
        WorkspaceEntity::Philosophy(_) | WorkspaceEntity::Policy(_) => {
            bail!(
                "`syu log` currently supports requirement and feature IDs only; `{id}` belongs to a non-trace layer."
            );
        }
    };

    let mut tracked_paths = traces;
    if let Some(path_filter) = path_filter {
        tracked_paths.retain(|tracked| Path::new(&tracked.path).starts_with(path_filter));
    }

    if tracked_paths.is_empty() {
        let mut filters = vec![format!("kind `{}`", kind.label())];
        if let Some(path_filter) = path_filter {
            filters.push(format!("path `{}`", path_filter.display()));
        }
        bail!(
            "no tracked history paths remain for `{id}` after applying {}",
            filters.join(" and ")
        );
    }

    Ok(HistoryTarget {
        id: id.to_string(),
        entity_kind,
        title,
        tracked_paths,
    })
}

fn tracked_paths_for_requirement(
    item: &Requirement,
    kind: HistoryKind,
    definition_path: &str,
) -> Result<Vec<TrackedPath>> {
    match kind {
        HistoryKind::Implementation => bail!(
            "`{}` is a requirement, so `--kind implementation` is not available. Use `--kind test`, `--kind definition`, or omit the flag.",
            item.id
        ),
        HistoryKind::All => {
            let mut tracked = vec![TrackedPath::definition(definition_path)];
            tracked.extend(collect_trace_paths("test", &item.tests));
            Ok(tracked)
        }
        HistoryKind::Definition => Ok(vec![TrackedPath::definition(definition_path)]),
        HistoryKind::Test => Ok(collect_trace_paths("test", &item.tests)),
    }
}

fn tracked_paths_for_feature(
    item: &Feature,
    kind: HistoryKind,
    definition_path: &str,
) -> Result<Vec<TrackedPath>> {
    match kind {
        HistoryKind::Test => bail!(
            "`{}` is a feature, so `--kind test` is not available. Use `--kind implementation`, `--kind definition`, or omit the flag.",
            item.id
        ),
        HistoryKind::All => {
            let mut tracked = vec![TrackedPath::definition(definition_path)];
            tracked.extend(collect_trace_paths("implementation", &item.implementations));
            Ok(tracked)
        }
        HistoryKind::Definition => Ok(vec![TrackedPath::definition(definition_path)]),
        HistoryKind::Implementation => {
            Ok(collect_trace_paths("implementation", &item.implementations))
        }
    }
}

fn collect_trace_paths(
    kind: &'static str,
    traces: &std::collections::BTreeMap<String, Vec<TraceReference>>,
) -> Vec<TrackedPath> {
    let mut tracked = Vec::new();
    for (language, references) in traces {
        for reference in references {
            tracked.push(TrackedPath {
                kind,
                path: reference.file.display().to_string(),
                language: Some(language.clone()),
                symbols: reference.symbols.clone(),
            });
        }
    }
    tracked
}

fn normalize_path_filter(workspace_root: &Path, path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return path
            .strip_prefix(workspace_root)
            .map(Path::to_path_buf)
            .with_context(|| {
                format!(
                    "path filter `{}` must stay inside workspace `{}`",
                    path.display(),
                    workspace_root.display()
                )
            });
    }

    Ok(path.to_path_buf())
}

fn resolve_git_repository_root(workspace_root: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .with_context(|| {
            format!(
                "failed to run `git rev-parse` in `{}`",
                workspace_root.display()
            )
        })?;

    if !output.status.success() {
        bail!(
            "workspace `{}` is not inside a Git repository, so `syu log` cannot inspect commit history.",
            workspace_root.display()
        );
    }

    let repository_root = String::from_utf8(output.stdout)
        .context("git repository root should be valid UTF-8")?
        .trim()
        .to_string();
    Ok(PathBuf::from(repository_root))
}

fn load_git_history(
    workspace_root: &Path,
    limit: usize,
    tracked_paths: &[TrackedPath],
) -> Result<Vec<MatchedCommit>> {
    let unique_paths = tracked_paths
        .iter()
        .map(|tracked| tracked.path.as_str())
        .collect::<BTreeSet<_>>();
    if unique_paths.is_empty() {
        return Ok(Vec::new());
    }

    let mut command = Command::new("git");
    command.arg("-C").arg(workspace_root).args([
        "log",
        "--relative",
        "--no-renames",
        "--max-count",
        &limit.to_string(),
        "--format=%x1E%H%x00%h%x00%an%x00%aI%x00%s",
        "--name-only",
        "-z",
        "--",
    ]);
    for path in unique_paths {
        command.arg(path);
    }

    let output = command
        .output()
        .with_context(|| format!("failed to run `git log` in `{}`", workspace_root.display()))?;
    if !output.status.success() {
        bail!(
            "failed to read git history for `{}`: {}",
            workspace_root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    parse_git_history(&output.stdout, tracked_paths)
}

fn parse_git_history(raw: &[u8], tracked_paths: &[TrackedPath]) -> Result<Vec<MatchedCommit>> {
    let mut commits = Vec::new();

    for record in raw.split(|byte| *byte == GIT_RECORD_SEPARATOR) {
        if record.is_empty() {
            continue;
        }

        let fields = record.split(|byte| *byte == 0).collect::<Vec<_>>();
        if fields.len() < 5 {
            bail!("unexpected `git log` output while parsing commit history");
        }

        let changed_files = fields[5..]
            .iter()
            .filter_map(|field| {
                let Ok(value) = std::str::from_utf8(field) else {
                    return None;
                };
                let trimmed = value.trim_start_matches('\n').trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .collect::<Vec<_>>();
        let reasons = matched_reasons(&changed_files, tracked_paths);
        if reasons.is_empty() {
            continue;
        }

        commits.push(MatchedCommit {
            sha: parse_git_field(fields[0])?,
            short_sha: parse_git_field(fields[1])?,
            author: parse_git_field(fields[2])?,
            authored_at: parse_git_field(fields[3])?,
            summary: parse_git_field(fields[4])?,
            reasons,
        });
    }

    Ok(commits)
}

fn parse_git_field(field: &[u8]) -> Result<String> {
    Ok(std::str::from_utf8(field)
        .context("git output should be valid UTF-8")?
        .to_string())
}

fn matched_reasons(changed_files: &[String], tracked_paths: &[TrackedPath]) -> Vec<TrackedPath> {
    let changed = changed_files
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    tracked_paths
        .iter()
        .filter(|tracked| changed.contains(tracked.path.as_str()))
        .cloned()
        .collect()
}

fn render_history_text(
    target: &HistoryTarget,
    repository_root: &Path,
    kind: HistoryKind,
    path_filter: Option<&Path>,
    commits: &[MatchedCommit],
) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "History: {} {} — {}",
        target.entity_kind, target.id, target.title
    )
    .expect("writing to String must succeed");
    writeln!(output, "Repository: {}", repository_root.display())
        .expect("writing to String must succeed");
    writeln!(output, "Selection: {}", kind.label()).expect("writing to String must succeed");
    if let Some(path_filter) = path_filter {
        writeln!(output, "Path filter: {}", path_filter.display())
            .expect("writing to String must succeed");
    }

    writeln!(output, "Tracked paths:").expect("writing to String must succeed");
    for tracked in &target.tracked_paths {
        writeln!(output, "- {}", render_tracked_path(tracked))
            .expect("writing to String must succeed");
    }

    if commits.is_empty() {
        writeln!(output, "Commits:\n- none").expect("writing to String must succeed");
        return output;
    }

    writeln!(output, "Commits:").expect("writing to String must succeed");
    for commit in commits {
        writeln!(
            output,
            "- {} {} {}",
            commit.short_sha, commit.authored_at, commit.summary
        )
        .expect("writing to String must succeed");
        for reason in &commit.reasons {
            writeln!(output, "  - {}", render_tracked_path(reason))
                .expect("writing to String must succeed");
        }
    }

    output
}

fn render_tracked_path(tracked: &TrackedPath) -> String {
    let mut output = format!("{}\t{}", tracked.kind, tracked.path);
    if let Some(language) = &tracked.language {
        write!(output, "\t{language}").expect("writing to String must succeed");
    }
    if !tracked.symbols.is_empty() {
        write!(
            output,
            "\t[{}]",
            tracked
                .symbols
                .iter()
                .map(|symbol| format!("`{symbol}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .expect("writing to String must succeed");
    }
    output
}

impl TrackedPath {
    fn definition(path: &str) -> Self {
        Self {
            kind: "definition",
            path: path.to_string(),
            language: None,
            symbols: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use crate::model::TraceReference;

    use super::{HistoryKind, collect_trace_paths, matched_reasons, normalize_path_filter};

    #[test]
    fn normalize_path_filter_accepts_relative_paths() {
        let normalized = normalize_path_filter(Path::new("/repo"), Path::new("src/command"))
            .expect("relative path should stay unchanged");
        assert_eq!(normalized, Path::new("src/command"));
    }

    #[test]
    fn collect_trace_paths_preserves_language_and_symbols() {
        let mut traces = BTreeMap::new();
        traces.insert(
            "rust".to_string(),
            vec![TraceReference {
                file: PathBuf::from("src/log.rs"),
                symbols: vec!["run_log_command".to_string()],
                doc_contains: Vec::new(),
            }],
        );

        let tracked = collect_trace_paths("implementation", &traces);
        assert_eq!(tracked[0].kind, "implementation");
        assert_eq!(tracked[0].path, "src/log.rs");
        assert_eq!(tracked[0].language.as_deref(), Some("rust"));
        assert_eq!(tracked[0].symbols, vec!["run_log_command"]);
    }

    #[test]
    fn matched_reasons_ignore_non_traced_files() {
        let tracked = vec![
            super::TrackedPath::definition("docs/syu/requirements/core/workspace.yaml"),
            super::TrackedPath {
                kind: "test",
                path: "tests/log_command.rs".to_string(),
                language: Some("rust".to_string()),
                symbols: vec!["history_command_supports_json".to_string()],
            },
        ];

        let reasons = matched_reasons(
            &["README.md".to_string(), "tests/log_command.rs".to_string()],
            &tracked,
        );

        assert_eq!(reasons.len(), 1);
        assert_eq!(reasons[0].kind, "test");
        assert_eq!(reasons[0].path, "tests/log_command.rs");
    }

    #[test]
    fn history_kind_labels_match_cli_values() {
        assert_eq!(HistoryKind::All.label(), "all");
        assert_eq!(HistoryKind::Definition.label(), "definition");
        assert_eq!(HistoryKind::Test.label(), "test");
        assert_eq!(HistoryKind::Implementation.label(), "implementation");
    }
}
