// FEAT-INIT-006

use std::io::{self, IsTerminal, Write};

use anyhow::{Context, Result, bail};

pub(crate) trait PromptIo {
    fn is_terminal(&self) -> bool;
    fn prompt_line(&mut self, label: &str, default: Option<&str>) -> Result<String>;
}

pub(crate) struct StdioPromptIo;

impl PromptIo for StdioPromptIo {
    fn is_terminal(&self) -> bool {
        io::stdin().is_terminal() && io::stdout().is_terminal()
    }

    fn prompt_line(&mut self, label: &str, default: Option<&str>) -> Result<String> {
        match default {
            Some(value) => print!("{label} [{value}]: "),
            None => print!("{label}: "),
        }
        io::stdout()
            .flush()
            .context("failed to flush interactive prompt")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("failed to read interactive input")?;
        Ok(input.trim().to_string())
    }
}

pub(crate) fn ensure_prompt_terminal(prompt_io: &impl PromptIo, message: &str) -> Result<()> {
    if prompt_io.is_terminal() {
        return Ok(());
    }

    bail!("{message}");
}

pub(crate) fn prompt_required(prompt_io: &mut impl PromptIo, label: &str) -> Result<String> {
    loop {
        let raw = prompt_io.prompt_line(label, None)?;
        if !raw.is_empty() {
            return Ok(raw);
        }
        eprintln!("{label} is required.");
    }
}

pub(crate) fn prompt_with_default(
    prompt_io: &mut impl PromptIo,
    label: &str,
    default: &str,
) -> Result<String> {
    let raw = prompt_io.prompt_line(label, Some(default))?;
    if raw.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(raw)
    }
}

pub(crate) fn prompt_optional(
    prompt_io: &mut impl PromptIo,
    label: &str,
    default: Option<&str>,
) -> Result<Option<String>> {
    let raw = prompt_io.prompt_line(label, default)?;
    if raw.is_empty() {
        Ok(None)
    } else {
        Ok(Some(raw))
    }
}

pub(crate) fn prompt_optional_with_default(
    prompt_io: &mut impl PromptIo,
    label: &str,
    default: Option<&str>,
) -> Result<Option<String>> {
    let raw = prompt_io.prompt_line(label, default)?;
    if raw.is_empty() {
        Ok(default
            .map(std::string::ToString::to_string)
            .filter(|value| !value.is_empty()))
    } else {
        Ok(Some(raw))
    }
}

pub(crate) fn prompt_bool(
    prompt_io: &mut impl PromptIo,
    label: &str,
    default: bool,
) -> Result<bool> {
    let default_value = if default { "yes" } else { "no" };
    loop {
        let raw = prompt_with_default(prompt_io, label, default_value)?;
        match raw.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" | "true" => return Ok(true),
            "n" | "no" | "false" => return Ok(false),
            _ => eprintln!("Please enter yes or no."),
        }
    }
}
