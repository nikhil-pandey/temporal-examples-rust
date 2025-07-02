# Custom Logger Example

Shows how to configure a custom logger for Rust Temporal Workers, so that workflow/activity logs can be written both to a console and a file, with custom formatting. Closely mirrors the [TypeScript custom-logger sample](https://github.com/temporalio/samples-typescript/tree/main/custom-logger).

## What it demonstrates
- Registering a custom logger (using `env_logger` with Rust's `log` crate) that logs to both stdout and a file with a custom JSON-per-line output for file.
- Workflow and activity that write application log messages.
- How to pass workflow arguments via client and see their logging in the worker and output file.

## How to run
1. Start Temporal server:
   ```sh
   temporal server start-dev
   ```
2. In this directory, start the worker (**will log to `worker.log` and console**):
   ```sh
   cargo run --bin worker
   ```
3. In another terminal, run the client (optionally: pass a message string):
   ```sh
   cargo run --bin client -- "Hello CustomLogger!"
   ```

## Output
- Worker console prints all logs (info+).
- `worker.log` contains JSON records of all log events with workflow/activity context.

## Parity with TypeScript
- [TypeScript example: custom-logger](https://github.com/temporalio/samples-typescript/tree/main/custom-logger)
- Rust uses `env_logger` and layers a custom file drain in addition to stdout.

---
See [../../README.md](../../README.md) for full context and workspace.
