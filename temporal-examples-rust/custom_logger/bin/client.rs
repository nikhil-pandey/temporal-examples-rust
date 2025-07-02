//! Client for the custom_logger example: starts workflow with user message, prints result.
use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;
use temporal_client::{WorkflowOptions, WfClientExt, WorkflowClientTrait};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let msg = std::env::args().nth(1).unwrap_or_else(|| "Hello from Rust!".to_string());
    let client = get_client().await?;
    info!(target: "custom_logger", "Client starting logging_workflow with message: {msg}");

    let input = vec![json!(msg.clone()).as_json_payload()?];
    let workflow_id = format!("custom-logger-{}", Uuid::new_v4());
    let start_res = client
        .start_workflow(
            input,
            "custom-logger".to_string(),
            workflow_id.clone(),
            "logging_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;
    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);
    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();
    let first = res_payloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing result payload"))?;
    let result: String = String::from_json_payload(first)?;
    println!("Workflow completed with result: {result}");
    Ok(())
}
