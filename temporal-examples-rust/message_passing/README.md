# Message Passing Example (Workflow-to-Workflow Signals)

This example demonstrates workflow-to-workflow message passing using Temporal signals (TypeScript parity: [message-passing](https://github.com/temporalio/samples-typescript/tree/main/message-passing)).

## Pattern

- **Consumer workflow**: Accepts string messages via `add_message` signals. After receiving all messages, receives a final `done` signal and completes with the received list.
- **Producer workflow**: Receives a consumer workflow id and a vector of messages, sends each as a signal, then signals `done`.
- **No activities are used**—all messaging is workflow-to-workflow via Temporal signals.

## How to run

1. Start Temporal server:
   ```bash
   temporal server start-dev
   ```
2. In this directory, start the worker:
   ```bash
   cargo run --bin worker
   ```
3. In another terminal, run the client:
   ```bash
   cargo run --bin client -- Foo,Bar,Baz
   ```
   - You may specify a comma-separated message list. Default: `Hello,world,from,producer`

4. The consumer should output the list collected from the producer’s signals!

## TypeScript parity

See [samples-typescript/message-passing](https://github.com/temporalio/samples-typescript/tree/main/message-passing)

## Details
- Task queue: `message-passing`
- Workflows exported: `consumer_workflow`, `producer_workflow`
- No activities (signal-only, demonstrates inter-workflow comms).

## See Also
- [See workspace README and matrix for more examples …](../../README.md)
