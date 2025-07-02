# Activities Cancellation & Heartbeating Example

Shows how to heartbeat long-running Activities, handle cancellation, perform cleanup logic post-cancellation, and skip activities as needed.

## How to run

1. Start Temporal server:
   ```
   temporal server start-dev
   ```
2. Start the worker for this crate:
   ```
   cargo run --bin worker
   ```
3. In another terminal, run the client:
   ```
   cargo run --bin client
   ```
4. To observe cleanup/cancellation/heartbeat resumption, cancel/interrupt the workflow execution partway and restart.

Detailed explanation and all samples: [../..](../../README.md).
