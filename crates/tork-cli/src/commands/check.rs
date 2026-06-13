//! `tork check` — type-check the project (`cargo check`, or `cargo clippy`).

use tork_orm_cli::Style;

pub fn run(clippy: bool, style: &Style) -> Result<(), String> {
    let command = if clippy { "clippy" } else { "check" };
    super::cargo::run(style, &[command], &[])
}
