//! Worker binary for the Continue-As-New example.

use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::{
    telemetry::TelemetryOptionsBuilder,
    worker::{WorkerConfigBuilder, WorkerVersioningStrategy},
};

use continue_as_new::workflow::looping_workflow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise env logger.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting continue_as_new worker …");

    // Shared Temporal client.
    let client = get_client().await?;

    // Build runtime & worker config.
    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("continue-as-new")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;

    // Core worker + high-level API.
    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "continue-as-new");

    // Register workflow implementation.
    worker.register_wf("looping_workflow", looping_workflow);

    // Start polling.
    worker.run().await?;

    Ok(())
}
