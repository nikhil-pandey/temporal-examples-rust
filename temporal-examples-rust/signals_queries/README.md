# Signals & Queries Example


This crate demonstrates handling signals to unblock state in a workflow.


## Pattern

- The workflow is initially `blocked=true`
- An `unblock` signal can flip `blocked` to false, unblocking the workflow
- The workflow completes as soon as it is unblocked (via signal) or cancelled (result string: "unblocked" or "cancelled")
- **Query registration (`blocked` query)**: NOT SUPPORTED in the Rust SDK (see below)

## How to run

1. Start Temporal dev server (once):
   ```
   temporal server start-dev
   ```
2. In this directory, start the worker:
   ```
   cargo run --bin worker
   ```

3. Open four additional terminals to run the clients:
   ```
   # Start the workflow (pass workflow id as argument):
   cargo run --bin start_workflow -- my-workflow-id

   # Querying the workflow state is **NOT YET SUPPORTED in Rust** (see note below)

   # Send an unblock signal:
   cargo run --bin signal_workflow -- my-workflow-id

   # Cancel the workflow:
   cargo run --bin cancel_workflow -- my-workflow-id
   ```

Observe worker/client stdout for results (and web UI for status).

---


### ⚠️ Query registration unavailable in Rust SDK

The Temporal Rust SDK does **NOT** currently support `register_query` or `register_query_handler` APIs for workflows. This means that querying workflow state, as demonstrated in the TypeScript example, is **not possible in Rust yet**. If/when the Rust SDK adds this feature, the example should be updated.

---

See [../../README.md](../../README.md) for workspace context and requirements.
