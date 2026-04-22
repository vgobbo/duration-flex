# Agentic Instructions for Duration Flex

As an AI agent working on this repository, you must adhere to the following standards and workflows.

## Context & Purpose
`duration-flex` is a Rust crate designed to make it easier to specify durations in configuration files using a human-readable format like `1h23m` or `1w6d23h49m59s`. It provides interoperability with `chrono` and `std::time` durations, and supports `serde` and `clap`.

## Engineering Standards

### Rust Development
- **Edition:** Rust 2021.
- **Formatting:** Use `cargo +nightly fmt`. A `rustfmt.toml` is provided with specific configurations (e.g., tabs for indentation).
- **Linting:** Use `cargo clippy`. The project enforces clean clippy runs (see `pre-commit` hook).
- **Testing:** Always run tests with `--all-features`: `cargo test --all-features`.
- **Toml Formatting:** Use `taplo fmt` for TOML files.

### Feature Management
- The project has two optional features: `clap` and `serde`.
- Use the `full` feature to enable both.
- Ensure all code changes are verified against combinations of these features.

### Commit Guidelines
- **Mandatory:** Use [Conventional Commits 1.0](https://www.conventionalcommits.org/en/v1.0.0/).
- **Atomic Commits:** Do not mix unrelated changes in a single commit unless explicitly requested. Each commit should represent a single logical change.
- Common types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`.

## Workflow & Tooling

### Git Hooks
- The project uses custom hooks located in `scripts/hooks/`.
- **Pre-commit:** Located at `scripts/hooks/impl/pre-commit`. It automatically runs `cargo clippy --fix`, `cargo +nightly fmt`, and `taplo fmt` on staged files.
- **Commit-msg:** Located at `scripts/hooks/impl/commit-msg`. It ensures the commit message follows Conventional Commits 1.0.
- Ensure these tools are installed in your environment:
  - `fish` shell (required to run scripts).
  - `taplo` (`cargo install taplo-cli`).
  - `nightly` rust toolchain.

### Target Setup
- If working in a fresh environment, you might need to run `./scripts/setup-hooks`.
- Optionally, `./scripts/setup-target` can be used to initialize a temporary `target/` directory.

## File Structure
- `src/lib.rs`: The main library implementation.
- `Cargo.toml`: Dependency and feature definitions.
- `scripts/`: Development and automation scripts.
- `CRATE.md`: The documentation used for the crate on crates.io.
