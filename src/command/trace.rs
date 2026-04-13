// FEAT-TRACE-001
// REQ-CORE-021

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::{
    cli::{LookupKind, OutputFormat, TraceArgs},
    coverage::normalize_relative_path,
    model::{Feature, Requirement, TraceReference},
    workspace::{Workspace, load_workspace},
};

use super::lookup::{EntitySummary, WorkspaceLookup};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum TraceLookupStatus {
    Owned,
    Partial,
    Unowned,
}

impl TraceLookupStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::Owned => "owned",
            Self::Partial => "partial",
            Self::Unowned => "unowned",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchMode {
    File,
    Symbol,
    Wildcard,
}

impl MatchMode {
    const fn label(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Symbol => "symbol",
            Self::Wildcard => "wildcard",
        }
    }

    fn from_label(label: &str) -> Self {
        match label {
            "symbol" => Self::Symbol,
            "wildcard" => Self::Wildcard,
            _ => Self::File,
        }
    }

    fn matched_label(self, symbol: Option<&str>) -> String {
        match self {
            Self::File => "file".to_string(),
            Self::Symbol => format!("symbol `{}`", symbol.expect("symbol match should exist")),
            Self::Wildcard => "wildcard `*`".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct TraceOwnerMatch {
    kind: &'static str,
    id: String,
    title: String,
    trace_role: String,
    language: String,
    file: String,
    declared_symbols: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matched_symbol: Option<String>,
    match_mode: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct TraceLookupOutput {
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
    status: TraceLookupStatus,
    matched_owners: Vec<TraceOwnerMatch>,
    file_only_owners: Vec<TraceOwnerMatch>,
    requirements: Vec<EntitySummary>,
    features: Vec<EntitySummary>,
    policies: Vec<EntitySummary>,
    philosophies: Vec<EntitySummary>,
}

#[derive(Debug, Clone, Copy)]
struct TraceOwnerMetadata<'a> {
    kind: LookupKind,
    id: &'a str,
    title: &'a str,
    trace_role: &'a str,
}

pub fn run_trace_command(args: &TraceArgs) -> Result<i32> {
    let symbol = match args.symbol.as_deref() {
        Some(symbol) if symbol.trim().is_empty() => {
            bail!("trace symbol must not be empty or whitespace");
        }
        Some(symbol) => Some(symbol.trim()),
        None => None,
    };

    let workspace = load_workspace(&args.workspace)?;
    let file = normalize_lookup_file(&workspace, &args.file)?;
    let output = lookup_trace(&workspace, &file, symbol);

    match args.format {
        OutputFormat::Text => print_text_output(&output),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .expect("serializing trace lookup output to JSON should succeed")
        ),
    }

    Ok(0)
}

fn normalize_lookup_file(workspace: &Workspace, file: &Path) -> Result<PathBuf> {
    let relative = if file.is_absolute() {
        file.strip_prefix(&workspace.root)
            .with_context(|| {
                format!(
                    "trace file `{}` must stay under workspace `{}`",
                    file.display(),
                    workspace.root.display()
                )
            })?
            .to_path_buf()
    } else {
        file.to_path_buf()
    };
    let normalized = normalize_relative_path(&relative);
    if normalized.as_os_str().is_empty() {
        bail!("trace file must not be empty");
    }
    Ok(normalized)
}

fn lookup_trace(workspace: &Workspace, file: &Path, symbol: Option<&str>) -> TraceLookupOutput {
    let lookup = WorkspaceLookup::new(workspace);
    let file_label = file.display().to_string();
    let mut matched_owners = Vec::new();
    let mut file_only_owners = Vec::new();

    collect_requirement_matches(
        &workspace.requirements,
        &file_label,
        file,
        symbol,
        &mut matched_owners,
        &mut file_only_owners,
    );
    collect_feature_matches(
        &workspace.features,
        &file_label,
        file,
        symbol,
        &mut matched_owners,
        &mut file_only_owners,
    );

    matched_owners.sort();
    matched_owners.dedup();
    file_only_owners.sort();
    file_only_owners.dedup();

    let context_matches = if matched_owners.is_empty() {
        &file_only_owners
    } else {
        &matched_owners
    };
    let related = collect_related_entities(lookup, context_matches);

    TraceLookupOutput {
        file: file_label,
        symbol: symbol.map(ToString::to_string),
        status: lookup_status(&matched_owners, &file_only_owners),
        matched_owners,
        file_only_owners,
        requirements: related.requirements,
        features: related.features,
        policies: related.policies,
        philosophies: related.philosophies,
    }
}

fn collect_requirement_matches(
    requirements: &[Requirement],
    file_label: &str,
    file: &Path,
    symbol: Option<&str>,
    matched_owners: &mut Vec<TraceOwnerMatch>,
    file_only_owners: &mut Vec<TraceOwnerMatch>,
) {
    for requirement in requirements {
        collect_trace_matches(
            TraceOwnerMetadata {
                kind: LookupKind::Requirement,
                id: &requirement.id,
                title: &requirement.title,
                trace_role: "test",
            },
            &requirement.tests,
            file_label,
            file,
            symbol,
            matched_owners,
            file_only_owners,
        );
    }
}

fn collect_feature_matches(
    features: &[Feature],
    file_label: &str,
    file: &Path,
    symbol: Option<&str>,
    matched_owners: &mut Vec<TraceOwnerMatch>,
    file_only_owners: &mut Vec<TraceOwnerMatch>,
) {
    for feature in features {
        collect_trace_matches(
            TraceOwnerMetadata {
                kind: LookupKind::Feature,
                id: &feature.id,
                title: &feature.title,
                trace_role: "implementation",
            },
            &feature.implementations,
            file_label,
            file,
            symbol,
            matched_owners,
            file_only_owners,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_trace_matches(
    owner: TraceOwnerMetadata<'_>,
    references: &BTreeMap<String, Vec<TraceReference>>,
    file_label: &str,
    file: &Path,
    symbol: Option<&str>,
    matched_owners: &mut Vec<TraceOwnerMatch>,
    file_only_owners: &mut Vec<TraceOwnerMatch>,
) {
    for (language, items) in references {
        for reference in items {
            if normalize_relative_path(&reference.file) != file {
                continue;
            }

            match match_mode(reference, symbol) {
                Some(mode) => matched_owners.push(trace_owner_match(
                    owner, language, file_label, reference, mode, symbol,
                )),
                None if symbol.is_some() => file_only_owners.push(trace_owner_match(
                    owner,
                    language,
                    file_label,
                    reference,
                    MatchMode::File,
                    None,
                )),
                None => {}
            }
        }
    }
}

fn trace_owner_match(
    owner: TraceOwnerMetadata<'_>,
    language: &str,
    file_label: &str,
    reference: &TraceReference,
    match_mode: MatchMode,
    symbol: Option<&str>,
) -> TraceOwnerMatch {
    let matched_symbol = match match_mode {
        MatchMode::Symbol => symbol.map(ToString::to_string),
        MatchMode::Wildcard => Some("*".to_string()),
        MatchMode::File => None,
    };

    TraceOwnerMatch {
        kind: owner.kind.label(),
        id: owner.id.to_string(),
        title: owner.title.to_string(),
        trace_role: owner.trace_role.to_string(),
        language: language.to_string(),
        file: file_label.to_string(),
        declared_symbols: reference.symbols.clone(),
        matched_symbol,
        match_mode: match_mode.label(),
    }
}

fn match_mode(reference: &TraceReference, symbol: Option<&str>) -> Option<MatchMode> {
    let Some(symbol) = symbol else {
        return Some(MatchMode::File);
    };

    if reference.symbols.iter().any(|candidate| candidate == "*") {
        return Some(MatchMode::Wildcard);
    }

    if reference
        .symbols
        .iter()
        .any(|candidate| candidate == symbol)
    {
        return Some(MatchMode::Symbol);
    }

    None
}

fn lookup_status(
    matched_owners: &[TraceOwnerMatch],
    file_only_owners: &[TraceOwnerMatch],
) -> TraceLookupStatus {
    if !matched_owners.is_empty() {
        TraceLookupStatus::Owned
    } else if !file_only_owners.is_empty() {
        TraceLookupStatus::Partial
    } else {
        TraceLookupStatus::Unowned
    }
}

struct RelatedEntities {
    requirements: Vec<EntitySummary>,
    features: Vec<EntitySummary>,
    policies: Vec<EntitySummary>,
    philosophies: Vec<EntitySummary>,
}

fn collect_related_entities(
    lookup: WorkspaceLookup<'_>,
    matches: &[TraceOwnerMatch],
) -> RelatedEntities {
    let mut requirements = BTreeMap::new();
    let mut features = BTreeMap::new();
    let mut policies = BTreeMap::new();
    let mut philosophies = BTreeMap::new();

    for owner in matches {
        match owner.kind {
            "requirement" => {
                insert_summary(
                    &mut requirements,
                    lookup,
                    LookupKind::Requirement,
                    &owner.id,
                );
                if let Some(requirement) = lookup.requirement(&owner.id) {
                    for feature_id in &requirement.linked_features {
                        insert_summary(&mut features, lookup, LookupKind::Feature, feature_id);
                    }
                    collect_requirement_context(
                        lookup,
                        requirement,
                        &mut policies,
                        &mut philosophies,
                    );
                }
            }
            "feature" => {
                insert_summary(&mut features, lookup, LookupKind::Feature, &owner.id);
                if let Some(feature) = lookup.feature(&owner.id) {
                    for requirement_id in &feature.linked_requirements {
                        insert_summary(
                            &mut requirements,
                            lookup,
                            LookupKind::Requirement,
                            requirement_id,
                        );
                        if let Some(requirement) = lookup.requirement(requirement_id) {
                            collect_requirement_context(
                                lookup,
                                requirement,
                                &mut policies,
                                &mut philosophies,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    RelatedEntities {
        requirements: requirements.into_values().collect(),
        features: features.into_values().collect(),
        policies: policies.into_values().collect(),
        philosophies: philosophies.into_values().collect(),
    }
}

fn collect_requirement_context(
    lookup: WorkspaceLookup<'_>,
    requirement: &Requirement,
    policies: &mut BTreeMap<String, EntitySummary>,
    philosophies: &mut BTreeMap<String, EntitySummary>,
) {
    for policy_id in &requirement.linked_policies {
        insert_summary(policies, lookup, LookupKind::Policy, policy_id);
        if let Some(policy) = lookup.policy(policy_id) {
            for philosophy_id in &policy.linked_philosophies {
                insert_summary(philosophies, lookup, LookupKind::Philosophy, philosophy_id);
            }
        }
    }
}

fn insert_summary(
    map: &mut BTreeMap<String, EntitySummary>,
    lookup: WorkspaceLookup<'_>,
    kind: LookupKind,
    id: &str,
) {
    let Some(title) = lookup.title_for(kind, id) else {
        return;
    };
    map.entry(id.to_string()).or_insert_with(|| EntitySummary {
        id: id.to_string(),
        title: title.to_string(),
        document_path: None,
    });
}

fn print_text_output(output: &TraceLookupOutput) {
    println!("File: {}", output.file);
    if let Some(symbol) = &output.symbol {
        println!("Symbol: {symbol}");
    }
    println!("Status: {}", output.status.label());

    match output.status {
        TraceLookupStatus::Owned => {
            println!("Matched trace owners:");
            for owner in &output.matched_owners {
                print_owner_match(owner);
            }
        }
        TraceLookupStatus::Partial => {
            let symbol = output
                .symbol
                .as_deref()
                .expect("partial trace lookups should include a symbol");
            println!("No trace owners matched symbol `{symbol}`.");
            println!("File owners without a matching symbol:");
            for owner in &output.file_only_owners {
                print_owner_match(owner);
            }
            println!(
                "Hint: Trace the symbol explicitly in the matching requirement or feature, or use `*` when the whole file belongs to one owner."
            );
        }
        TraceLookupStatus::Unowned => {
            println!(
                "No requirement or feature traces reference `{}`.",
                query_label(output)
            );
            println!(
                "Hint: Add the file to a requirement test trace or feature implementation trace, then rerun `syu validate . --genre trace`."
            );
            return;
        }
    }

    print_entity_section("Requirements", &output.requirements);
    print_entity_section("Features", &output.features);
    print_entity_section("Policies", &output.policies);
    print_entity_section("Philosophies", &output.philosophies);
}

fn print_owner_match(owner: &TraceOwnerMatch) {
    let matched_by =
        MatchMode::from_label(owner.match_mode).matched_label(owner.matched_symbol.as_deref());
    println!(
        "- {} {}\t{} ({}, {}, matched by {})",
        owner.kind, owner.id, owner.title, owner.language, owner.trace_role, matched_by
    );
    if !owner.declared_symbols.is_empty() {
        println!("  declared symbols: {}", owner.declared_symbols.join(", "));
    }
}

fn print_entity_section(heading: &str, items: &[EntitySummary]) {
    println!("{heading}:");
    if items.is_empty() {
        println!("- none");
        return;
    }

    for item in items {
        println!("- {}\t{}", item.id, item.title);
    }
}

fn query_label(output: &TraceLookupOutput) -> String {
    match &output.symbol {
        Some(symbol) => format!("{}::{symbol}", output.file),
        None => output.file.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    use crate::{config::SyuConfig, workspace::Workspace};

    use super::{
        MatchMode, TraceLookupOutput, TraceLookupStatus, normalize_lookup_file, query_label,
    };

    #[test]
    fn normalize_lookup_file_accepts_workspace_relative_paths() {
        let tempdir = tempdir().expect("tempdir should exist");
        let workspace = Workspace {
            root: tempdir.path().to_path_buf(),
            spec_root: tempdir.path().join("docs/syu"),
            config: SyuConfig::default(),
            philosophies: Vec::new(),
            policies: Vec::new(),
            requirements: Vec::new(),
            features: Vec::new(),
        };

        let normalized = normalize_lookup_file(&workspace, Path::new("./src/../src/lib.rs"))
            .expect("relative file should normalize");
        assert_eq!(normalized, PathBuf::from("src/lib.rs"));
    }

    #[test]
    fn normalize_lookup_file_rejects_absolute_paths_outside_the_workspace() {
        let tempdir = tempdir().expect("tempdir should exist");
        let workspace = Workspace {
            root: tempdir.path().to_path_buf(),
            spec_root: tempdir.path().join("docs/syu"),
            config: SyuConfig::default(),
            philosophies: Vec::new(),
            policies: Vec::new(),
            requirements: Vec::new(),
            features: Vec::new(),
        };

        let error = normalize_lookup_file(&workspace, Path::new("/tmp/outside.rs"))
            .expect_err("outside paths should fail");
        assert!(error.to_string().contains("must stay under workspace"));
    }

    #[test]
    fn query_label_includes_symbols_when_present() {
        let label = query_label(&TraceLookupOutput {
            file: "src/lib.rs".to_string(),
            symbol: Some("run".to_string()),
            status: TraceLookupStatus::Unowned,
            matched_owners: Vec::new(),
            file_only_owners: Vec::new(),
            requirements: Vec::new(),
            features: Vec::new(),
            policies: Vec::new(),
            philosophies: Vec::new(),
        });
        assert_eq!(label, "src/lib.rs::run");
    }

    #[test]
    fn match_mode_labels_round_trip() {
        assert_eq!(MatchMode::from_label("file"), MatchMode::File);
        assert_eq!(MatchMode::from_label("symbol"), MatchMode::Symbol);
        assert_eq!(MatchMode::from_label("wildcard"), MatchMode::Wildcard);
    }
}
