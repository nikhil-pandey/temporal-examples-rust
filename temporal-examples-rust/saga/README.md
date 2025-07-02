# Saga (Travel Booking) Example

Implements the Saga (compensation) pattern: sequential booking activities (flight/hotel/car), with rollback upon failure.

Mirrors the [TypeScript Saga sample](https://github.com/temporalio/samples-typescript/tree/main/saga), adapted for the Rust SDK.

## How to run

1. Start the Temporal dev server (if not already):
   ```
   temporal server start-dev
   ```
2. In one terminal, start the worker:
   ```
   cargo run --bin worker
   ```
3. In another terminal, start the client:
   ```
   cargo run --bin client
   ```
4. About half the runs will simulate a reservation failure and thus demonstrate compensation (cancelling already reserved steps), visible in the worker/client logs.

See repo root [README](../../README.md) for general info.

