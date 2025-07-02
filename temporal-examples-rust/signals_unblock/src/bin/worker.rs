//! Worker binary for the `signals_unblock` example.

use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use signals_unblock::workflow::signal_await_workflow;

const TASK_QUEUE: &str = "signals-unblock";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting signals_unblock worker …");

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

    // Register workflow implementation.
    worker.register_wf("signal_await_workflow", signal_await_workflow);

    worker.run().await?;

    Ok(())
}
