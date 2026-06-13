//! `tork format` — verify formatting (`cargo fmt -- --check`), or apply it with
//! `--fix` (`cargo fmt`).

use tork_orm_cli::Style;

pub fn run(fix: bool, style: &Style) -> Result<(), String> {
    if fix {
        super::cargo::run(style, &["fmt"], &[])
    } else {
        super::cargo::run(style, &["fmt", "--", "--check"], &[])
    }
}
