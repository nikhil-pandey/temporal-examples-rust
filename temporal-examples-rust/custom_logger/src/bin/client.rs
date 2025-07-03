//! Client for the `custom_logger` example. Starts `logging_workflow` and prints result.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::{WfClientExt, WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use uuid::Uuid;

const TASK_QUEUE: &str = "custom-logger";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // The worker installs the combined logger. For the *client* process we can
    // get away with a simple env_logger so we see output. Not strictly
    // required but useful when running the sample.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let msg = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Hello from client".to_string());

    let client = get_client().await?;

    info!(target: "custom_logger", "Client: starting logging_workflow msg='{msg}'");

    let input = vec![json!(msg.clone()).as_json_payload()?];

    let workflow_id = format!("custom-logger-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            TASK_QUEUE.to_string(),
            workflow_id.clone(),
            "logging_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);

    let payloads: Vec<temporal_sdk_core_protos::temporal::api::common::v1::Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    let first = payloads.first().expect("missing result payload");
    let out: String = String::from_json_payload(first)?;
    println!("Workflow completed with result: {out}");

    Ok(())
}
