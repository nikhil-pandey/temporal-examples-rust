# Mutex Workflow Example

This sample demonstrates an async mutex/lock coordinated by a Temporal workflow. Multiple clients (processes) request a lock by sending a signal; the workflow serializes access by queueing requests, so only one process holds the lock at a time. When the current holder completes work, it signals release and the lock is granted to the next queue entry.

## How to run

1. Start Temporal dev server:
   ```
   temporal server start-dev
   ```
2. In **one** terminal, start the worker (keeps polling):
   ```bash
   cargo run --bin worker
   ```

3. In **another** terminal, launch the lock coordinator for a lock id (defaults to `mutex-lock`). **Run this once**:
   ```bash
   cargo run --bin lock_starter           # or: cargo run --bin lock_starter my-lock-id
   ```

4. Finally, open **two additional terminals** and run the client twice:
   ```bash
   # terminal A
   cargo run --bin client                 # uses default lock id

   # terminal B (start a moment later)
   cargo run --bin client                 # second contender for the same lock
   ```

   The first client will acquire the lock immediately, hold it for ~5 seconds, then release. The second client waits until the lock is released before proceeding – demonstrating serialized access.

5. Observe the worker logs – you should **not** see the previous “Requesting lock” spam. Instead, each workflow requests once, then the coordinator grants and waits for release.

## Details

- Task Queue: `mutex`
- Workflow: `lock_workflow` (waits for signals to queue requests, then signals lock acquisition).
- Client: Spawns a `test_lock_workflow`, requests lock, waits for `lock-acquired` signal, does work, then releases.

See [../..](../../README.md) for Rust vs. TypeScript feature mapping.
