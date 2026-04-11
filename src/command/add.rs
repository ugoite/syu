// FEAT-ADD-001
// REQ-CORE-020

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::{
    cli::{AddArgs, LookupKind},
    coverage::normalize_relative_path,
    model::{
        FeatureDocument, FeatureRegistryDocument, PhilosophyDocument, PolicyDocument,
        RequirementDocument,
    },
    workspace::{Workspace, load_workspace},
};

use super::{lookup::WorkspaceLookup, shell_quote_path};

pub fn run_add_command(args: &AddArgs) -> Result<i32> {
    let workspace = load_workspace(&args.workspace)?;
    let parsed_id = ParsedId::parse(args.layer, &args.id)?;

    if args.layer != LookupKind::Feature && args.kind.is_some() {
        bail!("`--kind` is only supported when scaffolding features");
    }
    if WorkspaceLookup::new(&workspace)
        .find(&parsed_id.normalized)
        .is_some()
    {
        bail!(
            "{} `{}` already exists in `{}`",
            args.layer.label(),
            parsed_id.normalized,
            workspace.root.display()
        );
    }

    let feature_kind = match args.layer {
        LookupKind::Feature => Some(normalize_feature_kind(
            args.kind
                .as_deref()
                .unwrap_or(parsed_id.folder_slug.as_str()),
        )?),
        _ => None,
    };
    let target = resolve_target_path(
        &workspace,
        args.layer,
        args.file.as_deref(),
        &parsed_id,
        feature_kind.as_deref(),
    )?;
    let created_target_file = !target.absolute.exists();

    write_stub_document(args.layer, &parsed_id, feature_kind.as_deref(), &target)?;

    let registry_updated = if args.layer == LookupKind::Feature {
        update_feature_registry(
            &workspace,
            &target.absolute,
            feature_kind
                .as_deref()
                .expect("features require a registry kind"),
        )?
    } else {
        false
    };

    print_add_summary(
        &workspace,
        args.layer,
        &parsed_id.normalized,
        &target.absolute,
        created_target_file,
        registry_updated,
    );
    Ok(0)
}

#[derive(Debug, Clone)]
struct ParsedId {
    normalized: String,
    typed_prefix: String,
    title: String,
    folder_slug: String,
    file_slug: String,
}

impl ParsedId {
    fn parse(layer: LookupKind, raw: &str) -> Result<Self> {
        let normalized = normalize_definition_id(raw, layer)?;
        let segments: Vec<_> = normalized.split('-').collect();
        let suffix = &segments[1..segments.len() - 1];
        let title = if suffix.is_empty() {
            default_title(layer)
        } else {
            title_case_tokens(suffix)
        };
        let folder_slug = suffix
            .first()
            .map(|segment| segment.to_ascii_lowercase())
            .unwrap_or_else(|| default_folder_slug(layer));
        let file_slug = if suffix.len() > 1 {
            suffix[1..]
                .iter()
                .map(|segment| segment.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .join("-")
        } else {
            folder_slug.clone()
        };

        Ok(Self {
            typed_prefix: segments[..segments.len() - 1].join("-"),
            normalized,
            title,
            folder_slug,
            file_slug,
        })
    }
}

#[derive(Debug, Clone)]
struct TargetPath {
    absolute: PathBuf,
}

fn normalize_definition_id(raw: &str, layer: LookupKind) -> Result<String> {
    let normalized = raw.trim().to_ascii_uppercase();
    if normalized.is_empty()
        || normalized.split('-').any(|segment| segment.is_empty())
        || !normalized
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!(
            "{} IDs must contain only ASCII letters, numbers, and single hyphens: `{}`",
            layer.label(),
            raw.trim()
        );
    }

    let segments: Vec<_> = normalized.split('-').collect();
    let expected_prefix = match layer {
        LookupKind::Philosophy => "PHIL",
        LookupKind::Policy => "POL",
        LookupKind::Requirement => "REQ",
        LookupKind::Feature => "FEAT",
    };
    if segments.first().copied() != Some(expected_prefix) {
        bail!(
            "{} IDs must start with `{expected_prefix}-`: `{}`",
            layer.label(),
            raw.trim()
        );
    }
    if segments.len() < 2
        || !segments
            .last()
            .expect("segments should exist")
            .chars()
            .all(|ch| ch.is_ascii_digit())
    {
        bail!(
            "{} IDs must end with a numeric segment like `{expected_prefix}-001`: `{}`",
            layer.label(),
            raw.trim()
        );
    }

    Ok(normalized)
}

fn normalize_feature_kind(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed != trimmed.to_ascii_lowercase()
        || trimmed.split('-').any(|segment| segment.is_empty())
        || !trimmed
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!(
            "feature `--kind` must contain only lowercase ASCII letters, numbers, and single hyphens: `{}`",
            raw.trim()
        );
    }
    Ok(trimmed.to_string())
}

fn resolve_target_path(
    workspace: &Workspace,
    layer: LookupKind,
    explicit_file: Option<&Path>,
    parsed_id: &ParsedId,
    feature_kind: Option<&str>,
) -> Result<TargetPath> {
    let default_relative = default_document_path(layer, parsed_id, feature_kind);
    let absolute = if let Some(file) = explicit_file {
        resolve_explicit_file(workspace, file)?
    } else {
        workspace.spec_root.join(default_relative)
    };

    ensure_target_within_spec_root(workspace, layer, &absolute)?;
    Ok(TargetPath { absolute })
}

fn default_document_path(
    layer: LookupKind,
    parsed_id: &ParsedId,
    feature_kind: Option<&str>,
) -> PathBuf {
    match layer {
        LookupKind::Philosophy => PathBuf::from("philosophy/foundation.yaml"),
        LookupKind::Policy => PathBuf::from("policies/policies.yaml"),
        LookupKind::Requirement => PathBuf::from(format!(
            "requirements/{}/{}.yaml",
            parsed_id.folder_slug, parsed_id.file_slug
        )),
        LookupKind::Feature => {
            let kind = feature_kind.expect("feature default paths require a kind");
            PathBuf::from(format!("features/{kind}/{}.yaml", parsed_id.file_slug))
        }
    }
}

fn resolve_explicit_file(workspace: &Workspace, raw: &Path) -> Result<PathBuf> {
    if raw.is_absolute() {
        bail!(
            "`--file` must stay inside `{}` as a relative path",
            workspace.spec_root.display()
        );
    }

    let normalized = normalize_relative_path(raw);
    if normalized.as_os_str().is_empty()
        || normalized.is_absolute()
        || normalized
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!(
            "`--file` must stay within the workspace or configured spec root: `{}`",
            raw.display()
        );
    }

    let workspace_relative = workspace.root.join(&normalized);
    if workspace_relative.starts_with(&workspace.spec_root) {
        return Ok(workspace_relative);
    }

    let spec_relative = workspace.spec_root.join(&normalized);
    if spec_relative.starts_with(&workspace.spec_root) {
        return Ok(spec_relative);
    }

    bail!(
        "`--file` must point inside `{}` or be relative to that spec root: `{}`",
        workspace.spec_root.display(),
        raw.display()
    );
}

fn ensure_target_within_spec_root(
    workspace: &Workspace,
    layer: LookupKind,
    path: &Path,
) -> Result<()> {
    if !path.starts_with(&workspace.spec_root) {
        bail!(
            "target path `{}` must stay within `{}`",
            path.display(),
            workspace.spec_root.display()
        );
    }
    if !matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("yaml" | "yml")
    ) {
        bail!(
            "target path must point to a YAML document: `{}`",
            path.display()
        );
    }

    let layer_root = match layer {
        LookupKind::Philosophy => workspace.spec_root.join("philosophy"),
        LookupKind::Policy => workspace.spec_root.join("policies"),
        LookupKind::Requirement => workspace.spec_root.join("requirements"),
        LookupKind::Feature => workspace.spec_root.join("features"),
    };
    if !path.starts_with(&layer_root) {
        bail!(
            "{} stubs must stay under `{}`",
            layer.label(),
            layer_root.display()
        );
    }
    if layer == LookupKind::Feature && path == workspace.spec_root.join("features/features.yaml") {
        bail!("feature stubs must use a feature document path, not `features/features.yaml`");
    }

    Ok(())
}

fn write_stub_document(
    layer: LookupKind,
    parsed_id: &ParsedId,
    feature_kind: Option<&str>,
    target: &TargetPath,
) -> Result<()> {
    if let Some(parent) = target.absolute.parent() {
        fs::create_dir_all(parent)?;
    }

    if target.absolute.exists() {
        validate_existing_document(layer, &target.absolute, parsed_id)?;
        append_yaml_list_item(&target.absolute, render_item_block(layer, parsed_id))?;
    } else {
        let document = render_new_document(layer, parsed_id, feature_kind);
        fs::write(&target.absolute, document)?;
    }

    Ok(())
}

fn validate_existing_document(layer: LookupKind, path: &Path, parsed_id: &ParsedId) -> Result<()> {
    let raw = fs::read_to_string(path).with_context(|| {
        format!(
            "failed to read {} document `{}`",
            layer.label(),
            path.display()
        )
    })?;

    match layer {
        LookupKind::Philosophy => {
            let _: PhilosophyDocument = serde_yaml::from_str(&raw).with_context(|| {
                format!(
                    "failed to parse philosophy document from `{}`",
                    path.display()
                )
            })?;
        }
        LookupKind::Policy => {
            let _: PolicyDocument = serde_yaml::from_str(&raw).with_context(|| {
                format!("failed to parse policy document from `{}`", path.display())
            })?;
        }
        LookupKind::Requirement => {
            let document: RequirementDocument = serde_yaml::from_str(&raw).with_context(|| {
                format!(
                    "failed to parse requirement document from `{}`",
                    path.display()
                )
            })?;
            if document.prefix.trim() != parsed_id.typed_prefix {
                bail!(
                    "requirement document `{}` uses prefix `{}`, which does not match `{}`",
                    path.display(),
                    document.prefix,
                    parsed_id.typed_prefix
                );
            }
        }
        LookupKind::Feature => {
            let _: FeatureDocument = serde_yaml::from_str(&raw).with_context(|| {
                format!("failed to parse feature document from `{}`", path.display())
            })?;
        }
    }

    Ok(())
}

fn append_yaml_list_item(path: &Path, item_block: String) -> Result<()> {
    let mut existing = fs::read_to_string(path)
        .with_context(|| format!("failed to read YAML document `{}`", path.display()))?;
    while existing.ends_with('\n') {
        existing.pop();
    }
    existing.push_str("\n\n");
    existing.push_str(&item_block);
    existing.push('\n');
    fs::write(path, existing)
        .with_context(|| format!("failed to update YAML document `{}`", path.display()))
}

fn render_new_document(
    layer: LookupKind,
    parsed_id: &ParsedId,
    feature_kind: Option<&str>,
) -> String {
    match layer {
        LookupKind::Philosophy => format!(
            "category: Philosophy\nversion: 1\nlanguage: en\n\nphilosophies:\n{}\n",
            render_item_block(layer, parsed_id)
        ),
        LookupKind::Policy => format!(
            "category: Policies\nversion: 1\nlanguage: en\n\npolicies:\n{}\n",
            render_item_block(layer, parsed_id)
        ),
        LookupKind::Requirement => format!(
            "category: {} Requirements\nprefix: {}\n\nrequirements:\n{}\n",
            title_case_slug(&parsed_id.folder_slug),
            parsed_id.typed_prefix,
            render_item_block(layer, parsed_id)
        ),
        LookupKind::Feature => format!(
            "category: {} Features\nversion: 1\n\nfeatures:\n{}\n",
            title_case_slug(feature_kind.unwrap_or(parsed_id.folder_slug.as_str())),
            render_item_block(layer, parsed_id)
        ),
    }
}

fn render_item_block(layer: LookupKind, parsed_id: &ParsedId) -> String {
    match layer {
        LookupKind::Philosophy => format!(
            "  - id: {}\n    title: {}\n    product_design_principle: |\n      Describe the durable product design principle for {}.\n    coding_guideline: |\n      Describe the coding guideline that keeps {} actionable in day-to-day work.\n    linked_policies: []",
            parsed_id.normalized, parsed_id.title, parsed_id.normalized, parsed_id.normalized
        ),
        LookupKind::Policy => format!(
            "  - id: {}\n    title: {}\n    summary: Document the rule enforced by {}.\n    description: |\n      Describe how this policy turns philosophy into a concrete contributor rule.\n    linked_philosophies: []\n    linked_requirements: []",
            parsed_id.normalized, parsed_id.title, parsed_id.normalized
        ),
        LookupKind::Requirement => format!(
            "  - id: {}\n    title: {}\n    description: |\n      Describe the concrete requirement that {} adds to the repository.\n    priority: high\n    status: planned\n    linked_policies: []\n    linked_features: []\n    tests: {{}}",
            parsed_id.normalized, parsed_id.title, parsed_id.normalized
        ),
        LookupKind::Feature => format!(
            "  - id: {}\n    title: {}\n    summary: Describe the shipped capability that {} represents.\n    status: planned\n    linked_requirements: []\n    implementations: {{}}",
            parsed_id.normalized, parsed_id.title, parsed_id.normalized
        ),
    }
}

fn update_feature_registry(
    workspace: &Workspace,
    feature_document: &Path,
    kind: &str,
) -> Result<bool> {
    let registry_path = workspace.spec_root.join("features/features.yaml");
    let raw = fs::read_to_string(&registry_path).with_context(|| {
        format!(
            "failed to read feature registry `{}`",
            registry_path.display()
        )
    })?;
    let registry: FeatureRegistryDocument = serde_yaml::from_str(&raw).with_context(|| {
        format!(
            "failed to parse feature registry from `{}`",
            registry_path.display()
        )
    })?;
    let relative = feature_document
        .strip_prefix(workspace.spec_root.join("features"))
        .with_context(|| {
            format!(
                "feature document `{}` must stay under `{}`",
                feature_document.display(),
                workspace.spec_root.join("features").display()
            )
        })?;
    let portable = path_label(relative);

    if let Some(existing) = registry
        .files
        .iter()
        .find(|entry| path_label(&entry.file) == portable)
    {
        if existing.kind != kind {
            bail!(
                "feature registry already tracks `{portable}` under kind `{}`",
                existing.kind
            );
        }
        return Ok(false);
    }

    let mut updated = raw;
    while updated.ends_with('\n') {
        updated.pop();
    }
    updated.push_str(&format!("\n  - kind: {kind}\n    file: {portable}\n"));
    fs::write(&registry_path, updated).with_context(|| {
        format!(
            "failed to update feature registry `{}`",
            registry_path.display()
        )
    })?;
    Ok(true)
}

fn print_add_summary(
    workspace: &Workspace,
    layer: LookupKind,
    id: &str,
    target: &Path,
    created_target_file: bool,
    registry_updated: bool,
) {
    let action = if created_target_file {
        "created"
    } else {
        "updated"
    };
    println!(
        "{action} {} stub `{id}` in {}",
        layer.label(),
        display_workspace_path(workspace, target)
    );
    if registry_updated {
        println!(
            "updated {}",
            display_workspace_path(
                workspace,
                &workspace.spec_root.join("features/features.yaml")
            )
        );
    }
    println!();
    println!("Next steps:");
    println!("  1. Edit the generated stub fields and reciprocal links");
    println!(
        "  2. Run `syu validate {}` once the new item is linked",
        shell_quote_path(&workspace.root)
    );
}

fn display_workspace_path(workspace: &Workspace, path: &Path) -> String {
    path.strip_prefix(&workspace.root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn path_label(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn default_title(layer: LookupKind) -> String {
    match layer {
        LookupKind::Philosophy => "New philosophy".to_string(),
        LookupKind::Policy => "New policy".to_string(),
        LookupKind::Requirement => "New requirement".to_string(),
        LookupKind::Feature => "New feature".to_string(),
    }
}

fn default_folder_slug(layer: LookupKind) -> String {
    match layer {
        LookupKind::Philosophy => "philosophy".to_string(),
        LookupKind::Policy => "policies".to_string(),
        LookupKind::Requirement | LookupKind::Feature => "core".to_string(),
    }
}

fn title_case_tokens(tokens: &[&str]) -> String {
    tokens
        .iter()
        .map(|segment| title_case_slug(&segment.to_ascii_lowercase()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use crate::{cli::LookupKind, config::SyuConfig, workspace::Workspace};

    use super::{
        ParsedId, default_document_path, normalize_definition_id, normalize_feature_kind,
        resolve_explicit_file, title_case_slug,
    };

    #[test]
    fn normalize_definition_id_enforces_expected_prefixes() {
        assert_eq!(
            normalize_definition_id("req-auth-001", LookupKind::Requirement)
                .expect("requirement ids should normalize"),
            "REQ-AUTH-001"
        );
        assert!(normalize_definition_id("FEAT-AUTH-001", LookupKind::Requirement).is_err());
        assert!(normalize_definition_id("REQ-AUTH", LookupKind::Requirement).is_err());
    }

    #[test]
    fn parsed_ids_infer_title_folder_and_file_slugs() {
        let parsed = ParsedId::parse(LookupKind::Feature, "FEAT-AUTH-LOGIN-001")
            .expect("feature ids should parse");
        assert_eq!(parsed.typed_prefix, "FEAT-AUTH-LOGIN");
        assert_eq!(parsed.title, "Auth Login");
        assert_eq!(parsed.folder_slug, "auth");
        assert_eq!(parsed.file_slug, "login");
    }

    #[test]
    fn default_document_path_uses_kind_folder_for_features() {
        let parsed =
            ParsedId::parse(LookupKind::Feature, "FEAT-AUTH-LOGIN-001").expect("feature id");
        assert_eq!(
            default_document_path(LookupKind::Feature, &parsed, Some("auth")),
            PathBuf::from("features/auth/login.yaml")
        );
    }

    #[test]
    fn resolve_explicit_file_accepts_workspace_and_spec_relative_paths() {
        let tempdir = tempdir().expect("tempdir should exist");
        let workspace = Workspace {
            root: tempdir.path().join("workspace"),
            spec_root: tempdir.path().join("workspace/docs/spec"),
            config: SyuConfig::default(),
            philosophies: Vec::new(),
            policies: Vec::new(),
            requirements: Vec::new(),
            features: Vec::new(),
        };

        assert_eq!(
            resolve_explicit_file(
                &workspace,
                std::path::Path::new("docs/spec/features/auth/login.yaml")
            )
            .expect("workspace-relative paths should work"),
            workspace.spec_root.join("features/auth/login.yaml")
        );
        assert_eq!(
            resolve_explicit_file(&workspace, std::path::Path::new("features/auth/login.yaml"))
                .expect("spec-relative paths should work"),
            workspace.spec_root.join("features/auth/login.yaml")
        );
    }

    #[test]
    fn normalize_feature_kind_rejects_invalid_values() {
        assert_eq!(
            normalize_feature_kind("auth-login").expect("valid kind"),
            "auth-login"
        );
        assert!(normalize_feature_kind("Auth").is_err());
        assert!(normalize_feature_kind("auth login").is_err());
    }

    #[test]
    fn title_case_slug_expands_hyphenated_names() {
        assert_eq!(title_case_slug("auth-login"), "Auth Login");
    }
}
