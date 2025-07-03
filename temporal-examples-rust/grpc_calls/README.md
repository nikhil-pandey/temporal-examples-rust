# Raw gRPC Calls Example (Rust)

This example shows how to perform **raw gRPC calls** against the Temporal
Frontend service using the generated gRPC/protobuf stubs that ship with the
Rust SDK.  
It mirrors the [TypeScript `grpc-calls` sample](https://github.com/temporalio/samples-typescript/tree/main/grpc-calls).

Instead of running a workflow/worker pair, the example simply **lists all
namespaces** configured on the connected Temporal cluster by calling the
`ListNamespaces` RPC directly.

## What it demonstrates

* How to obtain a channel (`tonic::transport::Channel`) to the Temporal
  Frontend (default `http://localhost:7233`).
* How to use the generated `temporal.api.workflowservice.v1.WorkflowServiceClient`
  stub (re-exported by `temporal_sdk_core_protos`) to invoke RPCs.
* Differences between using the high-level Rust SDK vs. the low-level gRPC
  surface.

> **Note**: You do **not** need to run a worker for this sample – it is *pure
> client code*.

## How to run

1. Ensure the Temporal dev server is running:

   ```bash
   temporal server start-dev
   ```

2. Run the example client (this crate has no worker binary):

   ```bash
   cargo run --bin client
   ```

   You should see output similar to:

   ```text
   Namespaces returned by ListNamespaces:
   - default (state: REGISTERED)
   ```

That's it! Feel free to experiment with other RPCs by editing
`src/bin/client.rs`.

## Mapping to TypeScript sample

The TypeScript repo calls `ListNamespaces` and `DescribeNamespace`. Those RPCs
are available in the Rust stubs under the same names – the request/response
types are generated from the protobuf definitions.

For full API surface, inspect the generated module:

```rust
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::workflow_service_client::WorkflowServiceClient;
```

---

See the [root README](../../README.md) for the workspace overview and more
examples.
