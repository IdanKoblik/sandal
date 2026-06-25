# Contributing to Sandal

Thanks for your interest in contributing! This document describes how to
build the project, the code conventions we follow, and how to get your
changes merged.

> ⚠️ Sandal is in active development — interfaces and internals may change
> without notice.

## Getting Started

Sandal is a Linux RPG shell written in **Rust**. You'll need:

- A recent stable [Rust toolchain](https://rustup.rs/) (`cargo`, `rustc`)
- `rustfmt` and `clippy` components (`rustup component add rustfmt clippy`)
- `make` (optional — it just wraps the `cargo` commands below)

### Build & Run

The `Makefile` is a thin convenience wrapper around `cargo`:

```sh
make build      # cargo build
make release    # cargo build --release
make run        # cargo run
make test       # cargo test
make fmt        # cargo fmt
make fmt-check  # cargo fmt --check
make lint       # cargo clippy -- -D warnings
make check      # fmt-check + lint + test
make clean      # cargo clean
```

If you'd rather not use `make`, the equivalent `cargo` commands work the
same way.

## Code Conventions

Match the surrounding code. The key rules:

- **Language:** Rust, edition 2024.
- **Formatting:** all code must be formatted with `cargo fmt`. Run
  `make fmt` before committing; `make fmt-check` must pass.
- **Linting:** code must be clippy-clean. `cargo clippy -- -D warnings`
  (`make lint`) must produce no warnings.
- **Indentation:** 4 spaces, no tabs (except in the `Makefile`, where tabs
  are required). An `.editorconfig` is provided — please use an editor
  that respects it.
- **Naming:** follow standard Rust conventions — `snake_case` for
  functions, variables, and modules; `CamelCase` for types and traits;
  `SCREAMING_SNAKE_CASE` for constants.
- **Errors:** prefer returning `Result` and propagating with `?` over
  panicking. Surface user-facing errors through the shell prompt
  (e.g. `println!("sandal: {err}")`) as `main.rs` does.
- **Comments:** brief `//` comments to explain non-obvious intent. Don't
  over-comment obvious code.

## Testing

Unit tests live alongside the code they exercise, in a
`#[cfg(test)] mod tests` block at the bottom of the relevant module (see
`src/completion.rs` for a template).

- **When to add tests:** add or update tests for any new logic or bug fix
  whose behavior can be exercised in isolation (e.g. command parsing,
  completion, editor logic).
- **How to add a test:** add `#[test]` functions inside the module's
  `mod tests` block, or create one if it doesn't exist yet.
- **Run them:** `make test` (or `cargo test`). All tests must pass before
  a PR is merged.

## Commits & Pull Requests

- We follow [Conventional Commits](https://www.conventionalcommits.org/):
  `feat:`, `fix:`, `chore:`, `docs:`, etc. — optionally with a scope,
  e.g. `chore(build): Update Makefile`.
- Keep commits focused and the history readable.
- Open your PR against `main` and fill out the pull request template.
- Before submitting, make sure `make check` passes — that is:
  - `cargo fmt --check` is clean,
  - `cargo clippy -- -D warnings` produces no warnings,
  - `cargo test` passes.

## Reporting Bugs & Requesting Features

Use the issue templates (Bug report / Feature request) when opening an
issue.

Thanks for contributing!
