use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use hello_world::workflow::greet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the env_logger so we can see output.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting hello_world worker...");

    // Create an SDK client to communicate with the Temporal server.
    let client = get_client().await?;

    // Set up telemetry and runtime for the core worker.
    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    // Configure worker: namespace `default`, task queue `hello-world`.
    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("hello-world")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;

    // Initialize core worker and wrap it with the high-level Worker API.
    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "hello-world");

    // Register workflow(s).
    worker.register_wf("greet", greet);

    // Run the worker. This will block until shutdown signal.
    worker.run().await?;

    Ok(())
}
