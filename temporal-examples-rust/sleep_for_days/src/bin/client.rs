//! Client to run the sleep-for-days workflow and signal completion after two intervals.
use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

const TASK_QUEUE: &str = "sleep-for-days";
const COMPLETE_SIGNAL: &str = "complete";

/// Starts workflow with given interval, signals 'complete' after 2x interval.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Get interval_secs (CLI arg or default 10)
    let interval_secs = std::env::args()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10);

    let client = get_client().await?;

    info!(target: "sleep_for_days", "Client: starting workflow with interval_secs={interval_secs}");

    let input = vec![json!(interval_secs).as_json_payload()?];
    let workflow_id = format!("sleep-for-days-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            TASK_QUEUE.to_string(),
            workflow_id.clone(),
            "sleep_for_days_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    info!(target: "sleep_for_days", "Workflow started (id={workflow_id} run_id={})", start_res.run_id);

    // Sleep for 2 intervals then send complete signal
    let total = interval_secs * 2;
    info!(target: "sleep_for_days", "Client sleeping for {total}s then will signal completion");
    tokio::time::sleep(std::time::Duration::from_secs(total)).await;

    info!(target: "sleep_for_days", "Signaling 'complete' …");
    client
        .signal_workflow_execution(
            workflow_id.clone(),
            start_res.run_id.clone(),
            COMPLETE_SIGNAL.into(),
            None,
            None,
        )
        .await?;

    // Wait for workflow to complete
    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);
    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    let result_payload = res_payloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing result payload"))?;
    let result: String = String::from_json_payload(result_payload)?;
    println!("Workflow exited: {result}");
    Ok(())
}
use temporal_client::WfClientExt;
