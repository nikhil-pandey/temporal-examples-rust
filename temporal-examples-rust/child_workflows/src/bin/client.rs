//! Client binary for the **child_workflows** example.
//!
//! It starts the `parent_workflow` on task-queue `child-workflows`, waits for
//! completion, and prints the combined results returned by the two child
//! workflows.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::WfClientExt;
use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise logger so we can see SDK & application logs.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Shared Temporal client pointing at localhost dev-server.
    let client = get_client().await?;

    info!(target: "child_workflows", "Starting parent_workflow execution");

    // The workflow has no meaningful inputs, but we must still send at least
    // one payload – we use an empty JSON string for compatibility with other
    // examples.
    let input = vec![json!("").as_json_payload()?];

    let workflow_id = format!("child-wf-parent-{}", Uuid::new_v4());

    // Kick off the workflow on the `child-workflows` task-queue.
    let start_res = client
        .start_workflow(
            input,
            "child-workflows".to_string(), // task queue
            workflow_id.clone(),
            "parent_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    info!(
        target: "child_workflows",
        "Started parent workflow: id={workflow_id} run_id={}",
        start_res.run_id
    );

    // Wait for workflow completion.
    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);
    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    let first_payload = res_payloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing workflow result payload"))?;

    // Deserialize Vec<String> from the JSON payload.
    let results: Vec<String> = Vec::<String>::from_json_payload(first_payload)?;

    println!("Workflow completed. Combined child results: {results:?}");

    Ok(())
}
