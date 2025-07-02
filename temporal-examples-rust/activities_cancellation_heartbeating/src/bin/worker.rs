//! Worker for the cancellation & heartbeating example.

use std::sync::Arc;

use env_logger::Env;
use helpers::get_client;
use log::info;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use activities_cancellation_heartbeating::activities::{
    cleanup_activity, fake_progress_activity, skipped_activity,
};
use activities_cancellation_heartbeating::workflow::run_cancellable_activity;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting activities_cancellation_heartbeating worker …");

    let client = get_client().await?;

    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("activities-cancellation-heartbeating")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;

    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(
        Arc::new(core_worker),
        "activities-cancellation-heartbeating",
    );

    // Register activities and workflow.
    worker.register_activity("fake_progress_activity", fake_progress_activity);
    worker.register_activity("skipped_activity", skipped_activity);
    worker.register_activity("cleanup_activity", cleanup_activity);

    worker.register_wf("run_cancellable_activity", run_cancellable_activity);

    worker.run().await?;

    Ok(())
}
