//! Client binary for the async_activity_completion example.

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

    info!("Starting async_activity_workflow execution");

    // Pass an empty JSON payload – workflow doesn't require inputs.
    let input = vec![json!("").as_json_payload()?];

    let workflow_id = format!("async-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "async-activity-completion".to_string(),
            workflow_id.clone(),
            "async_activity_workflow".to_string(),
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

    let value: String = String::from_json_payload(payload)?;

    println!("Workflow completed with result: {value}");

    Ok(())
}
