use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{CoreRuntime, init_worker};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use http_request::{activities::make_http_request, workflow::http_workflow};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise logging.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting http_request worker ...");

    // Shared Temporal client.
    let client = get_client().await?;

    // Build runtime & worker config.
    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("http-request")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;

    // Create core worker & high-level worker wrapper.
    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "http-request");

    // Register workflow & activity.
    worker.register_activity("make_http_request", make_http_request);
    worker.register_wf("http_workflow", http_workflow);

    // Run until shutdown.
    worker.run().await?;

    Ok(())
}
