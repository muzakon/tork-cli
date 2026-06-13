//! A thin, transparent wrapper around `cargo` with inherited stdio.

use std::process::Command;

use tork_orm_cli::Style;

use crate::ui;

/// Runs `cargo <args> <extra>` inheriting stdio. A non-zero exit becomes an error.
pub fn run(style: &Style, args: &[&str], extra: &[String]) -> Result<(), String> {
    let shown: Vec<&str> = args
        .iter()
        .copied()
        .chain(extra.iter().map(String::as_str))
        .collect();
    ui::running(style, &format!("cargo {}", shown.join(" ")));

    let status = Command::new("cargo")
        .args(args)
        .args(extra)
        .status()
        .map_err(|e| format!("could not run cargo ({e}); is it installed and on PATH?"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("`cargo {}` failed", shown.join(" ")))
    }
}
