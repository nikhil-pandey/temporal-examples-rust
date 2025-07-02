//! Client that starts the Build-ID versioning workflow and prints result.
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
    // Use build id from env/CLI (default v1)
    let build_id = std::env::var("BUILD_ID").unwrap_or_else(|_| "v1".to_string());

    info!(target: "worker_versioning", "Starting versioned_greeting_workflow for build_id={build_id}");

    // Pass the build id as input so workflow can embed in output
    let input = vec![json!(build_id.clone()).as_json_payload()?];

    let workflow_id = format!("worker-versioning-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "worker-versioning".to_string(),
            workflow_id.clone(),
            "versioned_greeting_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);

    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    let greeting_payload = res_payloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing result payload"))?;

    let greeting: String = String::from_json_payload(greeting_payload)?;

    println!("Workflow completed with result: {greeting}");

    Ok(())
}
