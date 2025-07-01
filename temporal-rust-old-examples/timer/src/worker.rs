use std::sync::Arc;
use temporal_helpers::client::get_client;
use temporal_sdk::Worker;
use temporal_sdk::{WfContext, WorkflowResult};
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use crate::activities;
use crate::workflows;

pub async fn start_worker() -> Result<(), Box<dyn std::error::Error>> {
    let client = get_client().await?;

    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;

    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("timer")
        .versioning_strategy(WorkerVersioningStrategy::None { build_id: "core-worker".to_owned() })
        .build()?;

    let core_worker = init_worker(&runtime, worker_config, client)?;

    let mut worker = Worker::new_from_core(Arc::new(core_worker), "timer");

    worker.register_activity(
        "order_processing_activity",
        activities::order_processing_activity,
    );
    worker.register_activity("send_email_activity", activities::send_email_activity);

    worker.register_wf("sample_timer_workflow", wf_function);

    worker.run().await?;

    Ok(())
}

async fn wf_function(ctx: WfContext) -> WorkflowResult<()> {
    workflows::sample_timer_workflow(ctx, 2).await
}
