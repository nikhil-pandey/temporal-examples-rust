//! Client binary for activities_dependency_injection example.
use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::{AsJsonPayloadExt, FromJsonPayloadExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "World".to_string());
    let client = get_client().await?;

    info!("Starting multi_greet_workflow execution for name={name}");
    let input = vec![json!(name).as_json_payload()?];
    let workflow_id = format!("activities-di-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "activities-dep-inject".to_string(),
            workflow_id.clone(),
            "multi_greet_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    let handle = client.get_untyped_workflow_handle(workflow_id, start_res.run_id);
    let res_payloads: Vec<temporal_sdk_core_protos::temporal::api::common::v1::Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    let first = res_payloads.first().expect("missing result payload");
    let greetings: String = String::from_json_payload(first)?;
    println!("Workflow completed greetings:\n{greetings}");
    Ok(())
}
use temporal_client::WfClientExt;
