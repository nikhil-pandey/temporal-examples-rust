# Contributing Guidelines

Thank you for your interest in contributing to the Temporal Rust Examples project!

To ensure consistency and code quality across all examples, please follow these guidelines when submitting changes.

## Prerequisites

- Use the **nightly Rust toolchain** (see `rust-toolchain.toml` in the repo root).
- Make sure you have [temporal](https://github.com/temporalio/cli) CLI installed for starting and managing Temporal server.

## Code Style & Lint

- Format code using [`cargo fmt`](https://github.com/rust-lang/rustfmt) before committing.
- All code **must pass** [`cargo clippy -D warnings`](https://github.com/rust-lang/rust-clippy).
- Keep all lines in Markdown and code ≤100 characters where possible.

## PR Checklist

Before submitting a Pull Request, make sure to:

1. **Check your toolchain**:
   - Use `rustup show` to ensure `nightly` is active (matches repo's `rust-toolchain.toml`).
2. **Run all code and docs checks:**
   - Run `cargo fmt --all -- --check` to ensure formatting.
   - Run `cargo clippy --all -- -D warnings` to catch lints.
3. **Test all examples manually:**
   - Start a dev server: `temporal server start-dev`
   - For each example:
     - `cd <example-directory>`
     - `cargo run --bin worker` (in one terminal)
     - `cargo run --bin client` (in a second terminal, or as described per-example)
     - Confirm example executes as described in its README.
4. **Document new features clearly:**
   - Update or add README files for new or changed example crates, keeping descriptions ≤100 chars/line.

## Reporting Issues

- If you find missing features or API parity issues versus the TypeScript or Python SDKs, mention them in your PR or open an issue. For things unsupported by the Rust SDK, please add clear documentation or notes in the relevant `README.md`.

Thank you for helping improve the Rust Temporal examples!
