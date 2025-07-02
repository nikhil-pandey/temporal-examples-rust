//! Worker for the activities_dependency_injection example.
use env_logger::Env;
use helpers::get_client;
use log::info;
use std::sync::Arc;
use temporal_sdk::Worker;
use temporal_sdk_core::{init_worker, CoreRuntime};
use temporal_sdk_core_api::worker::WorkerVersioningStrategy;
use temporal_sdk_core_api::{telemetry::TelemetryOptionsBuilder, worker::WorkerConfigBuilder};

use activities_dependency_injection::activities::*;
use activities_dependency_injection::workflow::multi_greet_workflow;

/// Initialize dependency-injected singletons for activities at binary startup.
fn initialize_greeters() {
    // This is safe only if called exactly once before any activities run.
    // We use lazy static with OnceCell to enforce one-time init.
    // Replace the Lazy with a valid instance by directly dereferencing the OnceCell.
    EN_GREETER.get_or_init(|| std::sync::Arc::new(Greeter::new("en")));
    ES_GREETER.get_or_init(|| std::sync::Arc::new(Greeter::new("es")));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting activities_dependency_injection worker …");

    initialize_greeters();

    let client = get_client().await?;
    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;
    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("activities-dep-inject")
        .versioning_strategy(WorkerVersioningStrategy::None {
            build_id: "rust-sdk".to_owned(),
        })
        .build()?;
    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "activities-dep-inject");

    // Register activities and workflow.
    worker.register_activity("greet_english", greet_english);
    worker.register_activity("greet_spanish", greet_spanish);
    worker.register_wf("multi_greet_workflow", multi_greet_workflow);

    worker.run().await?;
    Ok(())
}
