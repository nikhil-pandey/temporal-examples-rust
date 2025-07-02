//! Worker A: runs on queue-a, registers activity & workflow.
use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use worker_specific_task_queues::activities::greet_from_queue;
use worker_specific_task_queues::workflow::multi_queue_workflow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!(target: "worker_specific_task_queues", "Starting worker A (queue-a)");

    let client = get_client().await?;

    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("queue-a")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;

    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "queue-a");

    // Register greet activity (can also register workflow for illustration)
    worker.register_activity("greet_from_queue", greet_from_queue);
    worker.register_wf("multi_queue_workflow", multi_queue_workflow);

    worker.run().await?;

    Ok(())
}
