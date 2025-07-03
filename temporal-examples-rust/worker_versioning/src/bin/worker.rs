//! Worker for the build-id versioning demo. See README for steps.
use env_logger::Env;
use helpers::get_client;
use log::info;
use std::sync::Arc;
use temporal_sdk::Worker;
use temporal_sdk_core::{CoreRuntime, init_worker};
use temporal_sdk_core_api::{
    telemetry::TelemetryOptionsBuilder,
    worker::{
        WorkerConfigBuilder, WorkerDeploymentOptions, WorkerDeploymentVersion,
        WorkerVersioningStrategy,
    },
};
use worker_versioning::workflow::versioned_greeting_workflow;

/// Read build id and strategy from CLI/env, returning tuple (build_id, strategy_name)
/// Parse CLI arguments/environment for build id and strategy.
/// Recognizes:
///   --build-id <id>, --strategy <legacy|deployment|none>, env BUILD_ID, STRATEGY
fn get_build_id_and_strategy() -> (String, String) {
    let mut build_id = std::env::var("BUILD_ID").unwrap_or_else(|_| "v1".to_string());
    let mut strategy = std::env::var("STRATEGY").unwrap_or_else(|_| "legacy".to_string());
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--build-id" => {
                if let Some(val) = args.next() {
                    build_id = val;
                }
            }
            "--strategy" => {
                if let Some(val) = args.next() {
                    strategy = val;
                }
            }
            _ => {}
        }
    }
    (build_id, strategy)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!(target: "worker_versioning", "Starting worker_versioning worker …");
    let client = get_client().await?;
    let telemetry_options = TelemetryOptionsBuilder::default().build()?;
    let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;
    let (build_id, strategy_str) = get_build_id_and_strategy();
    let versioning_strategy = match strategy_str.as_str() {
        "legacy" | "compatible" => WorkerVersioningStrategy::LegacyBuildIdBased {
            build_id: build_id.clone(),
        },
        "deployment" => {
            // Example WorkerDeploymentOptions
            let deployment_version = WorkerDeploymentVersion {
                deployment_name: "demo".to_string(),
                build_id: build_id.clone(),
            };
            WorkerVersioningStrategy::WorkerDeploymentBased(WorkerDeploymentOptions {
                version: deployment_version,
                use_worker_versioning: true,
                default_versioning_behavior: None,
            })
        }
        _ => WorkerVersioningStrategy::None {
            build_id: build_id.clone(),
        },
    };
    info!(target: "worker_versioning", "Using build_id={build_id} strategy={strategy_str}");
    let worker_config = WorkerConfigBuilder::default()
        .namespace("default")
        .task_queue("worker-versioning")
        .versioning_strategy(versioning_strategy)
        .build()?;
    let core_worker = init_worker(&runtime, worker_config, client)?;
    let mut worker = Worker::new_from_core(Arc::new(core_worker), "worker-versioning");
    worker.register_wf("versioned_greeting_workflow", versioned_greeting_workflow);
    worker.run().await?;
    Ok(())
}
