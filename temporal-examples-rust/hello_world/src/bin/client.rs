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

    // The name to greet can be provided as CLI arg; default to "World".
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "World".to_string());

    // Connect to the Temporal server via helper.
    let client = get_client().await?;

    info!("Starting 'greet' workflow for name={name}");

    // Start workflow execution on task queue `hello-world`.
    // Build payload vec
    let payload = json!(name).as_json_payload()?;
    let input = vec![payload];

    let workflow_id = format!("hello-{}", Uuid::new_v4());

    // Start the workflow execution and capture the server response which includes the `run_id`.
    let start_res = client
        .start_workflow(
            input,
            "hello-world".to_string(),
            workflow_id.clone(),
            "greet".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    // Create a handle so we can easily fetch results.
    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id.clone());

    // Await workflow completion and extract the single string payload.
    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    // Convert first payload to String.
    let greeting_payload = res_payloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing result payload"))?;

    let greeting: String = String::from_json_payload(greeting_payload)?;

    println!("Workflow completed with result: {greeting}");

    Ok(())
}
