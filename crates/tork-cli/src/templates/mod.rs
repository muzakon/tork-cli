//! The `tork new` project template.
//!
//! Files are static strings with placeholders (`@NAME@`, the dependency tables, and
//! a few database-conditional fragments). The generated layout is uniform and free
//! of `mod.rs`: every directory has a sibling `<dir>.rs` that declares its
//! submodules, and the top-level modules are declared in `main.rs`.

/// The database backend chosen for a generated project.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Database {
    None,
    Sqlite,
    Postgres,
    Mysql,
}

impl Database {
    /// Parses a `--db` flag value.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "none" | "no" | "" => Some(Database::None),
            "sqlite" => Some(Database::Sqlite),
            "postgres" | "postgresql" | "pg" => Some(Database::Postgres),
            "mysql" | "mariadb" => Some(Database::Mysql),
            _ => None,
        }
    }

    /// The dialect token written into the project's migration metadata.
    fn dialect(self) -> &'static str {
        match self {
            Database::None => "",
            Database::Sqlite => "sqlite",
            Database::Postgres => "postgres",
            Database::Mysql => "mysql",
        }
    }

    /// A sensible default connection URL for this backend.
    fn default_url(self) -> &'static str {
        match self {
            Database::None => "",
            Database::Sqlite => "sqlite://app.db",
            Database::Postgres => "postgres://postgres:postgres@localhost:5432/app",
            Database::Mysql => "mysql://root:root@localhost:3306/app",
        }
    }
}

/// One file in the generated project.
pub struct File {
    /// The path relative to the project root.
    pub path: &'static str,
    /// The file contents (with placeholders).
    pub content: &'static str,
}

/// Builds the git dependency table written for the framework (`tork`).
pub fn git_dep(git: &str, branch: Option<&str>) -> String {
    match branch {
        Some(branch) => format!("{{ git = \"{git}\", branch = \"{branch}\" }}"),
        None => format!("{{ git = \"{git}\" }}"),
    }
}

/// Builds the `tork-orm` dependency table, selecting the backend feature.
///
/// SQLite uses the default features; Postgres/MySQL turn the defaults off and
/// enable their backend plus the framework integration and migrations.
pub fn orm_dep(git: &str, branch: Option<&str>, db: Database) -> String {
    let mut table = format!("{{ git = \"{git}\"");
    if let Some(branch) = branch {
        table.push_str(&format!(", branch = \"{branch}\""));
    }
    match db {
        Database::Postgres => {
            table.push_str(", default-features = false, features = [\"postgres\", \"tork\", \"migrations\"]");
        }
        Database::Mysql => {
            table.push_str(", default-features = false, features = [\"mysql\", \"tork\", \"migrations\"]");
        }
        _ => {}
    }
    table.push_str(" }");
    table
}

/// The substitutions applied to every template file.
pub struct Context {
    pub name: String,
    pub tork_dep: String,
    pub orm_dep: String,
    pub db: Database,
}

/// Substitutes the placeholders in a template.
pub fn render(content: &str, ctx: &Context) -> String {
    let has_db = ctx.db != Database::None;

    let orm_dep_line = if has_db {
        format!("tork-orm = {}\n", ctx.orm_dep)
    } else {
        String::new()
    };
    let tork_metadata = if has_db {
        format!(
            "\n[package.metadata.tork]\ndialect = \"{}\"\n\n\
             [package.metadata.tork.migrations]\ndir = \"migrations\"\n\
             file_template = \"{{rev}}_{{slug}}\"\nrevision_style = \"sequence\"\n\
             truncate_slug_length = 40\n",
            ctx.db.dialect()
        )
    } else {
        String::new()
    };
    let core_db_mod = if has_db { "pub mod db;\n" } else { "" };
    let db_lifespan = if has_db {
        "        .lifespan::<core::db::Db>()\n"
    } else {
        ""
    };
    let env_db = if has_db {
        format!("DB_URL={}\nDB_MAX_CONNECTIONS=5\n", ctx.db.default_url())
    } else {
        String::new()
    };

    content
        .replace("@NAME@", &ctx.name)
        .replace("@TORK_DEP@", &ctx.tork_dep)
        .replace("@ORM_DEP_LINE@", &orm_dep_line)
        .replace("@TORK_METADATA@", &tork_metadata)
        .replace("@CORE_DB_MOD@", core_db_mod)
        .replace("@DB_LIFESPAN@", db_lifespan)
        .replace("@DB_URL@", ctx.db.default_url())
        .replace("@ENV_DB@", &env_db)
}

/// Every file in the generated project, in creation order.
pub fn files(db: Database) -> Vec<File> {
    let mut files = vec![
        File { path: "Cargo.toml", content: CARGO_TOML },
        File { path: "rust-toolchain.toml", content: RUST_TOOLCHAIN },
        File { path: ".gitignore", content: GITIGNORE },
        File { path: ".env.example", content: ENV_EXAMPLE },
        File { path: "README.md", content: README },
        File { path: "src/main.rs", content: MAIN_RS },
        File { path: "src/routers.rs", content: ROUTERS_RS },
        File { path: "src/routers/health.rs", content: HEALTH_RS },
        File { path: "src/models.rs", content: MODELS_RS },
        File { path: "src/services.rs", content: SERVICES_RS },
        File { path: "src/services/.gitkeep", content: "" },
        File { path: "src/repositories.rs", content: REPOSITORIES_RS },
        File { path: "src/repositories/.gitkeep", content: "" },
        File { path: "src/core.rs", content: CORE_RS },
        File { path: "src/core/settings.rs", content: SETTINGS_RS },
    ];
    if db != Database::None {
        files.push(File { path: "migrations/.gitkeep", content: "" });
        files.push(File { path: "src/core/db.rs", content: DB_RS });
    }
    files
}

const CARGO_TOML: &str = r#"[package]
name = "@NAME@"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
tork = @TORK_DEP@
@ORM_DEP_LINE@# garde's derive references `::garde` directly, so #[api_model] models need it.
garde = { version = "0.23", features = ["derive", "email"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
@TORK_METADATA@"#;

const RUST_TOOLCHAIN: &str = r#"[toolchain]
channel = "1.96"
"#;

const GITIGNORE: &str = "/target\n.env\n*.db\n*.db-wal\n*.db-shm\n";

const ENV_EXAMPLE: &str = "# Copy to .env and adjust.\nAPP_NAME=@NAME@\nAPP_HOST=0.0.0.0:8000\n@ENV_DB@";

const README: &str = r#"# @NAME@

A web service built on the [Tork](https://github.com/muzakon/tork-framework) framework.

## Develop

```sh
tork dev             # run with live reload (http://localhost:8000)
```

`GET /health` returns a liveness check; `GET /docs` serves the OpenAPI UI.

## Layout

```
src/
  main.rs          application entry point
  routers/         HTTP routers (one module per router)
  services/        business logic
  repositories/    data access
  models/ models.rs  serializable API models
  core/            settings, database, shared wiring
```

Configuration lives in `src/core/settings.rs` (`APP_*` environment variables). Add a
module by creating its file and declaring it in the sibling `<dir>.rs`.
"#;

const MAIN_RS: &str = r#"mod core;
mod models;
mod repositories;
mod routers;
mod services;

use tork::{App, OpenApi};

#[tork::main]
async fn main() -> tork::Result<()> {
    let config = core::settings::AppConfig::load()?;
    App::new()
@DB_LIFESPAN@        .include_router(routers::router())
        .openapi(
            OpenApi::new()
                .title(config.name.as_str())
                .version("0.1.0")
                .json("/openapi.json")
                .docs("/docs"),
        )
        .serve(&config.host)
        .await
}
"#;

const ROUTERS_RS: &str = r#"//! HTTP routers. Each submodule is a router; declare new ones with `pub mod`.

pub mod health;

/// The application's combined router.
pub fn router() -> tork::Router {
    health::router()
}
"#;

const HEALTH_RS: &str = r#"//! Health-check router.

use tork::{api_router, get};

use crate::models::HealthOut;

#[api_router(prefix = "/health", tags = ["health"])]
pub mod health_router {
    use super::*;

    /// Liveness probe.
    #[get("", response_model = HealthOut, summary = "Health check")]
    pub async fn health() -> tork::Result<HealthOut> {
        Ok(HealthOut {
            status: "ok".to_string(),
        })
    }
}

pub use health_router::router;
"#;

const MODELS_RS: &str = r#"//! Serializable API models (DTOs).

use tork::api_model;

/// The health-check response.
#[api_model]
pub struct HealthOut {
    pub status: String,
}
"#;

const SERVICES_RS: &str = r#"//! Business-logic services.
//!
//! Add one by creating `src/services/<name>.rs` and declaring `pub mod <name>;`
//! here.
"#;

const REPOSITORIES_RS: &str = r#"//! Data-access repositories.
//!
//! Add one by creating `src/repositories/<name>.rs` and declaring
//! `pub mod <name>;` here.
"#;

const CORE_RS: &str = r#"//! Core wiring: settings, database, and shared state.

pub mod settings;
@CORE_DB_MOD@"#;

const SETTINGS_RS: &str = r#"//! Application settings, loaded from the environment (prefix `APP`).

use tork::settings;

/// Top-level application settings. Override any field with an `APP_*` environment
/// variable (for example `APP_NAME`), a `.env` file, or process environment.
#[settings(prefix = "APP")]
pub struct AppConfig {
    /// Human-readable application name (`APP_NAME`).
    #[setting(default = "@NAME@")]
    pub name: String,

    /// Address the HTTP server binds to (`APP_HOST`).
    #[setting(default = "0.0.0.0:8000")]
    pub host: String,
}
"#;

const DB_RS: &str = r#"//! The database resource and its lifespan: connect and run migrations at startup.

use std::sync::Arc;

use tork::{settings, LifespanContext, Resources, Result};
use tork_orm::migration::FileMigrator;
use tork_orm::prelude::*;

/// Database settings, loaded from the environment (prefix `DB`).
#[settings(prefix = "DB")]
pub struct DatabaseConfig {
    #[setting(default = "@DB_URL@")]
    pub url: String,
    #[setting(default = 5, ge = 1, le = 64)]
    pub max_connections: u32,
}

/// The database resource, injected into handlers as `Arc<Database>`.
#[derive(Clone, Resources)]
pub struct Db {
    #[resource]
    pub database: Arc<Database>,
}

#[tork::lifespan]
impl Db {
    /// Connects to the database and applies pending migrations.
    async fn startup(_ctx: LifespanContext) -> Result<Self> {
        let config = DatabaseConfig::load()?;
        let database = Database::connect(&config.url, config.max_connections).await?;
        FileMigrator::new(database.clone(), "migrations")
            .up()
            .await?;
        Ok(Db {
            database: Arc::new(database),
        })
    }

    /// Closes the connection pool at shutdown.
    async fn shutdown(self) -> Result<()> {
        self.database.close().await;
        Ok(())
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(db: Database) -> Context {
        Context {
            name: "demo".to_owned(),
            tork_dep: "{ git = \"g\" }".to_owned(),
            orm_dep: orm_dep("g", None, db),
            db,
        }
    }

    #[test]
    fn database_parse_accepts_aliases_and_none() {
        assert!(matches!(Database::parse("sqlite"), Some(Database::Sqlite)));
        assert!(matches!(Database::parse("PostgreSQL"), Some(Database::Postgres)));
        assert!(matches!(Database::parse("pg"), Some(Database::Postgres)));
        assert!(matches!(Database::parse("mysql"), Some(Database::Mysql)));
        assert!(matches!(Database::parse("none"), Some(Database::None)));
        assert!(Database::parse("oracle").is_none());
    }

    #[test]
    fn settings_file_is_always_rendered() {
        for db in [Database::None, Database::Sqlite, Database::Postgres] {
            let paths: Vec<&str> = files(db).iter().map(|f| f.path).collect();
            assert!(paths.contains(&"src/core/settings.rs"), "settings.rs missing for a variant");
        }
    }

    #[test]
    fn db_files_only_present_with_a_database() {
        let with: Vec<&str> = files(Database::Sqlite).iter().map(|f| f.path).collect();
        assert!(with.contains(&"src/core/db.rs"));
        assert!(with.contains(&"migrations/.gitkeep"));

        let without: Vec<&str> = files(Database::None).iter().map(|f| f.path).collect();
        assert!(!without.contains(&"src/core/db.rs"));
        assert!(!without.contains(&"migrations/.gitkeep"));
    }

    #[test]
    fn cargo_toml_includes_orm_only_with_a_database() {
        let with_db = render(CARGO_TOML, &ctx(Database::Sqlite));
        assert!(with_db.contains("tork-orm = "));
        assert!(with_db.contains("dialect = \"sqlite\""));

        let no_db = render(CARGO_TOML, &ctx(Database::None));
        assert!(!no_db.contains("tork-orm"));
        assert!(!no_db.contains("metadata.tork"));
    }

    #[test]
    fn postgres_orm_dep_selects_the_backend_feature() {
        let table = orm_dep("g", None, Database::Postgres);
        assert!(table.contains("default-features = false"));
        assert!(table.contains("\"postgres\""));
    }

    #[test]
    fn main_includes_the_db_lifespan_only_with_a_database() {
        assert!(render(MAIN_RS, &ctx(Database::Sqlite)).contains(".lifespan::<core::db::Db>()"));
        assert!(!render(MAIN_RS, &ctx(Database::None)).contains(".lifespan"));
    }
}
