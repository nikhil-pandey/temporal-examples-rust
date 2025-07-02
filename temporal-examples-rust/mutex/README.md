# Mutex Workflow Example

This sample demonstrates an async mutex/lock coordinated by a Temporal workflow. Multiple clients (processes) request a lock by sending a signal; the workflow serializes access by queueing requests, so only one process holds the lock at a time. When the current holder completes work, it signals release and the lock is granted to the next queue entry.

## How to run

1. Start Temporal dev server:
   ```
   temporal server start-dev
   ```
2. In this crate directory, start the worker:
   ```
   cargo run --bin worker
   ```
3. In two other terminals, run the client (with a lock id argument, e.g. `my-lock-id`):
   ```
   cargo run --bin client my-lock-id
   ```
   Run the above command in two different terminals at the same time. The second will not finish until the first releases the lock (after about 5 seconds).

4. Observe logs and output for lock serialization.

## Details

- Task Queue: `mutex`
- Workflow: `lock_workflow` (waits for signals to queue requests, then signals lock acquisition).
- Client: Spawns a `test_lock_workflow`, requests lock, waits for `lock-acquired` signal, does work, then releases.

See [../..](../../README.md) for Rust vs. TypeScript feature mapping.
