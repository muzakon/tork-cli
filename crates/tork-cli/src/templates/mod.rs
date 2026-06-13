//! The `tork new` project template.
//!
//! Files are static strings with `@NAME@` (project name) and `@DEP@` (the git
//! dependency table) placeholders. The generated layout is uniform and free of
//! `mod.rs`: every directory has a sibling `<dir>.rs` that declares its submodules,
//! and the top-level modules are declared in `main.rs`.

/// One file in the generated project.
pub struct File {
    /// The path relative to the project root.
    pub path: &'static str,
    /// The file contents (with `@NAME@` / `@DEP@` placeholders).
    pub content: &'static str,
}

/// Builds the git dependency table written into the generated `Cargo.toml`.
pub fn git_dep(git: &str, branch: Option<&str>) -> String {
    match branch {
        Some(branch) => format!("{{ git = \"{git}\", branch = \"{branch}\" }}"),
        None => format!("{{ git = \"{git}\" }}"),
    }
}

/// Substitutes the placeholders in a template.
pub fn render(content: &str, name: &str, dep: &str) -> String {
    content.replace("@NAME@", name).replace("@DEP@", dep)
}

/// Every file in the generated project, in creation order.
pub fn files() -> Vec<File> {
    vec![
        File { path: "Cargo.toml", content: CARGO_TOML },
        File { path: "rust-toolchain.toml", content: RUST_TOOLCHAIN },
        File { path: ".gitignore", content: GITIGNORE },
        File { path: ".env.example", content: ENV_EXAMPLE },
        File { path: "README.md", content: README },
        File { path: "migrations/.gitkeep", content: "" },
        File { path: "src/main.rs", content: MAIN_RS },
        File { path: "src/routers.rs", content: ROUTERS_RS },
        File { path: "src/routers/health.rs", content: HEALTH_RS },
        File { path: "src/models.rs", content: MODELS_RS },
        File { path: "src/services.rs", content: SERVICES_RS },
        File { path: "src/services/.gitkeep", content: "" },
        File { path: "src/repositories.rs", content: REPOSITORIES_RS },
        File { path: "src/repositories/.gitkeep", content: "" },
        File { path: "src/core.rs", content: CORE_RS },
        File { path: "src/core/db.rs", content: DB_RS },
    ]
}

const CARGO_TOML: &str = r#"[package]
name = "@NAME@"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
tork = @DEP@
tork-orm = @DEP@
# garde's derive references `::garde` directly, so #[api_model] models need it.
garde = { version = "0.23", features = ["derive", "email"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

[package.metadata.tork]
dialect = "sqlite"

[package.metadata.tork.migrations]
dir = "migrations"
file_template = "{rev}_{slug}"
revision_style = "sequence"
truncate_slug_length = 40
"#;

const RUST_TOOLCHAIN: &str = r#"[toolchain]
channel = "1.96"
"#;

const GITIGNORE: &str = "/target\n.env\n*.db\n*.db-wal\n*.db-shm\n";

const ENV_EXAMPLE: &str = "# Copy to .env and adjust.\nRUST_LOG=info\nDB_URL=sqlite://app.db\n";

const README: &str = r#"# @NAME@

A web service built on the [Tork](https://github.com/muzakon/tork) framework.

## Develop

```sh
tork migrate up      # apply database migrations
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
  core/            configuration, database, shared wiring
```

Add a module by creating its file and declaring it in the sibling `<dir>.rs`.
"#;

const MAIN_RS: &str = r#"mod core;
mod models;
mod repositories;
mod routers;
mod services;

use tork::{App, OpenApi};

#[tork::main]
async fn main() -> tork::Result<()> {
    App::new()
        .lifespan::<core::db::Db>()
        .include_router(routers::router())
        .openapi(
            OpenApi::new()
                .title("@NAME@")
                .version("0.1.0")
                .json("/openapi.json")
                .docs("/docs"),
        )
        .serve("0.0.0.0:8000")
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

const CORE_RS: &str = r#"//! Core wiring: configuration, database, and shared state.

pub mod db;
"#;

const DB_RS: &str = r#"//! The database resource and its lifespan: connect and run migrations at startup.

use std::sync::Arc;

use tork::{settings, LifespanContext, Resources, Result};
use tork_orm::migration::FileMigrator;
use tork_orm::prelude::*;

/// Database settings, loaded from the environment (prefix `DB`).
#[settings(prefix = "DB")]
pub struct DatabaseConfig {
    #[setting(default = "sqlite://app.db")]
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
