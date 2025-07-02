Expense Approval Example
========================

This example demonstrates an expense approval workflow using Rust and Temporal.
Mirrors the [TypeScript expense sample](https://github.com/temporalio/samples-typescript/tree/main/expense)
where an expense request can be approved, rejected, or auto-timed-out by signal.

Features
--------
- **Activities:** Create an expense, log/send to HTTP server (dummy/log), mark as complete.
- **Workflow:** Wait for signals to approve/reject with timeout; signals can be sent from a client.
- **Signals:** `approve` and `reject` (can be sent by separate clients)
- **Timeout:** If no signal received, auto-marks as `timed_out`
- **Portable:** No external HTTP server required, just logs.

How to Run
----------
1. Start Temporal server:
    temporal server start-dev
2. In one terminal, run the worker:
    cargo run --bin worker
3. In another, start a new expense request:
    cargo run --bin client_start -- 120.0 Alice "Conference hotel"
   Output shows the workflow id (needed for approval/rejection).
4. To approve:
    cargo run --bin client_approve -- <workflow_id>
   To reject:
    cargo run --bin client_reject -- <workflow_id>
5. If you do nothing, the request auto-times-out after 30sec.
6. Output/logs will show workflow status transitions.


File Structure
--------------
- `src/activities.rs`: Activity functions and `Expense` struct
- `src/workflow.rs`: Workflow logic (selects on signals)
- `src/bin/worker.rs`: Runs the worker (task queue: "expense")
- `src/bin/client_start.rs`: Starts a workflow instance with CLI args
- `src/bin/client_approve.rs`, `client_reject.rs`: Send signals

Limits/Notes
------------
- No external HTTP server; endpoint is logged.
- All activity and workflow status is logged.
- Can test with multiple simultaneous runs.

TypeScript parity/sample: [expense](https://github.com/temporalio/samples-typescript/tree/main/expense)

See [../../README.md](../../README.md) for workspace setup and all Rust examples.
