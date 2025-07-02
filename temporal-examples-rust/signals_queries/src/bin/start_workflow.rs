//! Client binary to start workflow instance.
use env_logger::Env;
use helpers::get_client;
use log::info;
use serde_json::json;
use signals_queries::workflow::TASK_QUEUE;
use temporal_client::{WorkflowClientTrait, WorkflowOptions};
use temporal_sdk_core_protos::coresdk::AsJsonPayloadExt;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let client = get_client().await?;
    let id_arg = std::env::args()
        .nth(1)
        .unwrap_or_else(|| format!("signals-queries-{}", Uuid::new_v4()));

    let input = vec![json!("").as_json_payload()?];

    let workflow_id = id_arg;
    info!(target: "signals_queries", "Starting workflow with id={workflow_id}");

    let start_res = client
        .start_workflow(
            input,
            TASK_QUEUE.to_string(),
            workflow_id.clone(),
            "signals_queries_workflow".to_string(),
            None,
            WorkflowOptions::default(),
        )
        .await?;

    println!(
        "Started workflow: id={workflow_id} run_id={}",
        start_res.run_id
    );
    Ok(())
}
