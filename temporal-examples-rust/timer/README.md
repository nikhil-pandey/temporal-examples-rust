# Timer Example

Demonstrates running a long-running operation (simulated order processing) with a concurrent timer. If the order processing exceeds a threshold, a notification email activity is triggered.

## How to run

1. Start the Temporal dev server:
   ```
   temporal server start-dev
   ```
2. In this directory, start the worker:
   ```
   cargo run --bin worker
   ```
3. In another terminal, start the client (optionally give a threshold in seconds, defaults to 2):
   ```
   cargo run --bin client -- 6
   ```
4. Each run may or may not trigger the notification depending on the random duration.

See [../..](../../README.md) for workspace-wide context and more samples.
