# Child Workflows Example

Shows a parent workflow spawning multiple child workflows concurrently and collecting their results, emulating the "child workflows" pattern.

## How to run

1. Start Temporal server:
   ```
   temporal server start-dev
   ```
2. Start the worker for this crate:
   ```
   cargo run --bin worker
   ```
3. In another terminal, start the client:
   ```
   cargo run --bin client
   ```
4. The parent workflow will spawn, await, and collect results from two child workflows.

Learn more from the [main workspace README](../../README.md).
