//! Worker binary for the saga example.

use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use saga::{activities::*, workflow::book_trip_workflow};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting saga worker …");
    let client = get_client().await?;
    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;
    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("saga")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;
    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "saga");

    // Register all booking and cancellation activities.
    worker.register_activity("reserve_flight", reserve_flight);
    worker.register_activity("cancel_flight", cancel_flight);
    worker.register_activity("reserve_hotel", reserve_hotel);
    worker.register_activity("cancel_hotel", cancel_hotel);
    worker.register_activity("reserve_car", reserve_car);
    worker.register_activity("cancel_car", cancel_car);
    // Register the saga workflow.
    worker.register_wf("book_trip_workflow", book_trip_workflow);

    worker.run().await?;
    Ok(())
}
