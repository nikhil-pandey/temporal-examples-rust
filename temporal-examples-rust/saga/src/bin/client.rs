//! Client binary that starts the saga example workflow and prints outcome.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;
use temporal_client::WfClientExt;
use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = get_client().await?;
    info!("Starting book_trip_workflow execution");

    let input = vec![json!("").as_json_payload()?];
    let workflow_id = format!("saga-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "saga".to_string(),
            workflow_id.clone(),
            "book_trip_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);
    let payloads = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    if payloads.is_empty() {
        println!("Workflow completed (no result)");
    } else {
        let p = payloads.first().unwrap();
        // Try to decode output as JSON string
        let result: String = String::from_json_payload(p)?;
        println!("Workflow completed: {result}");
    }
    Ok(())
}
