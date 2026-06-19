//! The Tork developer CLI.
//!
//! `tork new` scaffolds a project, `tork migrate ...` drives the ORM migration
//! engine, and `tork build`/`check`/`format`/`dev` wrap the cargo dev loop, all
//! behind one colored interface.
#![forbid(unsafe_code)]

mod cli;
mod commands;
mod templates;
mod ui;

use std::process::ExitCode;

use clap::Parser;
use tork_orm_cli::Style;

use cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let style = Style::detect(cli.global.no_color);

    let result = match &cli.command {
        Command::New(args) => commands::new::run(args, &style),
        // Generate must build the project to read its models, so it drives cargo
        // rather than the ORM CLI's async DB-only path.
        Command::Migrate(cli::MigrateCommand::Generate { name }) => {
            commands::generate::run(name, &cli.global, &style)
        }
        Command::Migrate(command) => run_migrate(command, &cli.global, &style),
        Command::Build { args } => commands::build::run(args, &style),
        Command::Check { clippy } => commands::check::run(*clippy, &style),
        Command::Format { fix } => commands::format::run(*fix, &style),
        Command::Dev { bin } => commands::dev::run(bin.as_deref(), &style),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            ui::error(&style, &message);
            ExitCode::FAILURE
        }
    }
}

/// Runs a `migrate` subcommand on a small async runtime (the only async path).
fn run_migrate(
    command: &cli::MigrateCommand,
    global: &cli::GlobalArgs,
    style: &Style,
) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to start the async runtime: {e}"))?;
    runtime
        .block_on(tork_orm_cli::run_migrate(command, global, style))
        .map_err(|e| e.to_string())
}
