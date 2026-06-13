//! `tork dev` — run the project, restarting it when watched files change.
//!
//! Uses a self-contained `notify` watcher (no external `cargo watch` needed):
//! the first event after an idle period kills the running `cargo run` and starts
//! a fresh one. Ctrl-C in the terminal stops both (they share the process group).

use std::path::Path;
use std::process::{Child, Command};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use tork_orm_cli::Style;

use crate::ui;

/// The directories and files a change should trigger a restart for.
const WATCHED: [&str; 3] = ["src", "Cargo.toml", "migrations"];

pub fn run(bin: Option<&str>, style: &Style) -> Result<(), String> {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    })
    .map_err(|e| format!("failed to start the file watcher: {e}"))?;

    let mut watching = Vec::new();
    for path in WATCHED {
        if Path::new(path).exists() {
            watcher
                .watch(Path::new(path), RecursiveMode::Recursive)
                .map_err(|e| format!("failed to watch `{path}`: {e}"))?;
            watching.push(path);
        }
    }
    if watching.is_empty() {
        return Err("nothing to watch; run `tork dev` from a project root".into());
    }

    ui::header(style, "tork dev");
    ui::note(style, &format!("watching {} — press Ctrl-C to stop", watching.join(", ")));

    let mut child = spawn_run(bin, style)?;
    loop {
        // Block until something changes.
        if rx.recv().is_err() {
            break;
        }
        // Debounce: absorb the burst of events a single save produces.
        loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return stop(child),
            }
        }
        ui::step(style, "change detected, restarting");
        let _ = child.kill();
        let _ = child.wait();
        child = spawn_run(bin, style)?;
    }
    stop(child)
}

fn spawn_run(bin: Option<&str>, style: &Style) -> Result<Child, String> {
    let mut command = Command::new("cargo");
    command.arg("run");
    if let Some(bin) = bin {
        command.args(["--bin", bin]);
    }
    let label = match bin {
        Some(bin) => format!("cargo run --bin {bin}"),
        None => "cargo run".to_string(),
    };
    ui::running(style, &label);
    command
        .spawn()
        .map_err(|e| format!("could not run cargo ({e}); is it installed and on PATH?"))
}

fn stop(mut child: Child) -> Result<(), String> {
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}
