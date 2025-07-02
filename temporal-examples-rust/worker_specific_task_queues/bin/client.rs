//! Client for worker-specific-task-queues example. Starts workflow and prints both greetings.
use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::WorkflowOptions;
use temporal_client::{WfClientExt, WorkflowClientTrait};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = get_client().await?;

    info!(target: "worker_specific_task_queues", "Starting multi_queue_workflow execution");

    // This workflow does not require input, just an empty payload.
    let input = vec![json!("").as_json_payload()?];

    let workflow_id = format!("wsq-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "queue-a".to_string(), // Workflow run uses queue-a
            workflow_id.clone(),
            "multi_queue_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);
    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    let payload = res_payloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing result payload"))?;
    let greetings: Vec<String> = Vec::from_json_payload(payload)?;

    println!("Workflow completed greetings: {greetings:?}");
    Ok(())
}
