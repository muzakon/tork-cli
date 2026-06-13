//! `tork build` — compile the project (wraps `cargo build`).

use tork_orm_cli::Style;

pub fn run(args: &[String], style: &Style) -> Result<(), String> {
    super::cargo::run(style, &["build"], args)
}
