//! Client to start expense workflow instance
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
    let args: Vec<String> = std::env::args().collect();
    let amount: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(42.5);
    let requester = args.get(2).cloned().unwrap_or_else(|| "alice".to_owned());
    let description = args.get(3).cloned().unwrap_or_else(|| "dinner".to_owned());
    let input = json!({
        "amount": amount,
        "requester": requester,
        "description": description
    });
    let client = get_client().await?;
    let workflow_id = format!("expense-{}", Uuid::new_v4());
    info!("Starting expense workflow: id={workflow_id} ...");
    let result = client
        .start_workflow(
            vec![input.to_string().as_json_payload()?],
            "expense".to_string(),
            workflow_id.clone(),
            "expense_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;
    println!("Started workflow id={workflow_id} run_id={}", result.run_id);
    // wait for workflow completion
    let handle = client.get_untyped_workflow_handle(workflow_id, result.run_id);
    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();
    let result_string: String = String::from_json_payload(&res_payloads[0])?;
    println!("Workflow completed: {result_string}");
    Ok(())
}
