//! Worker for the sleep-for-days example.
use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{CoreRuntime, init_worker};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use sleep_for_days::activities::send_email;
use sleep_for_days::workflow::sleep_for_days_workflow;

const TASK_QUEUE: &str = "sleep-for-days";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!(target: "sleep_for_days", "Starting sleep_for_days worker …");

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

    worker.register_activity("send_email", send_email);
    worker.register_wf("sleep_for_days_workflow", sleep_for_days_workflow);

    worker.run().await?;
    Ok(())
}
