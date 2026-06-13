//! Console output, reusing the ORM CLI's ANSI [`Style`] and symbols so every Tork
//! command looks the same.

use tork_orm_cli::{sym, Style};

/// A bold section header on its own line.
pub fn header(style: &Style, text: &str) {
    println!("\n  {}", style.bold(text));
}

/// A dim, secondary note line.
pub fn note(style: &Style, text: &str) {
    println!("  {}", style.dim(text));
}

/// A created-file line: a green check and the dim path.
pub fn created(style: &Style, path: &str) {
    println!("  {} {}", style.green(sym::CHECK), style.dim(path));
}

/// A "next step" line: a cyan arrow and the instruction.
pub fn step(style: &Style, text: &str) {
    println!("  {} {}", style.cyan(sym::ARROW), text);
}

/// Announces a command about to run.
pub fn running(style: &Style, command: &str) {
    println!("\n  {} {}", style.dim("running"), style.cyan(command));
}

/// A success summary line.
pub fn success(style: &Style, text: &str) {
    println!("\n  {} {}\n", style.green(sym::CHECK), style.bold(text));
}

/// An error line to stderr.
pub fn error(style: &Style, message: &str) {
    eprintln!("\n  {} {}\n", style.red(sym::CROSS), message);
}
