//! Console output, reusing the ORM CLI's ANSI [`Style`] and symbols so every Tork
//! command looks the same.

use std::io::{self, Write};

use tork_orm_cli::{sym, Style};

/// Reports whether the session is interactive (stdin and stdout are a terminal).
pub fn is_interactive() -> bool {
    use std::io::IsTerminal;
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// Prompts for a line of text, returning the trimmed input or `default` when the
/// user just presses Enter (the default is shown in brackets when set).
pub fn prompt(style: &Style, question: &str, default: Option<&str>) -> io::Result<String> {
    let suffix = match default {
        Some(value) => format!(" {}", style.dim(&format!("[{value}]"))),
        None => String::new(),
    };
    loop {
        print!("  {} {question}{suffix} ", style.cyan(sym::ARROW));
        io::stdout().flush()?;
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            // EOF: fall back to the default if there is one.
            return Ok(default.unwrap_or_default().to_owned());
        }
        let answer = line.trim();
        if !answer.is_empty() {
            return Ok(answer.to_owned());
        }
        if let Some(value) = default {
            return Ok(value.to_owned());
        }
    }
}

/// Asks the user to pick one of `options`, returning the chosen index.
pub fn select(style: &Style, question: &str, options: &[&str], default: usize) -> io::Result<usize> {
    println!("  {} {question}", style.cyan(sym::ARROW));
    for (index, option) in options.iter().enumerate() {
        let marker = if index == default {
            style.green(sym::CHECK).to_string()
        } else {
            style.dim("-").to_string()
        };
        println!("    {marker} {}  {}", index + 1, option);
    }
    loop {
        print!("  {} ", style.dim(&format!("choose 1-{} [{}]", options.len(), default + 1)));
        io::stdout().flush()?;
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            return Ok(default);
        }
        let answer = line.trim();
        if answer.is_empty() {
            return Ok(default);
        }
        if let Ok(choice) = answer.parse::<usize>() {
            if (1..=options.len()).contains(&choice) {
                return Ok(choice - 1);
            }
        }
    }
}

/// Asks a yes/no question, returning the answer (the default applies on Enter).
pub fn confirm(style: &Style, question: &str, default: bool) -> io::Result<bool> {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        print!("  {} {question} {} ", style.cyan(sym::ARROW), style.dim(&format!("[{hint}]")));
        io::stdout().flush()?;
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            return Ok(default);
        }
        match line.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {}
        }
    }
}

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
