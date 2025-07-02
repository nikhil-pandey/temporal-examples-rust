//! Client that starts the cancellable workflow and cancels it after 5 seconds.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = get_client().await?;

    info!("Starting run_cancellable_activity workflow execution");

    // Workflow input – none needed but we send empty payload.
    let input = vec![json!("").as_json_payload()?];

    let workflow_id = format!("cancel-hb-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "activities-cancellation-heartbeating".to_string(),
            workflow_id.clone(),
            "run_cancellable_activity".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let run_id = start_res.run_id.clone();

    // Wait 5 seconds then cancel.
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    info!("Requesting workflow cancellation …");
    client
        .cancel_workflow_execution(workflow_id.clone(), Some(run_id), "demo".to_string(), None)
        .await?;

    println!("Cancellation request sent to workflow {workflow_id}");

    Ok(())
}
