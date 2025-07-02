//! Client for search_attributes example.
//!
//! Accepts a customer id as CLI arg, starts workflow on "search-attributes" queue
//! and demonstrates querying via list_workflows for the search attribute.

use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;

use temporal_client::{WfClientExt, WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Customer id from CLI arg, or default.
    let customer_id = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sample-cust-42".to_string());
    let client = get_client().await?;

    info!(
        "search_attributes: Starting set_search_attribute workflow for customer_id={customer_id}"
    );

    let payload = json!(customer_id.clone()).as_json_payload()?;
    let input = vec![payload];

    let workflow_id = format!("search-attr-{}", Uuid::new_v4());

    let start_res = client
        .start_workflow(
            input,
            "search-attributes".to_string(),
            workflow_id.clone(),
            "set_search_attribute".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    println!(
        "Started workflow with id={workflow_id}, run_id={}",
        start_res.run_id
    );

    // Immediately query open workflows with CustomerId attribute.
    let _query = format!("CustomerId=\"{customer_id}\"");
    // Note: Listing by search attribute is not yet available in this SDK version; please see README for CLI example.
    println!("(Rust SDK: 'list_workflows' by search attribute is not yet implemented; use temporal CLI or web UI!)");

    // Wait for workflow to complete
    let handle = client.get_untyped_workflow_handle(workflow_id.clone(), start_res.run_id.clone());
    let res_payloads: Vec<Payload> = handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();

    println!(
        "Workflow completed, result payload count: {}",
        res_payloads.len()
    );

    Ok(())
}
