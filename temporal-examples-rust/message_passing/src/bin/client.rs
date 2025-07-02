use env_logger::Env;
use helpers::get_client;
use log::info;

use temporal_client::WorkflowOptions;
use temporal_client::{WfClientExt, WorkflowClientTrait};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use temporal_sdk_core_protos::temporal::api::common::v1::Payload;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse arguments: messages as comma-separated, or default.
    let msgs_arg = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Hello,world,from,producer".to_string());
    let messages: Vec<String> = msgs_arg.split(',').map(|s| s.trim().to_string()).collect();

    let client = get_client().await?;
    info!(target: "message_passing", "Client: launching consumer workflow");

    // 1. Start consumer workflow first
    let consumer_id = format!("consumer-{}", Uuid::new_v4());
    let consumer_start = client
        .start_workflow(
            vec![],
            "message-passing".to_string(),
            consumer_id.clone(),
            "consumer_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;
    info!(target: "message_passing", "Started consumer id={consumer_id} run_id={}", consumer_start.run_id);

    // 2. Start producer workflow, giving consumer_id and messages
    let prod_input = message_passing::producer_workflow::ProducerInput {
        consumer_id: consumer_id.clone(),
        messages: messages.clone(),
    };
    let prod_id = format!("producer-{}", Uuid::new_v4());
    let prod_payload = serde_json::to_value(&prod_input)?.as_json_payload()?;
    client
        .start_workflow(
            vec![prod_payload],
            "message-passing".to_string(),
            prod_id.clone(),
            "producer_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;
    info!(target: "message_passing", "Started producer id={prod_id}");

    // 3. Wait for consumer workflow result and print.
    let consumer_handle =
        client.get_untyped_workflow_handle(consumer_id.clone(), consumer_start.run_id.clone());
    let payloads: Vec<Payload> = consumer_handle
        .get_workflow_result(Default::default())
        .await?
        .unwrap_success();
    let res_payload = payloads
        .first()
        .ok_or_else(|| anyhow::anyhow!("no consumer result payload"))?;
    let result: Vec<String> = serde_json::from_slice(&res_payload.data)?;
    println!("Consumer workflow completed, collected messages: {result:?}");
    Ok(())
}
