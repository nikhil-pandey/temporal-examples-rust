//! Worker for the `custom_logger` example.

use std::sync::Arc;

use custom_logger::logger;
use custom_logger::{activities::log_activity, workflow::logging_workflow};
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{CoreRuntime, init_worker};
use temporal_sdk_core_api::{
    telemetry::TelemetryOptionsBuilder,
    worker::{WorkerConfigBuilder, WorkerVersioningStrategy},
};

const TASK_QUEUE: &str = "custom-logger";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install custom logger (stdout + worker.log)
    logger::init();

    info!(target: "custom_logger", "Starting custom_logger worker …");

    let client = get_client().await?;

    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue(TASK_QUEUE)
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;

    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), TASK_QUEUE);

    // Register activity & workflow.
    worker.register_activity("log_activity", log_activity);
    worker.register_wf("logging_workflow", logging_workflow);

    worker.run().await?;

    Ok(())
}
