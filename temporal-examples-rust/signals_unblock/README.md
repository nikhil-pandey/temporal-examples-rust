# Signals Unblock Example

An example workflow that blocks until it receives a signal. Demonstrates using signals to unblock workflow execution in Rust.

## How to run

1. Start the Temporal dev server:
   ```
   temporal server start-dev
   ```
2. Run the worker:
   ```
   cargo run --bin worker
   ```
3. In another terminal, run the client:
   ```
   cargo run --bin client
   ```
4. The workflow will unblock and complete after client sends the required signal.

See [../..](../../README.md) for details and more samples.
