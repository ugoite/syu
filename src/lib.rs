pub mod cli;
pub mod command;
pub mod config;
pub mod coverage;
pub mod inspect;
pub mod language;
pub mod model;
pub mod report;
pub mod rules;
pub mod runtime;
pub mod workspace;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use std::io::IsTerminal;

pub fn run() -> Result<i32> {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(cli::Commands::Browse(args)) => command::browse::run_browse_command(&args),
        None if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() => {
            command::browse::run_browse_command(&cli::BrowseArgs::default())
        }
        None => {
            let mut command = cli::Cli::command();
            command.print_help()?;
            println!();
            Ok(0)
        }
        Some(cli::Commands::Validate(args)) => command::check::run_check_command(&args),
        Some(cli::Commands::Report(args)) => command::report::run_report_command(&args),
        Some(cli::Commands::Init(args)) => command::init::run_init_command(&args),
    }
}
