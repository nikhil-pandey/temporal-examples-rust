use std::sync::Arc;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use crate::activities;
use crate::workflows;

pub async fn start_worker() -> Result<(), Box<dyn std::error::Error>> {
    let client = temporal_helpers::client::get_client().await?;

    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("activities-cancellation-heartbeating")
        .versioning_strategy(WorkerVersioningStrategy::None { build_id: "rust-sdk".to_owned() })
        .build()?;

    let core_worker = init_worker(&runtime, worker_config, client)?;

    let mut worker = Worker::new_from_core(
        Arc::new(core_worker),
        "activities-cancellation-heartbeating",
    );

    worker.register_activity("fake_progress_activity", activities::fake_progress_activity);
    worker.register_activity("skipped_activity", activities::skipped_activity);
    worker.register_activity("cleanup_activity", activities::cleanup_activity);

    worker.register_wf(
        "run_cancellable_activity",
        workflows::run_cancellable_activity,
    );

    worker.run().await?;

    Ok(())
}
