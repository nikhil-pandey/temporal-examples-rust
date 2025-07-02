//! Worker for the mutex example: registers lock and test workflows on "mutex" queue.

use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use mutex::workflow::{lock_workflow, test_lock_workflow};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!(target: "mutex", "Starting mutex worker");

    let client = get_client().await?;

    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("mutex")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;

    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "mutex");

    worker.register_wf("lock_workflow", lock_workflow);
    worker.register_wf("test_lock_workflow", test_lock_workflow);

    worker.run().await?;

    Ok(())
}
