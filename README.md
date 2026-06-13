# tork CLI

The developer CLI for the [Tork](https://github.com/tork-rs/tork) web framework:
scaffold a project, run database migrations, and drive the build/dev loop — all behind
one colored interface.

```
tork new        Scaffold a new Tork project
tork migrate    Database migrations (up, down, status, create, redo, init)
tork build      Compile the project (wraps cargo build)
tork check      Type-check the project (cargo check, or --clippy)
tork format     Format the code (--fix to apply)
tork dev        Run the project, restarting on file changes
```

## Install

The CLI is part of the [tork superproject](https://github.com/tork-rs/tork) and is
built from there (it shares crates with the ORM by path):

```sh
git clone --recurse-submodules https://github.com/tork-rs/tork.git
cd tork
cargo install --path cli/crates/tork-cli      # installs the `tork` binary
```

Or just build it and use the binary directly: `cargo build --manifest-path cli/Cargo.toml`
then `cli/target/debug/tork`.

## Getting started: create a project

```sh
tork new my-api          # scaffold ./my-api
cd my-api
tork migrate up          # create the database schema (sqlite://app.db by default)
tork dev                 # run with live reload → http://localhost:8000
```

`GET /health` returns a liveness check, and `GET /docs` serves the OpenAPI UI.

### What `tork new` generates

A uniform project with no `mod.rs` files (each directory has a sibling `<dir>.rs` that
declares its modules):

```
my-api/
  Cargo.toml            # git deps on tork + tork-orm; migration config
  rust-toolchain.toml
  .env.example          # RUST_LOG, DB_URL
  migrations/           # SQL migrations (managed by `tork migrate`)
  src/
    main.rs             # App::new().lifespan(...).include_router(...).serve(...)
    routers.rs          # declares each router module
    routers/health.rs   # a sample #[api_router] / #[get("")] endpoint
    models.rs           # #[api_model] DTOs
    services.rs   services/         # business logic (add files here)
    repositories.rs   repositories/ # data access
    core.rs   core/db.rs            # config + database lifespan (runs migrations at startup)
```

Add a module by creating its file (e.g. `src/services/billing.rs`) and declaring
`pub mod billing;` in the sibling `src/services.rs`.

Options: `tork new <name> [--here] [--framework-git <url>] [--orm-git <url>] [--branch <ref>]`.
`--here` scaffolds into the current directory; the git flags pin the generated project's
`tork`/`tork-orm` dependencies.

## Migrations

Run from inside a project (the CLI reads `[package.metadata.tork.migrations]` from
`Cargo.toml`). The database URL comes from `-d/--database-url`, `DATABASE_URL`, or `DB_URL`.

```sh
tork migrate init                 # create the migrations directory
tork migrate create add_users     # scaffold a new SQL migration file
tork migrate up                   # apply all pending  (also: up <revision>)
tork migrate status               # show applied / pending
tork migrate down                 # revert one  (also: down <n> | base | <revision>)
tork migrate redo                 # revert the most recent and re-apply
```

Migrations are plain `.sql` files with `-- revision:` / `-- migrate:up` / `-- migrate:down`
headers, so the binary needs no compilation — just a `migrations/` directory and a
database URL. The matching schema is usually generated from your models; see the
[ORM migration guide](https://github.com/tork-rs/tork-orm/blob/main/docs/08-migrations-and-cli.md).

Global migration flags: `--dir <path>` (migrations directory), `--table <name>`
(bookkeeping table, default `_tork_migrations`), `-y/--yes` (skip confirmation),
`--allow-checksum-mismatch` (downgrade an edited-applied-migration error to a
warning; development only — `tork migrate` aborts on checksum drift by default).

## Build / check / format / dev

Transparent wrappers over `cargo`, plus a self-contained watcher for `dev`:

```sh
tork build                # cargo build  (extra args forwarded: tork build --release)
tork check                # cargo check   (tork check --clippy → cargo clippy)
tork format               # verify formatting (cargo fmt -- --check)
tork format --fix         # apply formatting (cargo fmt)
tork dev                  # watch src/, Cargo.toml, migrations/ → rebuild & restart on save
tork dev --bin worker     # run a specific binary target
```

`--no-color` (or `NO_COLOR=1`, or piping) disables colored output. `tork --help` and
`tork <command> --help` print full usage.
