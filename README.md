# Temporal Rust Examples Workspace

⚠️ **Warning: This repository is fully AI-generated.** ⚠️

## Project purpose

This repository is a modern, curated set of examples for using Temporal with Rust.
It aims to provide Rust developers with easy-to-follow, working sample projects
demonstrating core concepts and best practices—closely modeled after the official
[TypeScript samples](https://github.com/temporalio/samples-typescript),
with rough feature parity.
Whether you are learning Temporal from scratch or porting workflows from TypeScript,
this repo is your starting point.

## Prerequisites

* **Rust nightly toolchain** ([see rust-toolchain.toml](./rust-toolchain.toml))
* **Temporal server** running locally.
    - Start one with: `temporal server start-dev`
      (see [`temporal` CLI](https://docs.temporal.io/toolbox/cli))
* **`temporal` CLI**
  (install from [releases](https://github.com/temporalio/cli)
  or with Homebrew: `brew install temporal`)


## How to run an example

Each example is an independent [Cargo workspace](./Cargo.toml) member.
The general workflow for running them:

1. **Start the Temporal dev server** (once):
   ```
   temporal server start-dev
   ```
2. **Open one terminal per worker:** In the chosen example directory, run:
   ```
   cargo run --bin worker
   ```
3. **Open a client terminal:** In the same directory, run:
   ```
   cargo run --bin client
   ```
4. Inspect results via `stdout` or the Temporal Web UI at
   [http://localhost:8233](http://localhost:8233).

Each crate in `temporal-examples-rust/` contains its own README with custom instructions.

---

## Example Matrix: Rust Crates and TypeScript Parity

| Rust Example Crate                     | Description                                              | TypeScript Sample                    |
|----------------------------------------|----------------------------------------------------------|--------------------------------------|
| [hello_world](temporal-examples-rust/hello_world)                    | Minimal "Hello, World!" workflow                            | [hello-world](https://github.com/temporalio/samples-typescript/tree/main/hello-world) |
| [timer](temporal-examples-rust/timer)                                  | Timers, concurrent activities, threshold timeout/notification | [timer](https://github.com/temporalio/samples-typescript/tree/main/timer)            |
| [http_request](temporal-examples-rust/http_request)                    | Activity does HTTP request, returns response to workflow      | [http](https://github.com/temporalio/samples-typescript/tree/main/http)              |
| [async_activity_completion](temporal-examples-rust/async_activity_completion) | Activity async-completes out-of-band using client      | [async-activity-completion](https://github.com/temporalio/samples-typescript/tree/main/async-activity-completion) |
| [child_workflows](temporal-examples-rust/child_workflows)              | Parent spawning two child workflows, collecting results       | [child-workflows](https://github.com/temporalio/samples-typescript/tree/main/child-workflows) |
| [continue_as_new](temporal-examples-rust/continue_as_new)              | Looping workflow with Continue-As-New for iteration cap       | [continue-as-new](https://github.com/temporalio/samples-typescript/tree/main/continue-as-new) |
| [activities_cancellation_heartbeating](temporal-examples-rust/activities_cancellation_heartbeating) | Activity heartbeats, cancellation, cleanup, skip             | [cancellation](https://github.com/temporalio/samples-typescript/tree/main/cancellation) |
| [signals_unblock](temporal-examples-rust/signals_unblock)              | Signals used to unblock waiting work in workflow              | [signals](https://github.com/temporalio/samples-typescript/tree/main/signals)        |
| [message_passing](temporal-examples-rust/message_passing)              | Workflow-to-workflow message passing via signals              | [message-passing](https://github.com/temporalio/samples-typescript/tree/main/message-passing) |
| [cron_workflows](temporal-examples-rust/cron_workflows)                | Workflow scheduled via cron, counts each run                  | [cron-workflows](https://github.com/temporalio/samples-typescript/tree/main/cron-workflows) |
| [mutex](temporal-examples-rust/mutex)                                  | Distributed mutex with signals                                | [mutex](https://github.com/temporalio/samples-typescript/tree/main/mutex)           |
| [saga](temporal-examples-rust/saga)                                    | Saga pattern (compensating transactions/orchestration)         | [saga](https://github.com/temporalio/samples-typescript/tree/main/saga)             |
| [search_attributes](temporal-examples-rust/search_attributes)          | Set and query custom Search Attributes                         | [search-attributes](https://github.com/temporalio/samples-typescript/tree/main/search-attributes) |
| [worker_versioning](temporal-examples-rust/worker_versioning)          | Build-ID based worker versioning demo                          | [worker-versioning](https://github.com/temporalio/samples-typescript/tree/main/worker-versioning) |
| signals_queries | Signals, queries, and cancellation via query-registration | [signals-queries](https://github.com/temporalio/samples-typescript/tree/main/signals-queries) |
| [activities_dependency_injection](temporal-examples-rust/activities_dependency_injection) | Activity dependency-injection pattern (DI via statics, English/Spanish greetings) | [activities-dependency-injection](https://github.com/temporalio/samples-typescript/tree/main/activities-dependency-injection) |
| [worker_specific_task_queues](temporal-examples-rust/worker_specific_task_queues) | Run activities on different task queues with distinct workers | [worker-specific-task-queues](https://github.com/temporalio/samples-typescript/tree/main/worker-specific-task-queues) |
| [sleep_for_days](temporal-examples-rust/sleep_for_days) | Long-running workflow sleeping & sending periodic email, completes by signal | [sleep-for-days](https://github.com/temporalio/samples-typescript/tree/main/sleep-for-days) |
| [custom_logger](temporal-examples-rust/custom_logger) | Custom logger (env_logger to stdout/file, log from workflow/activity) | [custom-logger](https://github.com/temporalio/samples-typescript/tree/main/custom-logger) |
| [expense](temporal-examples-rust/expense) | Expense approval workflow (signals approve/reject with timeout) | [expense](https://github.com/temporalio/samples-typescript/tree/main/expense) |
| [grpc_calls](temporal-examples-rust/grpc_calls) | Raw gRPC calls demo (list namespaces / executions) | [grpc-calls](https://github.com/temporalio/samples-typescript/tree/main/grpc-calls) |

---

## Not yet supported in Rust SDK

Some TypeScript samples rely on server or SDK features not yet available in the official Rust SDK. The following use cases are **not yet portable to Rust:**

| TypeScript Sample                  | Notes (why unsupported in Rust)                                                        |
|------------------------------------|--------------------------------------------------------------------------------------|
| [encryption](https://github.com/temporalio/samples-typescript/tree/main/encryption)   | Workflow/Activity interception (e.g. data encryption) not implemented as of writing. |
| [schedule](https://github.com/temporalio/samples-typescript/tree/main/schedule)         | Schedule API not yet exposed in Rust SDK (can use gRPC as workaround).               |
| Sinks (any)                        | [Sinks](https://docs.temporal.io/dev-guide/go/sinks) not implemented in Rust SDK.     |
| Web UI hooks                       | Temporal Web UI server hooks (custom view, interception, etc.) unsupported.           |

If you discover missing functionality, please check the [Rust SDK progress](https://github.com/temporalio/sdk-core/issues) or open an issue here.

---

## Contributing & Feedback

Contributions, issue reports, and feature parity PRs are all welcome! See each crate's README for details on running and experimenting with the examples.
