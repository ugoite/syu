pub mod cli;
pub mod coverage;
pub mod command;
pub mod config;
pub mod inspect;
pub mod language;
pub mod model;
pub mod report;
pub mod rules;
pub mod runtime;
pub mod workspace;

use anyhow::Result;
use clap::Parser;

pub fn run() -> Result<i32> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Validate(args) => command::check::run_check_command(&args),
        cli::Commands::Report(args) => command::report::run_report_command(&args),
        cli::Commands::Init(args) => command::init::run_init_command(&args),
    }
}
