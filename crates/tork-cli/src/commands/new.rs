//! `tork new` — scaffold a uniform Tork project (no `mod.rs`; 2018-style sibling
//! module files).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tork_orm_cli::Style;

use crate::cli::NewArgs;
use crate::templates;
use crate::ui;

pub fn run(args: &NewArgs, style: &Style) -> Result<(), String> {
    if !is_valid_name(&args.name) {
        return Err(format!(
            "`{}` is not a valid project name (start with a letter; use letters, digits, `_`, `-`)",
            args.name
        ));
    }

    let root = if args.here {
        PathBuf::from(".")
    } else {
        PathBuf::from(&args.name)
    };
    if !args.here {
        if root.exists() {
            return Err(format!("`{}` already exists", args.name));
        }
        fs::create_dir(&root).map_err(|e| format!("cannot create `{}`: {e}", root.display()))?;
    }

    let dep = templates::git_dep(&args.git, args.branch.as_deref());
    ui::header(style, &format!("Creating {}", args.name));

    for file in templates::files() {
        let path = root.join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create `{}`: {e}", parent.display()))?;
        }
        let content = templates::render(file.content, &args.name, &dep);
        fs::write(&path, content).map_err(|e| format!("cannot write `{}`: {e}", path.display()))?;
        ui::created(style, file.path);
    }

    // Best-effort: initialize a git repository (ignored if git is unavailable).
    let _ = Command::new("git").args(["init", "-q"]).current_dir(&root).status();

    ui::success(style, &format!("{} is ready", args.name));
    if !args.here {
        ui::step(style, &format!("cd {}", args.name));
    }
    ui::step(style, "tork migrate up      # create the database schema");
    ui::step(style, "tork dev             # run with live reload");
    println!();
    Ok(())
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
