use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "syu",
    version,
    about = "Specification-driven development helper"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(visible_alias = "check")]
    Validate(ValidateArgs),
    Report(ReportArgs),
    Init(InitArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ValidateArgs {
    #[arg(default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    #[arg(long, action = ArgAction::SetTrue)]
    pub fix: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub no_fix: bool,
}

pub type CheckArgs = ValidateArgs;

#[derive(Debug, Clone, Args)]
pub struct ReportArgs {
    #[arg(default_value = ".")]
    pub workspace: PathBuf,

    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct InitArgs {
    #[arg(default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}
