# HTTP Request Example

This example shows how to invoke an Activity that makes an outbound HTTP request and use its response in a Workflow.

## How to run

1. Start Temporal server:
   ```
   temporal server start-dev
   ```
2. In this directory, start the worker:
   ```
   cargo run --bin worker
   ```
3. In another terminal, start the client:
   ```
   cargo run --bin client
   ```
4. The result will contain a UUID echoed from `httpbin.org`.

Learn more in the [workspace README](../../README.md).
