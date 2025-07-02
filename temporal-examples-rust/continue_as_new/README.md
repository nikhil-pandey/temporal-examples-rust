# Continue-As-New Example

Demonstrates the Continue-As-New pattern: a workflow loops, re-executing itself with new state until a completion condition.

## How to run

1. Start Temporal server:
   ```
   temporal server start-dev
   ```
2. Start the worker in this directory:
   ```
   cargo run --bin worker
   ```
3. In a separate terminal, start the client:
   ```
   cargo run --bin client
   ```
4. The client and worker will print progress as the workflow "continues as new" for 10 iterations.

See the [repository README](../../README.md) for overview and all example links.
