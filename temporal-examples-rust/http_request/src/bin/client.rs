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

    // Shared Temporal client.
    let client = get_client().await?;

    info!("Starting http_workflow execution");

    // We don't require inputs for this workflow but pass an empty JSON string to
    // keep the API happy.
    let input = vec![json!("").as_json_payload()?];

    let workflow_id = format!("http-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "http-request".to_string(),
            workflow_id.clone(),
            "http_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id.clone());

    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    let answer_payload = res_payloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing result payload"))?;

    let answer: String = String::from_json_payload(answer_payload)?;

    println!("Workflow completed with result: {answer}");

    Ok(())
}
