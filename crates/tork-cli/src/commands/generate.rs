//! `tork migrate generate`: build the project, diff its models against the live
//! database, and write the resulting migration.
//!
//! Every `#[derive(Model)]` registers its schema in a link-time registry, so the
//! model schema only exists inside the *compiled* project — not in this CLI binary.
//! Generate therefore cannot run the way the other `migrate` subcommands do.
//! Instead it writes a tiny generator binary into the project, runs it with
//! `cargo run` (which links the project's models and database driver), and removes
//! it afterwards. The generator connects to the database and diffs it against the
//! models, so a live database connection is required.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tork_orm_cli::cli::GlobalArgs;
use tork_orm_cli::Style;

use crate::ui;

/// The name of the throwaway generator binary written into `src/bin/`.
const GEN_BIN: &str = "__tork_generate";

/// Builds the project's models, diffs them against the live database, and writes a
/// migration named `name` into the configured migrations directory.
pub fn run(name: &str, global: &GlobalArgs, style: &Style) -> Result<(), String> {
    let root = std::env::current_dir()
        .map_err(|e| format!("cannot read the current directory: {e}"))?;

    let manifest = root.join("Cargo.toml");
    let cargo_toml = std::fs::read_to_string(&manifest).map_err(|e| {
        format!(
            "cannot read {}: {e}; run `tork migrate generate` from your project root",
            manifest.display()
        )
    })?;
    let parsed: toml::Value = cargo_toml
        .parse()
        .map_err(|e| format!("invalid Cargo.toml: {e}"))?;

    let crate_name = parsed
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or("Cargo.toml has no [package] name")?;
    // The crate path identifier replaces dashes with underscores.
    let crate_ident = crate_name.replace('-', "_");

    // The migrations directory: --dir, then [package.metadata.tork.migrations].dir,
    // then the conventional `migrations`.
    let dir = global.dir.clone().unwrap_or_else(|| {
        parsed
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("tork"))
            .and_then(|t| t.get("migrations"))
            .and_then(|m| m.get("dir"))
            .and_then(|d| d.as_str())
            .unwrap_or("migrations")
            .to_string()
    });

    // The diff needs a live database, so a URL is mandatory.
    let db_url = global
        .database_url
        .clone()
        .or_else(|| std::env::var("DB_URL").ok())
        .ok_or(
            "no database URL; pass --database-url or set DATABASE_URL / DB_URL \
             (generate diffs your models against the live database)",
        )?;

    std::fs::create_dir_all(root.join(&dir))
        .map_err(|e| format!("cannot create the migrations directory `{dir}`: {e}"))?;

    // Write the generator binary; the guard removes it however we return.
    let _bin = TempBin::write(&root, &crate_ident)?;

    ui::running(style, "cargo run (building the project to read its models)");

    let child = Command::new("cargo")
        .current_dir(&root)
        .args(["run", "--bin", GEN_BIN])
        .env("TORK_GEN_DB_URL", &db_url)
        .env("TORK_GEN_DIR", &dir)
        .env("TORK_GEN_NAME", name)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("could not run cargo ({e}); is it installed and on PATH?"))?;

    // stderr is inherited (build progress streams live); stdout carries the result.
    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed while running the generator: {e}"))?;

    if !output.status.success() {
        return Err(
            "generate failed while building or running the project (see the output above)"
                .to_string(),
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(path) = stdout
        .lines()
        .find_map(|line| line.strip_prefix("__TORK_GENERATED__"))
    {
        ui::created(style, path);
        ui::success(style, "Generated migration from your models");
        Ok(())
    } else if stdout.contains("__TORK_NOCHANGE__") {
        ui::success(style, "Schema already matches your models; nothing to generate");
        Ok(())
    } else {
        Err("the generator produced no result; check the output above".to_string())
    }
}

/// The throwaway generator source, removed on drop.
struct TempBin {
    /// The generator file (`src/bin/__tork_generate.rs`).
    path: PathBuf,
    /// `src/bin` if this command created it (so it can be cleaned up when empty).
    created_dir: Option<PathBuf>,
}

impl TempBin {
    /// Writes the generator into `<root>/src/bin/`, creating the directory if absent.
    fn write(root: &Path, crate_ident: &str) -> Result<Self, String> {
        let bin_dir = root.join("src").join("bin");
        let created_dir = if bin_dir.exists() {
            None
        } else {
            std::fs::create_dir_all(&bin_dir)
                .map_err(|e| format!("cannot create {}: {e}", bin_dir.display()))?;
            Some(bin_dir.clone())
        };
        let path = bin_dir.join(format!("{GEN_BIN}.rs"));
        std::fs::write(&path, source(crate_ident))
            .map_err(|e| format!("cannot write the generator binary: {e}"))?;
        Ok(Self { path, created_dir })
    }
}

impl Drop for TempBin {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        // Only removes the directory if we created it and it is now empty.
        if let Some(dir) = &self.created_dir {
            let _ = std::fs::remove_dir(dir);
        }
    }
}

/// Renders the generator source for a project whose lib crate is `crate_ident`.
///
/// `use <crate> as _;` forces the project's library (and thus every model's
/// link-time registration) into the binary without importing any names.
fn source(crate_ident: &str) -> String {
    const TEMPLATE: &str = r#"//! Temporary generator written by `tork migrate generate`. Safe to delete.
#![allow(unused_extern_crates)]
use __CRATE__ as _;

use std::path::Path;
use std::process::exit;

use tork_orm::migration::generate::generate_and_write;
use tork_orm::Database;

#[tokio::main]
async fn main() {
    let url = std::env::var("TORK_GEN_DB_URL").expect("TORK_GEN_DB_URL");
    let dir = std::env::var("TORK_GEN_DIR").expect("TORK_GEN_DIR");
    let name = std::env::var("TORK_GEN_NAME").expect("TORK_GEN_NAME");
    let db = match Database::connect(&url, 1).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("tork: could not connect to the database: {e}");
            exit(2);
        }
    };
    match generate_and_write(&db, Path::new(&dir), &name).await {
        Ok(Some(path)) => println!("__TORK_GENERATED__{}", path.display()),
        Ok(None) => println!("__TORK_NOCHANGE__"),
        Err(e) => {
            eprintln!("tork: generate failed: {e}");
            exit(3);
        }
    }
}
"#;
    TEMPLATE.replace("__CRATE__", crate_ident)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_links_the_project_crate_and_calls_generate() {
        let src = source("my_api");
        // Forces the project's lib (and its link-time model registry) into the binary.
        assert!(src.contains("use my_api as _;"));
        // Diffs the registered models against the live database.
        assert!(src.contains("generate_and_write"));
        assert!(src.contains("Database::connect"));
        // Reports its result on stdout for the parent process to parse.
        assert!(src.contains("__TORK_GENERATED__"));
        assert!(src.contains("__TORK_NOCHANGE__"));
    }

    #[test]
    fn source_substitutes_dashed_crate_names_as_underscores() {
        // Callers pass the path identifier (dashes already turned into underscores).
        let src = source("server_app");
        assert!(src.contains("use server_app as _;"));
        assert!(!src.contains("__CRATE__"));
    }
}
