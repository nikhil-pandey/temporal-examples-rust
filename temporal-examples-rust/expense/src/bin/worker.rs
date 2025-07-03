//! Worker for expense approval example.
use env_logger::Env;
use expense::activities::{complete_expense, create_expense, http_post};
use expense::workflow::expense_workflow;
use helpers::get_client;
use log::info;
use std::sync::Arc;
use temporal_sdk::Worker;
use temporal_sdk_core::{CoreRuntime, init_worker};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting expense worker ...");
    let client = get_client().await?;
    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;
    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("expense")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;
    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "expense");
    worker.register_activity("create_expense", create_expense);
    worker.register_activity("http_post", http_post);
    worker.register_activity("complete_expense", complete_expense);
    worker.register_wf("expense_workflow", expense_workflow);
    worker.run().await?;
    Ok(())
}
