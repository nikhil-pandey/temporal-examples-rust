# Worker-Specific Task Queues Example

This example demonstrates routing Temporal Activities to different Task Queues by running separate worker processes registering different activities on each queue. The workflow dispatches activities to both queues and collects their results. This matches the [worker-specific-task-queues TypeScript sample](https://github.com/temporalio/samples-typescript/tree/main/worker-specific-task-queues).

## What it shows

* How to run two separate worker binaries on different task queues
* How the same workflow can call activities on different queues using `ActivityOptions.task_queue`
* Clean, explicit routing for multi-component services

## Layout

    src/
      activities.rs         -- both activities (param: queue name)
      workflow.rs           -- workflow calls greet ("queue-a"), then greet ("queue-b")
    bin/worker_a.rs        -- registers to "queue-a"
    bin/worker_b.rs        -- registers to "queue-b"
    bin/client.rs          -- starts workflow and prints greetings

## How to run

1. Start Temporal dev server:
   ```sh
   temporal server start-dev
   ```
2. In one terminal:
   ```sh
   cd temporal-examples-rust/worker_specific_task_queues
   cargo run --bin worker_a
   ```
3. In a second terminal:
   ```sh
   cargo run --bin worker_b
   ```
4. In a third terminal:
   ```sh
   cargo run --bin client
   ```

You should see both "Hello from queue-a" and "Hello from queue-b" printed.

## Mapping to TypeScript

TypeScript parity: [worker-specific-task-queues sample](https://github.com/temporalio/samples-typescript/tree/main/worker-specific-task-queues)

---
See main [README](../../README.md) for full index and prerequisites.
