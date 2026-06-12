# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

- `cargo build` — build the project
- `cargo run` — run the web server (default port 10119, override with `PORT=8080 cargo run`)
- `cargo run <subcommand>` — CLI subcommands: `server`, `init-db`, `set-password <pwd>`, `import <file>`, `clear`
- `cargo check` — type-check without producing a binary
- `cargo test` — run all tests
- `cargo test <test_name>` — run a specific test by name
- `cargo clippy` — lint the codebase
- `cargo fmt` — format all Rust source files

## Configuration

- Config file: `config.toml` (auto-created on first run)
- `admin_password` — password for import/clear/edit/delete operations
- Override config path via `CONFIG_PATH` env var

## Project Structure

```
src/main.rs      — entry point, routes, handlers, CLI dispatch
src/config.rs    — config.toml read/write
src/cli.rs       — clap CLI argument definitions
src/db.rs        — SQLite pool init, search, insert, update, delete
src/excel.rs     — Excel template generation and parsing
src/models.rs    — Bank, SearchResponse, BankUpdate structs
templates/       — HTML template (embedded via include_str!)
```
