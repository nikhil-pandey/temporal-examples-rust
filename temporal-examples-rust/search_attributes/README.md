# Search Attributes Example (Rust)

This example demonstrates how to use Temporal Search Attributes in a workflow.
The workflow sets a custom search attribute (`CustomerId`) at execution start. The client demonstrates filtering open workflows using this attribute.

## What it does

* **Workflow**: Accepts a string argument (customer ID), upserts the "CustomerId" search attribute, waits 2 seconds (so it will show up in open workflows), then completes.
* **Client**: Starts the workflow, then immediately calls `list_workflows` filtering on `CustomerId`, and prints the results.
* **Worker**: Runs the workflow on the "search-attributes" task queue.

## How to run

**Start Temporal server (if needed):**

```sh
temporal server start-dev
```

**In one terminal, start the worker:**

```sh
cd temporal-examples-rust/search_attributes
cargo run --bin worker
```

**In another terminal, run the client:**

```sh
cargo run --bin client -- CUSTOMER_ID_VALUE
# Or omit CUSTOMER_ID_VALUE for default
```

**Example output:**

```
Started workflow with id=search-attr-436a... run_id=...
Listing open workflows with CustomerId='sample-cust-42':
- WorkflowID: search-attr-436a...  RunID: ...  Status: Some(...)
Workflow completed, result payload count: 1
```

## How to list workflows using the `temporal` CLI

The CLI provides additional insight and parity with other SDKs.

```
temporal workflow list --query "CustomerId='sample-cust-42'"
```

Or use the web UI at http://localhost:8233 and search `CustomerId='sample-cust-42'`

## Notes
- Custom search attributes (like `CustomerId`) may need to be added to the server's Search Attributes schema if running against real Temporal Cloud. This demo assumes Temporal dev server where dynamic attributes are allowed.
- This feature matches the [TypeScript search-attributes sample](https://github.com/temporalio/samples-typescript/tree/main/search-attributes).
- The Rust SDK exposes both `WfContext::upsert_search_attributes` and `WorkflowClient::list_workflows()`.

See main [README](../../README.md) for context and dependencies.
