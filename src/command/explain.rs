// FEAT-EXPLAIN-001

use std::fmt::Write as _;

use anyhow::Result;
use serde::Serialize;

use crate::{
    cli::{ExplainArgs, OutputFormat},
    workspace::load_workspace,
};

use super::relate::{
    DirectMatches, Gap, JsonRelateOutput, RelatedNode, RelatedTrace, SelectionSummary,
    build_relation_report,
};

#[derive(Debug, Clone, Serialize)]
struct ExplainOutput {
    selection: SelectionSummary,
    assessment: ExplainAssessment,
    direct_matches: DirectMatches,
    chain: ExplainChain,
    traces: Vec<RelatedTrace>,
    gaps: Vec<Gap>,
}

#[derive(Debug, Clone, Serialize)]
struct ExplainChain {
    philosophies: Vec<RelatedNode>,
    policies: Vec<RelatedNode>,
    requirements: Vec<RelatedNode>,
    features: Vec<RelatedNode>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ExplainAssessment {
    Aligned,
    NeedsAttention,
}

impl ExplainAssessment {
    const fn label(self) -> &'static str {
        match self {
            Self::Aligned => "aligned",
            Self::NeedsAttention => "needs-attention",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Self::Aligned => {
                "The connected philosophy, policy, requirement, and feature chain is present with no obvious graph gaps."
            }
            Self::NeedsAttention => {
                "The connected chain is present, but at least one obvious gap or mismatch still needs review."
            }
        }
    }
}

pub fn run_explain_command(args: &ExplainArgs) -> Result<i32> {
    let workspace = load_workspace(&args.workspace)?;
    let relation = build_relation_report(&workspace, &args.selector)?;
    let output = build_explain_output(relation);

    match args.format {
        OutputFormat::Text => print!("{}", render_explain_text(&output)),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .expect("serializing explain output to JSON should succeed")
        ),
    }

    Ok(0)
}

fn build_explain_output(relation: JsonRelateOutput) -> ExplainOutput {
    let assessment = if relation.gaps.is_empty() {
        ExplainAssessment::Aligned
    } else {
        ExplainAssessment::NeedsAttention
    };

    ExplainOutput {
        selection: relation.selection,
        assessment,
        direct_matches: relation.direct_matches,
        chain: ExplainChain {
            philosophies: relation.philosophies,
            policies: relation.policies,
            requirements: relation.requirements,
            features: relation.features,
        },
        traces: relation.traces,
        gaps: relation.gaps,
    }
}

fn render_explain_text(output: &ExplainOutput) -> String {
    let mut rendered = String::new();
    writeln!(
        rendered,
        "Selection: {} {}",
        output.selection.kind, output.selection.query
    )
    .expect("write to string");
    writeln!(rendered, "Assessment: {}", output.assessment.label()).expect("write to string");
    writeln!(rendered, "{}", output.assessment.summary()).expect("write to string");
    writeln!(rendered).expect("write to string");

    writeln!(rendered, "Connected chain:").expect("write to string");
    render_nodes(&mut rendered, "Philosophies", &output.chain.philosophies);
    render_nodes(&mut rendered, "Policies", &output.chain.policies);
    render_nodes(&mut rendered, "Requirements", &output.chain.requirements);
    render_nodes(&mut rendered, "Features", &output.chain.features);
    writeln!(rendered).expect("write to string");

    writeln!(rendered, "Direct matches:").expect("write to string");
    if output.direct_matches.definitions.is_empty() && output.direct_matches.traces.is_empty() {
        writeln!(rendered, "- none").expect("write to string");
    } else {
        for node in &output.direct_matches.definitions {
            writeln!(rendered, "- {} {} — {}", node.kind, node.id, node.title)
                .expect("write to string");
        }
        for trace in &output.direct_matches.traces {
            let symbols = if trace.symbols.is_empty() {
                "file-only".to_string()
            } else {
                trace.symbols.join(", ")
            };
            writeln!(
                rendered,
                "- {} {} {} {}\t{} ({symbols})",
                trace.owner_kind, trace.owner_id, trace.relation_kind, trace.language, trace.file
            )
            .expect("write to string");
        }
    }
    writeln!(rendered).expect("write to string");

    writeln!(rendered, "Traces in scope:").expect("write to string");
    if output.traces.is_empty() {
        writeln!(rendered, "- none").expect("write to string");
    } else {
        for trace in &output.traces {
            let symbols = if trace.symbols.is_empty() {
                "file-only".to_string()
            } else {
                trace.symbols.join(", ")
            };
            let direct = if trace.direct_match {
                " (direct match)"
            } else {
                ""
            };
            writeln!(
                rendered,
                "- {} {} {} {}\t{} ({symbols}){direct}",
                trace.owner_kind, trace.owner_id, trace.relation_kind, trace.language, trace.file
            )
            .expect("write to string");
        }
    }
    writeln!(rendered).expect("write to string");

    writeln!(rendered, "Obvious gaps:").expect("write to string");
    if output.gaps.is_empty() {
        writeln!(rendered, "- none").expect("write to string");
    } else {
        for gap in &output.gaps {
            writeln!(rendered, "- {} {} — {}", gap.kind, gap.id, gap.message)
                .expect("write to string");
        }
    }

    rendered
}

fn render_nodes(rendered: &mut String, label: &str, nodes: &[RelatedNode]) {
    writeln!(rendered, "{label}:").expect("write to string");
    if nodes.is_empty() {
        writeln!(rendered, "- none").expect("write to string");
        return;
    }

    for node in nodes {
        writeln!(
            rendered,
            "- {} {} — {} ({})",
            node.kind, node.id, node.title, node.document_path
        )
        .expect("write to string");
    }
}

#[cfg(test)]
mod tests {
    use super::ExplainAssessment;

    #[test]
    fn assessment_labels_stay_stable() {
        assert_eq!(ExplainAssessment::Aligned.label(), "aligned");
        assert_eq!(ExplainAssessment::NeedsAttention.label(), "needs-attention");
    }
}
