// FEAT-LSP-001
// REQ-CORE-001

use anyhow::{Result, bail};
use regex::Regex;
use serde_json::Value;
use std::path::Path;
use std::sync::LazyLock;

use crate::workspace::{Workspace, load_workspace};

use super::protocol::{
    Hover, InitializeParams, InitializeResult, MarkupContent, ServerCapabilities,
    TextDocumentPositionParams,
};

pub(crate) struct LspHandlers {
    workspace: Option<Workspace>,
    initialized: bool,
}

impl LspHandlers {
    pub(crate) fn new() -> Self {
        Self {
            workspace: None,
            initialized: false,
        }
    }

    pub(crate) fn handle_initialize(&mut self, params: InitializeParams) -> Result<Value> {
        let root_path = if let Some(root_uri) = params.root_uri {
            uri_to_path(&root_uri)?
        } else {
            std::env::current_dir()?
        };

        self.workspace = Some(load_workspace(&root_path)?);

        let result = InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(true),
            },
        };

        Ok(serde_json::to_value(result)?)
    }

    pub(crate) fn handle_initialized(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    pub(crate) fn handle_shutdown(&mut self) -> Result<Value> {
        self.initialized = false;
        Ok(Value::Null)
    }

    pub(crate) fn handle_hover(&self, params: TextDocumentPositionParams) -> Result<Option<Hover>> {
        let workspace = match &self.workspace {
            Some(ws) => ws,
            None => bail!("workspace not initialized"),
        };

        let file_path = uri_to_path(&params.text_document.uri)?;
        let line = params.position.line as usize;

        let content = std::fs::read_to_string(&file_path)?;
        let lines: Vec<&str> = content.lines().collect();

        if line >= lines.len() {
            return Ok(None);
        }

        let current_line = lines[line];
        let char_pos = params.position.character as usize;

        if let Some(spec_id) = find_spec_id_at_position(current_line, char_pos)
            && let Some(hover_content) = create_hover_for_spec_id(workspace, &spec_id)
        {
            return Ok(Some(hover_content));
        }

        Ok(None)
    }
}

fn uri_to_path(uri: &str) -> Result<std::path::PathBuf> {
    if let Some(path_str) = uri.strip_prefix("file://") {
        Ok(Path::new(path_str).to_path_buf())
    } else {
        Ok(Path::new(uri).to_path_buf())
    }
}

fn find_spec_id_at_position(line: &str, char_pos: usize) -> Option<String> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\b(PHIL-[A-Z0-9-]+|POL-[A-Z0-9-]+|REQ-[A-Z0-9-]+|FEAT-[A-Z0-9-]+)\b").unwrap()
    });

    for cap in RE.captures_iter(line) {
        let matched = cap.get(0)?;
        let start = matched.start();
        let end = matched.end();

        if char_pos >= start && char_pos <= end {
            return Some(matched.as_str().to_string());
        }
    }

    None
}

fn create_hover_for_spec_id(workspace: &Workspace, spec_id: &str) -> Option<Hover> {
    if spec_id.starts_with("PHIL-") {
        workspace
            .philosophies
            .iter()
            .find(|p| p.id == spec_id)
            .map(|phil| {
                let content = format!(
                    "# {}\n\n**{}**\n\n## Product Design Principle\n{}\n\n## Coding Guideline\n{}",
                    phil.id, phil.title, phil.product_design_principle, phil.coding_guideline
                );
                Hover {
                    contents: MarkupContent::markdown(content),
                    range: None,
                }
            })
    } else if spec_id.starts_with("POL-") {
        workspace
            .policies
            .iter()
            .find(|p| p.id == spec_id)
            .map(|pol| {
                let content = format!(
                    "# {}\n\n**{}**\n\n## Summary\n{}\n\n## Description\n{}",
                    pol.id, pol.title, pol.summary, pol.description
                );
                Hover {
                    contents: MarkupContent::markdown(content),
                    range: None,
                }
            })
    } else if spec_id.starts_with("REQ-") {
        workspace
            .requirements
            .iter()
            .find(|r| r.id == spec_id)
            .map(|req| {
                let content = format!(
                    "# {}\n\n**{}**\n\n{}\n\n**Priority:** {} | **Status:** {}",
                    req.id, req.title, req.description, req.priority, req.status
                );
                Hover {
                    contents: MarkupContent::markdown(content),
                    range: None,
                }
            })
    } else if spec_id.starts_with("FEAT-") {
        workspace
            .features
            .iter()
            .find(|f| f.id == spec_id)
            .map(|feat| {
                let content = format!(
                    "# {}\n\n**{}**\n\n{}\n\n**Status:** {}",
                    feat.id, feat.title, feat.summary, feat.status
                );
                Hover {
                    contents: MarkupContent::markdown(content),
                    range: None,
                }
            })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_spec_id_at_position() {
        let line = "// FEAT-AUTH-001 implements authentication";
        assert_eq!(
            find_spec_id_at_position(line, 5),
            Some("FEAT-AUTH-001".to_string())
        );
        assert_eq!(find_spec_id_at_position(line, 0), None);
    }

    #[test]
    fn test_uri_to_path() {
        let uri = "file:///home/user/file.txt";
        let path = uri_to_path(uri).unwrap();
        assert_eq!(path.to_str().unwrap(), "/home/user/file.txt");
    }
}
