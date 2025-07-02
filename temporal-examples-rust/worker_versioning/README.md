# Worker Versioning (Build-ID) Example

This example demonstrates [Build-ID–based worker versioning](https://docs.temporal.io/dev-guide/fault-tolerance#worker-versioning) in Temporal. This is essential for zero-downtime upgrades and decoupling deployment from workflow execution.

---

## What it shows

- **How to run multiple worker binaries, each registering as a different build id** using `WorkerVersioningStrategy` options (`Compatible`/`NewDefaultSet`).
- How incompatible changes can result in a new Build ID Set and isolation of new/existing workflow tasks.
- How a workflow's logic can reflect current build id for demonstration purposes.

---

## How to run

1. **Start the dev server with versioning support** (if not already):
   ```sh
   temporal server start-dev --dynamic-config-value "frontend.enable_worker_versioning=true"
   ```
   *(If you omit the flag, versioning features may be disabled. This flag is usually on by default in >=1.22)*

2. **Start a worker as build id v1**:
   ```sh
   cargo run --bin worker --release -- --build-id v1 --strategy compatible
   # Options:
   #   --build-id <YOUR_BUILD_ID>
   #   --strategy <compatible|incompatible|new-default-set>
   #   (also set BUILD_ID/STRATEGY env vars if desired)
   ```

3. **Start a client to launch the workflow**:
   ```sh
   cargo run --bin client --release
   # Should print: Hello from worker Build ID: v1!
   ```

4. **Stop v1 worker. Update the workflow logic** (e.g., change the greeting string in `workflow.rs`, or use --build-id v2), then start a new v2 worker with incompatible strategy:
   ```sh
   cargo run --bin worker --release -- --build-id v2 --strategy incompatible
   ```

   - Now, existing workflow runs will only execute on the v1 worker; new runs go to v2.
   - Try running a client again for a new workflow.

---

## File structure

- `src/workflow.rs` – Demo workflow returning greeting with build id
- `src/bin/worker.rs` – Selects build id & versioning strategy from CLI/env
- `src/bin/client.rs` – Starts the workflow, prints result

---

## See also
- [TypeScript sample](https://github.com/temporalio/samples-typescript/tree/main/worker-versioning)
- [Temporal docs: Worker Versioning](https://docs.temporal.io/dev-guide/fault-tolerance#worker-versioning)

---

For more, see the main README and parent workspace.
