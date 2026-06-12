# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

- `cargo build` — build the project
- `cargo run` — run the binary (default port 2222, override with `PORT=8080 cargo run`)
- `cargo check` — type-check without producing a binary (fastest feedback)
- `cargo test` — run all tests
- `cargo test <test_name>` — run a specific test by name
- `cargo clippy` — lint the codebase
- `cargo fmt` — format all Rust source files

## Project Structure

A single-crate binary project with no external dependencies. `src/main.rs` is the entry point — currently a minimal "Hello, world!" scaffold.

```
src/main.rs   — binary entry point
Cargo.toml    — crate manifest (edition 2024)
```

As the project grows, modules will live under `src/` (e.g., `src/query.rs`, `src/bank.rs`).
