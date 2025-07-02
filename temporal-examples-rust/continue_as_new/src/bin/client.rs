//! Client binary that starts the Continue-As-New looping workflow.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::{WfClientExt, WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = get_client().await?;

    info!("Starting looping_workflow execution");

    // Initial argument is iteration 0.
    let input = vec![json!(0_u32).as_json_payload()?];

    let workflow_id = format!("loop-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "continue-as-new".to_string(),
            workflow_id.clone(),
            "looping_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    // Fetch handle so we can await completion.
    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);

    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    if res_payloads.is_empty() {
        println!("Workflow completed (no result)");
    } else {
        // Deserialize the unit value just to satisfy clippy.
        let p = res_payloads.first().unwrap();
        let _unit: () = <()>::from_json_payload(p)?;
        println!("Workflow completed");
    }

    Ok(())
}
