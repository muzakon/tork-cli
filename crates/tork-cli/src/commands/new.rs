//! `tork new` — scaffold a uniform Tork project (no `mod.rs`; 2018-style sibling
//! module files), asking for the project name and database interactively.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tork_orm_cli::Style;

use crate::cli::NewArgs;
use crate::templates::{self, Context, Database};
use crate::ui;

pub fn run(args: &NewArgs, style: &Style) -> Result<(), String> {
    let interactive = ui::is_interactive();

    let name = resolve_name(args, style, interactive)?;
    let db = resolve_database(args, style, interactive)?;

    let root = if args.here {
        PathBuf::from(".")
    } else {
        PathBuf::from(&name)
    };
    if !args.here {
        if root.exists() {
            return Err(format!("`{name}` already exists"));
        }
        fs::create_dir(&root).map_err(|e| format!("cannot create `{}`: {e}", root.display()))?;
    }

    let ctx = Context {
        name: name.clone(),
        tork_dep: templates::git_dep(&args.framework_git, args.branch.as_deref()),
        orm_dep: templates::orm_dep(&args.orm_git, args.branch.as_deref(), db),
        db,
    };

    ui::header(style, &format!("Creating {name}"));
    for file in templates::files(db) {
        let path = root.join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create `{}`: {e}", parent.display()))?;
        }
        let content = templates::render(file.content, &ctx);
        fs::write(&path, content).map_err(|e| format!("cannot write `{}`: {e}", path.display()))?;
        ui::created(style, file.path);
    }

    // Best-effort: initialize a git repository (ignored if git is unavailable).
    let _ = Command::new("git").args(["init", "-q"]).current_dir(&root).status();

    ui::success(style, &format!("{name} is ready"));
    if !args.here {
        ui::step(style, &format!("cd {name}"));
    }
    if db != Database::None {
        ui::step(style, "tork migrate up      # create the database schema");
    }
    ui::step(style, "tork dev             # run with live reload");
    println!();
    Ok(())
}

/// Resolves the project name from the argument or an interactive prompt.
fn resolve_name(args: &NewArgs, style: &Style, interactive: bool) -> Result<String, String> {
    if let Some(name) = &args.name {
        return validate_name(name);
    }
    if !interactive {
        return Err("a project name is required (e.g. `tork new my_app`)".to_owned());
    }
    loop {
        let answer = ui::prompt(style, "Project name?", Some("my_app"))
            .map_err(|e| format!("could not read input: {e}"))?;
        match validate_name(&answer) {
            Ok(name) => return Ok(name),
            Err(message) => ui::error(style, &message),
        }
    }
}

/// Resolves the database backend from `--db` or interactive prompts.
fn resolve_database(args: &NewArgs, style: &Style, interactive: bool) -> Result<Database, String> {
    if let Some(value) = &args.db {
        return Database::parse(value)
            .ok_or_else(|| format!("unknown database `{value}` (sqlite, postgres, mysql, or none)"));
    }
    if !interactive {
        // Non-interactive default keeps the previous behavior: a SQLite project.
        return Ok(Database::Sqlite);
    }
    let wants_db = ui::confirm(style, "Add a database?", true)
        .map_err(|e| format!("could not read input: {e}"))?;
    if !wants_db {
        return Ok(Database::None);
    }
    let choice = ui::select(
        style,
        "Which database?",
        &["SQLite", "PostgreSQL", "MySQL"],
        0,
    )
    .map_err(|e| format!("could not read input: {e}"))?;
    Ok(match choice {
        1 => Database::Postgres,
        2 => Database::Mysql,
        _ => Database::Sqlite,
    })
}

/// Validates a project name, returning it owned on success.
fn validate_name(name: &str) -> Result<String, String> {
    if is_valid_name(name) {
        Ok(name.to_owned())
    } else {
        Err(format!(
            "`{name}` is not a valid project name (start with a letter; use letters, digits, `_`, `-`)"
        ))
    }
}

/// A conservative crate-name check (a leading letter/underscore, then word chars).
fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
