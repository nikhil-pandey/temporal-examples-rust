# Async Activity Completion Example

Demonstrates how to implement Activities that are completed asynchronously ("out of band") using the Temporal client from outside the activity execution context.

## How to run

1. Start the Temporal server:
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
4. Observe that the activity completes after a short delay, due to async (spawned task) completion.

See more at [../..](../../README.md).
