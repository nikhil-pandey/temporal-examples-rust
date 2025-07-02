//! Client binary that starts the workflow, waits three seconds, and sends a signal.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::{WfClientExt, WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

use signals_unblock::workflow::UNBLOCK_SIGNAL;

const TASK_QUEUE: &str = "signals-unblock";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = get_client().await?;

    info!("Starting signal_await_workflow execution");

    // No meaningful input but Temporal requires at least one payload. Use empty JSON string.
    let input = vec![json!("").as_json_payload()?];

    let workflow_id = format!("signals-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            TASK_QUEUE.to_string(),
            workflow_id.clone(),
            "signal_await_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    // Send the `unblock` signal after a short delay.
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    info!("Sending '{UNBLOCK_SIGNAL}' signal …");

    client
        .signal_workflow_execution(
            workflow_id.clone(),
            start_res.run_id.clone(),
            UNBLOCK_SIGNAL.to_string(),
            None,
            None,
        )
        .await?;

    // Await workflow completion & print result.
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
