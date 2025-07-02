# Sleep-for-days Example

Demonstrates a long-running workflow that sleeps in intervals and sends a periodic email, exiting when it receives a `complete` signal. Closely mirrors the [TypeScript sleep-for-days sample](https://github.com/temporalio/samples-typescript/tree/main/sleep-for-days).

## Pattern
- Workflow loops: sleeps for interval, calls an email activity, repeats until `complete` signal is received.
- Uses timers and signals as core Rust SDK features.

## Files

- `src/activities.rs`: One fake email activity: logs a sent message.
- `src/workflow.rs`: Main workflow: accepts `interval_secs: u64` (default 30 days), loops, listening for a `complete` signal.
- `bin/worker.rs`: Worker process (runs on `sleep-for-days` queue, registers workflow/activity).
- `bin/client.rs`: Starts workflow (you can pass interval_secs CLI arg, default 10 sec), then after two intervals sends `complete` signal to finish the workflow.

## How to Run

1. Start the Temporal server:
   ```sh
   temporal server start-dev
   ```
2. In this directory, start the worker:
   ```sh
   cargo run --bin worker
   ```
3. In a second terminal, run the client (optionally pass `interval_secs`, default is 10 seconds):
   ```sh
   cargo run --bin client -- 5
   ```
   The client starts the workflow, lets it run for two intervals, then signals completion and exits.

## What You'll See
- The worker logs an email-sent message after each interval.
- After two intervals, the client sends a complete signal; workflow returns.
- This pattern demonstrates long timers + signals in Temporal Rust.

## TypeScript Mapping

- [sleep-for-days sample (TypeScript)](https://github.com/temporalio/samples-typescript/tree/main/sleep-for-days)
- This Rust version uses the same interval/signal pattern as in the TS sample.

---
See parent [README](../../README.md) for all examples and prerequisites.
