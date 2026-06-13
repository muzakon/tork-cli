//! The `tork` command surface (clap derive). The `migrate` subcommand reuses the
//! ORM CLI's command + global-flag types so behavior is identical.

use clap::builder::Styles;
use clap::{Args, Parser, Subcommand};

pub use tork_orm_cli::cli::{GlobalArgs, MigrateCommand};

/// The default git source for a new project's framework dependency (`tork`).
pub const DEFAULT_FRAMEWORK_GIT: &str = "https://github.com/tork-rs/tork-framework.git";

/// The default git source for a new project's ORM dependency (`tork-orm`).
pub const DEFAULT_ORM_GIT: &str = "https://github.com/tork-rs/tork-orm.git";

/// Colored clap help (green headers/usage, cyan literals).
fn help_styles() -> Styles {
    use clap::builder::styling::{AnsiColor, Style};
    Styles::styled()
        .header(Style::new().bold().fg_color(Some(AnsiColor::Green.into())))
        .usage(Style::new().bold().fg_color(Some(AnsiColor::Green.into())))
        .literal(Style::new().fg_color(Some(AnsiColor::Cyan.into())))
        .placeholder(Style::new().fg_color(Some(AnsiColor::Cyan.into())))
}

#[derive(Parser)]
#[command(
    name = "tork",
    about = "The Tork web framework CLI: scaffold, migrate, and run.",
    version,
    styles = help_styles(),
    propagate_version = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Migration flags (database URL, directory, table). Shared across commands.
    #[command(flatten)]
    pub global: GlobalArgs,
}

#[derive(Subcommand)]
pub enum Command {
    /// Scaffold a new Tork project.
    New(NewArgs),

    /// Database migrations: up, down, status, create, redo, init.
    #[command(subcommand)]
    Migrate(MigrateCommand),

    /// Compile the project (wraps `cargo build`).
    Build {
        /// Extra arguments forwarded to `cargo build`.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Type-check the project (wraps `cargo check`, or `cargo clippy` with `--clippy`).
    Check {
        /// Run `cargo clippy` instead of `cargo check`.
        #[arg(long)]
        clippy: bool,
    },

    /// Format the code. Verifies by default; `--fix` writes changes (`cargo fmt`).
    Format {
        /// Apply formatting in place instead of only checking.
        #[arg(long)]
        fix: bool,
    },

    /// Run the project, rebuilding and restarting on file changes.
    Dev {
        /// The binary target to run (defaults to the package's default binary).
        #[arg(long)]
        bin: Option<String>,
    },
}

/// `tork new` arguments.
#[derive(Args)]
pub struct NewArgs {
    /// The project name (also the directory created for it). Prompted when omitted.
    pub name: Option<String>,

    /// The database backend: `sqlite`, `postgres`, `mysql`, or `none`. Prompted
    /// when omitted in an interactive session.
    #[arg(long)]
    pub db: Option<String>,

    /// Scaffold into the current directory instead of creating a new one.
    #[arg(long)]
    pub here: bool,

    /// The git URL for the generated project's framework dependency (`tork`).
    #[arg(long, default_value = DEFAULT_FRAMEWORK_GIT)]
    pub framework_git: String,

    /// The git URL for the generated project's ORM dependency (`tork-orm`).
    #[arg(long, default_value = DEFAULT_ORM_GIT)]
    pub orm_git: String,

    /// Pin the git dependencies to a branch (default: each repo's default branch).
    #[arg(long)]
    pub branch: Option<String>,
}
