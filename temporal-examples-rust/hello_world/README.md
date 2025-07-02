# Hello World Example

This example demonstrates the simplest Temporal workflow possible: a `greet` workflow that takes a string name and returns a greeting.

## How to run

1. Start Temporal dev server (once per host):
   ```
   temporal server start-dev
   ```
2. In this crate directory, start the worker (keep this terminal open):
   ```
   cargo run --bin worker
   ```
3. In another terminal, start the client:
   ```
   cargo run --bin client
   ```
4. Optionally, pass a custom name to the workflow:
   ```
   cargo run --bin client -- Berty
   ```

For further details and more examples, see [../..](../../README.md).
