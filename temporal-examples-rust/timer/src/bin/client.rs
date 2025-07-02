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

    // Parse optional threshold argument.
    let threshold_arg = std::env::args()
        .nth(1)
        .map(|v| v.parse::<u64>().expect("threshold must be u64"))
        .unwrap_or(2);

    let client = get_client().await?;

    info!("Starting timer workflow with threshold={threshold_arg}s");

    let input = vec![json!(threshold_arg).as_json_payload()?];

    let workflow_id = format!("timer-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "timer".to_string(),
            workflow_id.clone(),
            "sample_timer_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);

    // Wait for completion.
    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    // We don't care about actual result but print confirmation.
    if res_payloads.is_empty() {
        println!("Workflow completed");
    } else {
        let payload = res_payloads.first().unwrap();
        let _unit: () = <()>::from_json_payload(payload)?;
        println!("Workflow completed (unit result)");
    }

    Ok(())
}
